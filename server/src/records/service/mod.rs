use actix_web::web::Data;
use orm::prelude::{
    CrudService, DynamoRepositoryError, LastEvaluatedKey, QueryData, QueryResult, RepositoryIndex,
};
use uuid::Uuid;
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;

use crate::records::entities::{RecordEntity};
use crate::records::models::NewRecordDTO;
use crate::records::repository::{DynamoRecordsRepository, RecordPrimaryIndex};

#[derive(Clone)]
pub struct RecordsService {
    repository: DynamoRecordsRepository,
}

impl CrudService<RecordEntity, DynamoRecordsRepository> for RecordsService {
    fn get_repository(&self) -> &DynamoRecordsRepository {
        &self.repository
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryRecordIndex {
    pk: String,
}

impl QueryRecordIndex {
    pub fn find_all() -> Self {
        Self {
            pk: "RECORD".to_string(),
        }
    }
}

impl RepositoryIndex for QueryRecordIndex {}

impl RecordsService {
    pub fn new(repository: DynamoRecordsRepository) -> Self {
        Self { repository }
    }

    pub async fn find_by_id(
        &self,
        uuid: Uuid,
    ) -> Result<Option<RecordEntity>, DynamoRepositoryError> {
        self.find(RecordPrimaryIndex::find_by_id(uuid)).await
    }

    pub async fn find_all_paged(
        &self,
        last_evaluated_key: Option<LastEvaluatedKey>,
    ) -> Result<QueryResult<RecordEntity>, DynamoRepositoryError> {
        self.query(QueryData::new(
            QueryRecordIndex::find_all(),
            last_evaluated_key,
        ))
            .await
    }

    pub async fn find_all(&self) -> Result<Vec<RecordEntity>, DynamoRepositoryError> {
        self.query_all(QueryRecordIndex::find_all()).await
    }

    pub async fn create_record(
        &self,
        record: &NewRecordDTO,
        ai_service: Data<SentenceEncoderService>,
        weaviate_service: Data<WeaviateService>,
    ) -> Result<RecordEntity, anyhow::Error> {
        let record: RecordEntity = record.to_owned().into();
        self.create(record.clone()).await?;

        let record = ai_service.encode_record(record.clone()).await;

        weaviate_service.insert_record(&record).await?;

        self.upsert(record.clone()).await?;

        Ok(record)
    }

    pub async fn update_record(
        &self,
        record_id: Uuid,
        record: &RecordEntity,
        ai_service: Data<SentenceEncoderService>,
        weaviate_service: Data<WeaviateService>,
    ) -> Result<RecordEntity, anyhow::Error> {
        // Check if record exists
        self.find(RecordPrimaryIndex::find_by_id(record_id))
            .await?
            .ok_or(DynamoRepositoryError::ItemNotFoundError)?;

        let entity = RecordEntity {
            id: record_id,
            ..record.clone()
        };

        self.upsert(entity.clone()).await?;

        let record = ai_service.encode_record(entity.clone()).await;

        weaviate_service.update_record(&record).await?;

        self.upsert(record).await?;

        Ok(entity)
    }
}
