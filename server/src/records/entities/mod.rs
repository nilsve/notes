use apistos::ApiComponent;
use rust_bert::pipelines::sentence_embeddings::Embedding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use orm::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct RecordEntity {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub encoded: Option<Embedding>,
    // created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordIndex {}

#[derive(Debug, Clone, Serialize)]
pub struct RecordPrimaryKey {
    pub pk: String,
    pub sk: String,
}

impl Entity for RecordEntity {
    type PrimaryKey = RecordPrimaryKey;
    type IndexFields = RecordIndex;

    fn get_primary_key(&self) -> Self::PrimaryKey {
        RecordPrimaryKey {
            pk: "RECORD".to_string(),
            sk: format!("RECORD_ID#{}", self.id),
        }
    }

    fn get_index_fields(&self) -> Self::IndexFields {
        RecordIndex {}
    }
}
