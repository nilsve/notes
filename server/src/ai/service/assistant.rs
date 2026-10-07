use std::env;
use std::sync::Arc;

use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::chat_completion::ChatCompletionRequest;
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, MessageRole};

use crate::records::entities::RecordEntity;

/// Default model used when `OPENAI_MODEL` is not set.
const DEFAULT_MODEL: &str = "gpt-4o-mini";

#[derive(Clone)]
pub struct AssistantService {
    client: Arc<OpenAIClient>,
    model: String,
}

impl AssistantService {
    /// Creates a client for any OpenAI compatible server.
    ///
    /// Environment variables:
    /// * `OPENAI_API_KEY`  - required, api key of your server
    /// * `OPENAI_API_BASE` - optional, base url of your OpenAI compatible server
    ///   (e.g. `http://localhost:1234/v1`). Defaults to `https://api.openai.com/v1`.
    /// * `OPENAI_MODEL`    - optional, model name served by your server.
    ///   Defaults to `gpt-4o-mini`.
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

    pub async fn ask_question(&self, question: &str, records: &[RecordEntity]) -> anyhow::Result<String> {
        let mut starting_message =
            "Here comes a list of relevant records. Derive your answer from these records\n".to_string();

        for record in records {
            starting_message.push_str(&format!(
                "TITLE: {}, BODY: {}",
                record.title.clone(),
                record.body.clone()
            ));
        }

        starting_message.push_str("\nReply in Markdown format. Here comes the question:\n");

        let request = ChatCompletionRequest::new(
            self.model.clone(),
            vec![
                ChatCompletionMessage {
                    role: MessageRole::system,
                    content: Content::Text(starting_message),
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
            ],
        );

        let response = self.client.chat_completion(request).await?;

        let answer = response
            .inner
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(answer)
    }
}
