use aws_sdk_dynamodb::Client;
use serde::Serialize;
use uuid::Uuid;

use orm::prelude::{DynamoRepository, RepositoryIndex};

use crate::records::entities::RecordEntity;

/// DynamoDB table used for storing records. Override with the `DYNAMO_TABLE` env var.
fn table_name() -> &'static str {
    static TABLE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TABLE
        .get_or_init(|| std::env::var("DYNAMO_TABLE").unwrap_or_else(|_| "records".to_string()))
        .as_str()
}

#[derive(Clone)]
pub struct DynamoRecordsRepository {
    client: Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordPrimaryIndex {
    pk: String,
    sk: String,
}

impl RecordPrimaryIndex {
    pub fn find_by_id(uuid: Uuid) -> Self {
        Self {
            pk: "RECORD".to_string(),
            sk: format!("RECORD_ID#{}", uuid),
        }
    }
}

impl RepositoryIndex for RecordPrimaryIndex {}

impl DynamoRecordsRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl DynamoRepository<RecordEntity> for DynamoRecordsRepository {
    fn get_table_name(&self) -> &'static str {
        table_name()
    }

    fn get_client(&self) -> &'_ Client {
        &self.client
    }
}
