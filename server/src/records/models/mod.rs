use apistos::ApiComponent;
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::records::entities::RecordEntity;

/// Payload for creating a record. Timestamps are assigned by the database.
#[derive(Debug, Clone, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct NewRecordDTO {
    pub title: String,
    pub body: String,
}

/// Payload for updating a record. Timestamps are assigned by the database.
#[derive(Debug, Clone, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct UpdateRecordDTO {
    pub title: String,
    pub body: String,
}

impl From<NewRecordDTO> for RecordEntity {
    fn from(new_record: NewRecordDTO) -> Self {
        let now = Utc::now();

        RecordEntity {
            id: Uuid::new_v4(),
            title: new_record.title,
            body: new_record.body,
            encoded: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Timestamps are exposed as RFC 3339 strings.
#[derive(Debug, Clone, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct RecordDTO {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RecordEntity> for RecordDTO {
    fn from(record: RecordEntity) -> Self {
        RecordDTO {
            id: record.id,
            title: record.title,
            body: record.body,
            created_at: record.created_at.to_rfc3339(),
            updated_at: record.updated_at.to_rfc3339(),
        }
    }
}
