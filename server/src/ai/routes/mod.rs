use actix_web::{get, post, web, web::{Data, Json}, HttpRequest, HttpResponse, Responder};
use actix_ws::Message;
use futures_util::StreamExt;
use openai_api_rs::v1::chat_completion::chat_completion_stream::ChatCompletionStreamResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ai::service::assistant::AssistantService;
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;
use crate::records::entities::RecordEntity;
use crate::records::service::RecordsService;

pub fn get_routes() -> actix_web::Scope {
    actix_web::web::scope("/ai")
        .service(query)
        .service(query_ws)
}

#[derive(Debug, Deserialize)]
struct QuestionModel {
    question: String,
}

#[derive(Debug, Serialize)]
struct ResponseModel {
    answer: String,
}

/// One streaming event, as a JSON text frame.
fn event(value: serde_json::Value) -> String {
    value.to_string()
}

/// Runs the RAG stage: embed the question, find the closest records, hydrate them.
async fn retrieve_context(
    question: &str,
    records_service: &RecordsService,
    weaviate_service: &WeaviateService,
    encoding_service: &SentenceEncoderService,
) -> actix_web::Result<Vec<RecordEntity>> {
    let encoded_question = encoding_service.encode_string(question.to_string()).await;

    let result = weaviate_service
        .query_records(encoded_question)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    result
        .get_records(records_service)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)
}

/// Non-streaming ask: the caller waits for the whole answer.
#[post("")]
async fn query(
    question: Json<QuestionModel>,
    records_service: Data<RecordsService>,
    weaviate_service: Data<WeaviateService>,
    incident_ai_service: Data<AssistantService>,
    encoding_service: Data<SentenceEncoderService>,
) -> actix_web::Result<HttpResponse> {
    let result_records = retrieve_context(
        &question.question,
        &records_service,
        &weaviate_service,
        &encoding_service,
    )
    .await?;

    let answer = incident_ai_service
        .ask_question(&question.question, &result_records)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ResponseModel { answer }))
}

/// Streaming ask over a WebSocket at `/api/ai/ws`.
///
/// The client upgrades, then sends one JSON message per question:
///
/// ```json
/// {"question":"Have we seen a 500 on checkout before?"}
/// ```
///
/// and receives JSON text frames back as they are produced:
///
/// ```json
/// {"type":"reasoning","delta":"The user asks ..."}
/// {"type":"answer","delta":"Yes — "}
/// {"type":"done"}
/// {"type":"error","message":"..."}
/// ```
///
/// A WebSocket rather than a streamed `POST` because HTTP responses get buffered
/// by middleboxes and clients, which defeats the point; a socket is a real
/// bidirectional channel and each frame lands as it is sent.
///
/// The connection stays open for further questions until the client closes it.
#[get("/ws")]
async fn query_ws(
    req: HttpRequest,
    body: web::Payload,
    records_service: web::Data<RecordsService>,
    weaviate_service: web::Data<WeaviateService>,
    incident_ai_service: web::Data<AssistantService>,
    encoding_service: web::Data<SentenceEncoderService>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        loop {
            // Wait for the next question from this client.
            let question = match next_question(&mut session, &mut msg_stream).await {
                Some(question) => question,
                None => break,
            };

            let records = match retrieve_context(
                &question,
                &records_service,
                &weaviate_service,
                &encoding_service,
            )
            .await
            {
                Ok(records) => records,
                Err(err) => {
                    let _ = session
                        .text(event(json!({
                            "type": "error",
                            "message": format!("Could not gather context: {err}"),
                        })))
                        .await;
                    continue;
                }
            };

            let mut stream = match incident_ai_service.ask_question_stream(&question, &records).await {
                Ok(stream) => stream,
                Err(err) => {
                    let _ = session
                        .text(event(json!({
                            "type": "error",
                            "message": format!("Could not reach the model: {err}"),
                        })))
                        .await;
                    continue;
                }
            };

            let mut finished = false;

            while let Some(item) = stream.next().await {
                let frame = match item {
                    Ok(ChatCompletionStreamResponse::Reasoning(delta)) => {
                        event(json!({ "type": "reasoning", "delta": delta }))
                    }
                    Ok(ChatCompletionStreamResponse::Content(delta)) => {
                        event(json!({ "type": "answer", "delta": delta }))
                    }
                    Ok(ChatCompletionStreamResponse::Done) => {
                        finished = true;
                        event(json!({ "type": "done" }))
                    }
                    Ok(ChatCompletionStreamResponse::ToolCall(_)) => continue,
                    Err(err) => {
                        finished = true;
                        event(json!({
                            "type": "error",
                            "message": format!("Model stream failed: {err}"),
                        }))
                    }
                };

                // A failed send means the client went away; drop the model stream.
                if session.text(frame).await.is_err() {
                    return;
                }

                if finished {
                    break;
                }
            }

            if !finished {
                let _ = session.text(event(json!({ "type": "done" }))).await;
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

/// Reads frames until a usable question arrives. Returns `None` when the client
/// closed the connection.
async fn next_question(
    session: &mut actix_ws::Session,
    msg_stream: &mut actix_ws::MessageStream,
) -> Option<String> {
    while let Some(message) = msg_stream.recv().await {
        match message {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<QuestionModel>(&text) {
                    Ok(model) if !model.question.trim().is_empty() => return Some(model.question),
                    Ok(_) => {
                        let _ = session
                            .text(event(json!({
                                "type": "error",
                                "message": "Question is empty",
                            })))
                            .await;
                    }
                    Err(_) => {
                        let _ = session
                            .text(event(json!({
                                "type": "error",
                                "message": "Expected JSON like {\"question\":\"...\"}",
                            })))
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => return None,
            Err(_) => return None,
            // Ping/pong/binary frames are handled by the session; keep waiting.
            Ok(_) => continue,
        }
    }

    None
}
