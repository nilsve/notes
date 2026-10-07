use actix_web::{post};
use actix_web::web::{Data, Json};
use serde::{Deserialize, Serialize};
use crate::ai::service::assistant::AssistantService;
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;
use crate::records::service::RecordsService;

pub fn get_routes() -> actix_web::Scope {
    actix_web::web::scope("/ai")
        .service(query)
}

#[derive(Debug, Deserialize)]
struct QuestionModel {
    question: String,
}

#[derive(Debug, Serialize)]
struct ResponseModel {
    answer: String,
}


#[post("")]
async fn query(
    question: Json<QuestionModel>,
    records_service: Data<RecordsService>,
    weaviate_service: Data<WeaviateService>,
    incident_ai_service: Data<AssistantService>,
    encoding_service: Data<SentenceEncoderService>,
) -> Json<ResponseModel> {
    let encoded_question = encoding_service.encode_string(question.0.question.clone()).await;

    let result = weaviate_service.query_records(encoded_question).await.unwrap();

    let result_records = result.get_records(&records_service).await.unwrap();

    let answer = incident_ai_service
        .ask_question(&question.0.question, &result_records)
        .await
        .unwrap();

    println!("Result: {:?}", answer);

    Json(ResponseModel { answer })
}