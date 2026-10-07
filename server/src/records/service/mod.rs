use actix_web::web::Data;
use chrono::Utc;
use orm::prelude::{CrudService, RepositoryError};
use uuid::Uuid;

use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;
use crate::records::entities::RecordEntity;
use crate::records::models::{NewRecordDTO, UpdateRecordDTO};
use crate::records::repository::PostgresRecordRepository;

#[derive(Clone)]
pub struct RecordsService {
    repository: PostgresRecordRepository,
}

impl CrudService<RecordEntity, PostgresRecordRepository> for RecordsService {
    fn get_repository(&self) -> &PostgresRecordRepository {
        &self.repository
    }
}

impl RecordsService {
    pub fn new(repository: PostgresRecordRepository) -> Self {
        Self { repository }
    }

    pub async fn find_by_id(&self, uuid: Uuid) -> Result<Option<RecordEntity>, RepositoryError> {
        self.find(uuid).await
    }

    pub async fn find_all(&self) -> Result<Vec<RecordEntity>, RepositoryError> {
        self.all().await
    }

    /// Persists the record, indexes it semantically and stores the embedding back
    /// on the row.
    pub async fn create_record(
        &self,
        record: &NewRecordDTO,
        ai_service: Data<SentenceEncoderService>,
        weaviate_service: Data<WeaviateService>,
    ) -> Result<RecordEntity, anyhow::Error> {
        let entity: RecordEntity = record.to_owned().into();
        let persisted = self.create(entity).await?;

        let encoded = ai_service.encode_record(persisted.clone()).await;
        weaviate_service.insert_record(&encoded).await?;

        let saved = self.upsert(encoded).await?;

        Ok(saved)
    }

    pub async fn update_record(
        &self,
        record_id: Uuid,
        update: &UpdateRecordDTO,
        ai_service: Data<SentenceEncoderService>,
        weaviate_service: Data<WeaviateService>,
    ) -> Result<RecordEntity, anyhow::Error> {
        let existing = self
            .find(record_id)
            .await?
            .ok_or(RepositoryError::ItemNotFoundError)?;

        let entity = RecordEntity {
            id: record_id,
            title: update.title.clone(),
            body: update.body.clone(),
            encoded: existing.encoded,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };

        self.upsert(entity.clone()).await?;

        let encoded = ai_service.encode_record(entity).await;
        weaviate_service.update_record(&encoded).await?;

        let saved = self.upsert(encoded).await?;

        Ok(saved)
    }

    pub async fn delete_record(
        &self,
        record_id: Uuid,
        weaviate_service: Data<WeaviateService>,
    ) -> Result<RecordEntity, anyhow::Error> {
        let existing = self
            .find(record_id)
            .await?
            .ok_or(RepositoryError::ItemNotFoundError)?;

        self.delete(record_id).await?;
        weaviate_service.delete_record(&existing).await?;

        Ok(existing)
    }
}
