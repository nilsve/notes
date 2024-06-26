use apistos::ApiComponent;
use rust_bert::pipelines::sentence_embeddings::Embedding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use orm::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct NoteEntity {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub encoded: Option<Embedding>,
    // created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoteIndex {}

#[derive(Debug, Clone, Serialize)]
pub struct NotePrimaryKey {
    pub pk: String,
    pub sk: String,
}

impl Entity for NoteEntity {
    type PrimaryKey = NotePrimaryKey;
    type IndexFields = NoteIndex;

    fn get_primary_key(&self) -> Self::PrimaryKey {
        NotePrimaryKey {
            pk: "NOTE".to_string(),
            sk: format!("NOTE_ID#{}", self.id),
        }
    }

    fn get_index_fields(&self) -> Self::IndexFields {
        NoteIndex {}
    }
}
