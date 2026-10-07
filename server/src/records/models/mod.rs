use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::records::entities::RecordEntity;

#[derive(Debug, Clone, Deserialize, ApiComponent, JsonSchema)]
pub struct NewRecordDTO {
    pub title: String,
    pub body: String,
}

impl From<NewRecordDTO> for RecordEntity {
    fn from(new_record: NewRecordDTO) -> Self {
        RecordEntity {
            id: Uuid::new_v4(),
            title: new_record.title,
            body: new_record.body,
            encoded: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct RecordDTO {
    pub id: Uuid,
    pub title: String,
    pub body: String,
}

impl From<RecordEntity> for RecordDTO {
    fn from(record: RecordEntity) -> Self {
        RecordDTO {
            id: record.id,
            title: record.title,
            body: record.body,
        }
    }
}

impl From<RecordDTO> for RecordEntity {
    fn from(record: RecordDTO) -> Self {
        RecordEntity {
            id: record.id,
            title: record.title,
            body: record.body,
            encoded: None,
        }
    }
}

