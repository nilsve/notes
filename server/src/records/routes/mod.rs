use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put};
use uuid::Uuid;
use anyhow::Result;

use orm::prelude::{CrudService, DynamoRepositoryError};
use orm::server::ActixAnyhow;
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;
use crate::helpers::{Truncatable};

use crate::records::models::{NewRecordDTO, RecordDTO};

use crate::records::service::RecordsService;

pub fn get_routes() -> actix_web::Scope {
    actix_web::web::scope("/records")
        .service(get_records)
        .service(get_record_by_id)
        .service(create_record)
        .service(update_record)
        .service(delete_record_by_id)
}

// Actix route for retrieving all records
// #[api_operation(summary = "List all records")]
#[get("")]
async fn get_records(
    records_service: Data<RecordsService>,
) -> Result<Json<Vec<RecordDTO>>, DynamoRepositoryError> {
    Ok(Json(records_service.find_all().await?.into_iter().map(|record| {
        let mut record_dto: RecordDTO = record.into();

        record_dto.body = record_dto.body.truncate_with_dots(100);

        record_dto
    }).collect()))
}

#[get("/{id}")]
async fn get_record_by_id(
    path: Path<Uuid>,
    records_service: Data<RecordsService>,
) -> Json<Option<RecordDTO>> {
    Json(records_service.find_by_id(path.into_inner()).await.unwrap().map(|record| record.into()))
}

#[delete("/{id}")]
async fn delete_record_by_id(
    path: Path<Uuid>,
    records_service: Data<RecordsService>,
    weaviate_service: Data<WeaviateService>,
) -> ActixAnyhow<Json<RecordDTO>> {
    let record = records_service.find_by_id(path.into_inner()).await.map_err(anyhow::Error::from)?;

    if let Some(record) = &record {
        records_service.delete(record.clone()).await.map_err(anyhow::Error::from)?;
        weaviate_service.delete_record(record).await.map_err(anyhow::Error::from)?;
    }

    Ok(Json(record.map(|record| record.into()).unwrap()))
}

#[post("")]
async fn create_record(
    record: Json<NewRecordDTO>,
    records_service: Data<RecordsService>,
    ai_service: Data<SentenceEncoderService>,
    weaviate_service: Data<WeaviateService>,
) -> ActixAnyhow<Json<RecordDTO>> {
    Ok(Json(records_service.create_record(&record, ai_service, weaviate_service).await?.into()))
}

#[put("/{id}")]
async fn update_record(
    path: Path<Uuid>,
    record: Json<RecordDTO>,
    records_service: Data<RecordsService>,
    ai_service: Data<SentenceEncoderService>,
    weaviate_service: Data<WeaviateService>,
) -> ActixAnyhow<Json<RecordDTO>> {
    Ok(Json(
        records_service
            .update_record(path.into_inner(), &record.into_inner().into(), ai_service, weaviate_service)
            .await?.into(),
    ))
}
