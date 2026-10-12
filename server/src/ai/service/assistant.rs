use std::env;
use std::sync::Arc;

use futures_util::Stream;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::chat_completion::ChatCompletionRequest;
use openai_api_rs::v1::chat_completion::chat_completion_stream::{
    ChatCompletionStreamRequest, ChatCompletionStreamResponse,
};
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, Content, MessageRole, ReasoningEffort,
};

use crate::records::entities::RecordEntity;

/// Default model used when `OPENAI_MODEL` is not set.
const DEFAULT_MODEL: &str = "gpt-4o-mini";

/// Reasoning effort requested from the model when `OPENAI_REASONING_EFFORT` is
/// not set.
const DEFAULT_REASONING_EFFORT: &str = "medium";

/// Reasoning effort requested on every completion.
///
/// `OPENAI_REASONING_EFFORT`: `none` | `minimal` | `low` | `medium` | `high` |
/// `xhigh`. Defaults to `medium`.
///
/// Note that `none` is sent explicitly rather than omitted: omitting the field
/// leaves the decision to the server, and the bundled endpoint thinks by default.
/// Levels above `none` are advisory on some deployments - measured on halogen,
/// `low` produced *more* reasoning tokens than `high`.
fn reasoning_effort() -> ReasoningEffort {
    let requested = env::var("OPENAI_REASONING_EFFORT")
        .unwrap_or_else(|_| DEFAULT_REASONING_EFFORT.to_string())
        .trim()
        .to_ascii_lowercase();

    match requested.as_str() {
        "none" => ReasoningEffort::None,
        "minimal" => ReasoningEffort::Minimal,
        "low" => ReasoningEffort::Low,
        "high" => ReasoningEffort::High,
        "xhigh" => ReasoningEffort::Xhigh,
        "medium" | _ => ReasoningEffort::Medium,
    }
}

/// System prompt: the retrieved records the answer has to be derived from.
fn build_system_prompt(records: &[RecordEntity]) -> String {
    let mut prompt =
        "Here comes a list of relevant records. Derive your answer from these records\n".to_string();

    for record in records {
        prompt.push_str(&format!("TITLE: {}, BODY: {}", record.title, record.body));
    }

    prompt.push_str("\nReply in Markdown format. Here comes the question:\n");
    prompt
}

fn build_messages(question: &str, records: &[RecordEntity]) -> Vec<ChatCompletionMessage> {
    vec![
        ChatCompletionMessage {
            role: MessageRole::system,
            content: Content::Text(build_system_prompt(records)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
        ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text(question.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
    ]
}

#[derive(Clone)]
pub struct AssistantService {
    client: Arc<OpenAIClient>,
    model: String,
}

impl AssistantService {
    /// Creates a client for any OpenAI compatible server.
    ///
    /// Environment variables:
    /// * `OPENAI_API_KEY`          - required, api key of your server
    /// * `OPENAI_API_BASE`         - optional, base url of your OpenAI compatible server
    ///   (e.g. `http://localhost:1234/v1`). Defaults to `https://api.openai.com/v1`.
    /// * `OPENAI_MODEL`            - optional, model name served by your server.
    ///   Defaults to `gpt-4o-mini`.
    /// * `OPENAI_REASONING_EFFORT` - optional, see [`reasoning_effort`].
    pub fn new() -> Self {
        let api_key = env::var("OPENAI_API_KEY").expect("Couldnt find OpenAI api key in env");

        let mut builder = OpenAIClient::builder().with_api_key(api_key);

        if let Ok(base) = env::var("OPENAI_API_BASE") {
            if !base.is_empty() {
                builder = builder.with_endpoint(base);
            }
        }

        let client = Arc::new(builder.build().expect("Couldnt create OpenAI client"));

        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Self { client, model }
    }

    /// Non-streaming ask. Returns only the final answer; any chain of thought the
    /// model produced is discarded.
    pub async fn ask_question(&self, question: &str, records: &[RecordEntity]) -> anyhow::Result<String> {
        let request = ChatCompletionRequest::new(self.model.clone(), build_messages(question, records))
            .reasoning_effort(reasoning_effort());

        let response = self.client.chat_completion(request).await?;

        let answer = response
            .inner
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(answer)
    }

    /// Streaming ask. Yields the model's output incrementally, keeping reasoning
    /// and answer as separate events so the caller can render them apart.
    ///
    /// Errors from the transport surface as `Err` items on the stream rather than
    /// silently truncating it.
    pub async fn ask_question_stream(
        &self,
        question: &str,
        records: &[RecordEntity],
    ) -> anyhow::Result<impl Stream<Item = Result<ChatCompletionStreamResponse, openai_api_rs::v1::error::APIError>>> {
        let request =
            ChatCompletionStreamRequest::new(self.model.clone(), build_messages(question, records))
                .reasoning_effort(reasoning_effort());

        let stream = self.client.try_chat_completion_stream(request).await?;

        Ok(stream)
    }
}
