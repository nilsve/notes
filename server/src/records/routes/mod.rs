use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put};
use uuid::Uuid;

use orm::prelude::{ActixAnyhow, RepositoryError};
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;
use crate::helpers::Truncatable;

use crate::records::models::{NewRecordDTO, RecordDTO, UpdateRecordDTO};

use crate::records::service::RecordsService;

pub fn get_routes() -> actix_web::Scope {
    actix_web::web::scope("/records")
        .service(get_records)
        .service(get_record_by_id)
        .service(create_record)
        .service(update_record)
        .service(delete_record_by_id)
}

#[get("")]
async fn get_records(records_service: Data<RecordsService>) -> Json<Vec<RecordDTO>> {
    let records = records_service.find_all().await.unwrap_or_default();

    Json(
        records
            .into_iter()
            .map(|record| {
                let mut record_dto: RecordDTO = record.into();

                // The list view only needs a preview of the body.
                record_dto.body = record_dto.body.truncate_with_dots(140);

                record_dto
            })
            .collect(),
    )
}

#[get("/{id}")]
async fn get_record_by_id(
    path: Path<Uuid>,
    records_service: Data<RecordsService>,
) -> Result<Json<Option<RecordDTO>>, RepositoryError> {
    let record = records_service.find_by_id(path.into_inner()).await?;

    Ok(Json(record.map(|record| record.into())))
}

#[post("")]
async fn create_record(
    record: Json<NewRecordDTO>,
    records_service: Data<RecordsService>,
    ai_service: Data<SentenceEncoderService>,
    weaviate_service: Data<WeaviateService>,
) -> ActixAnyhow<Json<RecordDTO>> {
    let created = records_service
        .create_record(&record, ai_service, weaviate_service)
        .await?;

    Ok(Json(created.into()))
}

#[put("/{id}")]
async fn update_record(
    path: Path<Uuid>,
    record: Json<UpdateRecordDTO>,
    records_service: Data<RecordsService>,
    ai_service: Data<SentenceEncoderService>,
    weaviate_service: Data<WeaviateService>,
) -> ActixAnyhow<Json<RecordDTO>> {
    let updated = records_service
        .update_record(path.into_inner(), &record, ai_service, weaviate_service)
        .await?;

    Ok(Json(updated.into()))
}

#[delete("/{id}")]
async fn delete_record_by_id(
    path: Path<Uuid>,
    records_service: Data<RecordsService>,
    weaviate_service: Data<WeaviateService>,
) -> ActixAnyhow<Json<RecordDTO>> {
    let deleted = records_service
        .delete_record(path.into_inner(), weaviate_service)
        .await?;

    Ok(Json(deleted.into()))
}
