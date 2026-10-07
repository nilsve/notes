use std::env;
use std::error::Error;
use std::sync::Arc;
use rust_bert::pipelines::sentence_embeddings::Embedding;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;
use weaviate_community::collections::objects::{MultiObjects, Object, ObjectListParameters};
use weaviate_community::collections::query::ExploreQuery;
use weaviate_community::collections::schema::{Class, Properties, Property};
use weaviate_community::WeaviateClient;
use crate::records::entities::RecordEntity;
use crate::records::service::RecordsService;

/// Weaviate class used for storing record vectors. Override with `WEAVIATE_CLASS`.
fn record_class() -> &'static str {
    static CLASS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CLASS
        .get_or_init(|| std::env::var("WEAVIATE_CLASS").unwrap_or_else(|_| "Record".to_string()))
        .as_str()
}

#[derive(Error, Debug)]
pub enum WeaviateServiceError {
    #[error("Weaviate client error")]
    WeaviateClientError
}

impl From<Box<dyn Error>> for WeaviateServiceError {
    fn from(_error: Box<dyn Error>) -> Self {
        WeaviateServiceError::WeaviateClientError
    }
}

#[derive(Clone)]
pub struct WeaviateService {
    client: Arc<WeaviateClient>,
}

pub trait FromEmbeddingToF64 {
    fn to_f64_vec(&self) -> Vec<f64>;
}

impl FromEmbeddingToF64 for &Embedding {
    fn to_f64_vec(&self) -> Vec<f64> {
        self.iter().map(|value| *value as f64).collect()
    }
}

#[derive(Serialize, Debug)]
struct ExploreQueryParams {
    vector: Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct ExploreQueryResultDataItem {
    beacon: String,
    certainty: f64,
}

#[derive(Deserialize, Debug)]
pub struct ExploreQueryResultData {
    #[allow(non_snake_case)]
    Explore: Vec<ExploreQueryResultDataItem>,
}

#[derive(Deserialize, Debug)]
pub struct ExploreQueryResult {
    data: ExploreQueryResultData,
}

impl ExploreQueryResult {
    pub async fn get_records(&self, records_service: &RecordsService) -> anyhow::Result<Vec<RecordEntity>> {
        let mut result = Vec::new();

        for item in &self.data.Explore {
            let splitted = &item.beacon.split("/").collect::<Vec<&str>>();
            let input = splitted.get(splitted.len() - 1).expect("No uuid found");
            let id = Uuid::parse_str(input).unwrap();

            let record_entity = records_service.find_by_id(id).await.unwrap().expect("Weaviate returned non existing record!");
            result.push(record_entity);
        }

        Ok(result)
    }
}

impl From<&RecordEntity> for Object {
    fn from(record: &RecordEntity) -> Self {
        let mut builder = Object::builder(record_class(), record.into())
            .with_id(record.id);

        if let Some(vector) = &record.encoded {
            builder = builder.with_vector(vector.to_f64_vec());
        }

        builder.build()
    }
}

impl From<&RecordEntity> for Value {
    fn from(record: &RecordEntity) -> Self {
        serde_json::json!({
            "title": &record.title,
            "content": &record.body,
        })
    }
}

impl WeaviateService {
    pub async fn new() -> Result<Self, WeaviateServiceError> {
        println!("Creating Weaviate service");
        let hostname = env::var("WEAVIATE_HOSTNAME").unwrap();
        let port = env::var("WEAVIATE_PORT").unwrap();
        let url = format!("http://{}:{}", hostname, port);

        println!("Weaviate URL: {}", url);

        let client = WeaviateClient::builder(&url).build()?;

        let service = Self {
            client: Arc::new(client),
        };

        service.setup_database().await?;

        Ok(service)
    }

    pub async fn insert_record(
        &self,
        record: &RecordEntity,
    ) -> Result<(), WeaviateServiceError> {
        self.client.objects.create(&(record.into()), None).await?;

        Ok(())
    }

    pub async fn find_record(
        &self,
        record_id: &Uuid,
    ) -> Result<Object, WeaviateServiceError> {
        let record = self.client.objects.get(record_class(), record_id, None, None, None).await?;

        Ok(record)
    }

    pub async fn all_records(&self) -> Result<MultiObjects, WeaviateServiceError> {
        let records = self.client.objects.list(ObjectListParameters::builder().with_include("vector").with_class_name(record_class()).build()).await?;

        Ok(records)
    }

    pub async fn query_records(
        &self,
        query_vector: Embedding,
    ) -> Result<ExploreQueryResult, WeaviateServiceError> {
        let query_data = serde_json::to_string(&ExploreQueryParams {
            vector: (&query_vector).to_f64_vec(),
        }).unwrap().replace("\"", "");

        let query = ExploreQuery::builder()
            .with_limit(5)
            .with_near_vector(&query_data)
            .with_fields(vec!["beacon", "certainty"])
            .build();

        let json_value = self.client.query.explore(query).await?;

        let deserialized: ExploreQueryResult = serde_json::from_value(json_value).unwrap();

        Ok(deserialized)
        // let records = self.client.query.
    }

    pub async fn delete_record(
        &self,
        record: &RecordEntity,
    ) -> Result<(), WeaviateServiceError> {
        self.client.objects.delete(record_class(), &record.id, None, None).await?;

        Ok(())
    }

    pub async fn update_record(
        &self,
        record: &RecordEntity,
    ) -> Result<(), WeaviateServiceError> {
        match self.delete_record(record).await {
            Ok(_) => println!("Successfully deleted item for updating {:?}", record.id),
            Err(_) => println!("Coudln't delete item for updating {:?}", record.id)
        };

        self.insert_record(record).await?;

        Ok(())
    }

    async fn setup_database(&self) -> Result<(), WeaviateServiceError> {
        let schema = self.client.schema.get().await?;

        if !schema.classes.iter().any(|class| class.class == record_class()) {
            println!("Creating Record class");
            let class = Class::builder(record_class())
                .with_properties(Properties::new(vec![
                    Property::builder("title", vec!["text"]).build(),
                    Property::builder("content", vec!["text"]).build(),
                ]))
                .build();

            self.client.schema.create_class(&class).await?;
        } else {
            println!("Record class already exists");
        }

        Ok(())
    }

    pub fn get_client(&self) -> Arc<WeaviateClient> {
        self.client.clone()
    }
}
