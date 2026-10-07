use chrono::{DateTime, Utc};
use rust_bert::pipelines::sentence_embeddings::Embedding;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use orm::prelude::Entity;

/// Table holding the records.
pub const TABLE: &str = "records";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEntity {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    /// Sentence embedding of `title` + `body`, kept in sync with Weaviate.
    pub encoded: Option<Embedding>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for RecordEntity {
    type Pk = Uuid;

    fn primary_key(&self) -> Self::Pk {
        self.id
    }
}

/// `encoded` is stored as `jsonb`, which doesn't map onto `Vec<f32>` directly.
impl<'r> FromRow<'r, PgRow> for RecordEntity {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let encoded: Option<sqlx::types::Json<Vec<f32>>> = row.try_get("encoded")?;

        Ok(RecordEntity {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            body: row.try_get("body")?,
            encoded: encoded.map(|sqlx::types::Json(vector)| vector),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// DDL for the records table. Idempotent, so it runs on every boot.
pub fn migration() -> Vec<String> {
    vec![
        format!(
            r#"CREATE TABLE IF NOT EXISTS {TABLE} (
                id          UUID PRIMARY KEY,
                title       TEXT NOT NULL,
                body        TEXT NOT NULL,
                encoded     JSONB,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
            )"#
        ),
        format!(
            "CREATE INDEX IF NOT EXISTS {TABLE}_created_at_idx ON {TABLE} (created_at DESC)"
        ),
    ]
}
