use actix_cors::Cors;
use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::web::scope;
use aws_config::load_from_env;
use aws_sdk_dynamodb::Client;
use dotenvy::dotenv;
use env_logger::Env;
use std::env;
use std::path::PathBuf;

use orm::prelude::*;
use crate::ai::service::assistant::AssistantService;
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;

use crate::records::repository::DynamoRecordsRepository;
use crate::records::service::RecordsService;

mod ai;
mod records;
mod helpers;

#[actix_web::main]
async fn main() -> Result<(), DynamoRepositoryError> {
    println!("Starting server...");
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    };

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    println!("Loading AWS configurations...");

    let config = load_from_env().await;
    let client = Client::new(&config);

    let repository = DynamoRecordsRepository::new(client);
    let records_service = RecordsService::new(repository);
    let weaviate_service = WeaviateService::new().await.unwrap();
    let ai_service = SentenceEncoderService::new();
    let assistant_service = AssistantService::new();

    let path: PathBuf = env::var("FRONTEND_LOCATION")
        .unwrap_or_else(|_| "static".to_string())
        .into();

    println!("Looking for frontend files at: {:?}", path);

    let mut frontend_path = None;

    match path.canonicalize() {
        Ok(path_buf) => {
            println!("Serving frontend from: {:?}", path_buf);

            frontend_path = Some(path_buf);
        }
        Err(_) => {
            let path: PathBuf = "./static".into();

            match path.canonicalize() {
                Ok(path_buf) => {
                    println!("Serving frontend from: {:?}", path_buf);
                    frontend_path = Some(path_buf);
                }
                Err(_) => {
                    println!("Frontend not found at: {:?}, or /static. Not serving", path);
                }
            }
        }
    }

    println!("Starting web server...");

    // Start actix server
    actix_web::HttpServer::new(move || {
        let mut app = actix_web::App::new()
            .app_data(actix_web::web::Data::new(records_service.clone()))
            .app_data(actix_web::web::Data::new(ai_service.clone()))
            .app_data(actix_web::web::Data::new(weaviate_service.clone()))
            .app_data(actix_web::web::Data::new(assistant_service.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(scope("/api").service(records::routes::get_routes()).service(ai::routes::get_routes()));

        if let Some(path) = &frontend_path {
            app = app.service(Files::new("/", path).index_file("index.html"));
        }

        app
    })
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
        .unwrap();

    Ok(())
}

// Unit test
#[cfg(test)]
mod test {
    use std::str::FromStr;
    use aws_config::load_from_env;
    use aws_sdk_dynamodb::Client;
    use dotenvy::dotenv;
    use env_logger::Env;
    use uuid::Uuid;
    use crate::ai::service::assistant::AssistantService;
    use crate::ai::service::encoder::SentenceEncoderService;
    use crate::ai::service::weaviate::WeaviateService;
    use crate::records::entities::RecordEntity;
    use crate::records::repository::DynamoRecordsRepository;
    use crate::records::service::RecordsService;

    // Create test for updating weaviate object
    #[tokio::test]
    async fn test_update_record() {
        // Create a new record
        let record = RecordEntity {
            id: Uuid::from_str("5e9177dc-fd5b-4db5-b1e5-f108cd84a93c").unwrap(),
            title: "title".to_string(),
            body: "content".to_string(),
            encoded: None,
        };

        // Create a new weaviate service
        let weaviate_service = WeaviateService::new().await.unwrap();

        // Update the record
        let result = weaviate_service.all_records().await.unwrap();

        println!("Result: {:?}", result);
    }

    #[tokio::test]
    async fn test_querying() {
        let question = "What is the secret password?";

        match dotenv() {
            Ok(_) => println!("Loaded .env file"),
            Err(_) => println!("No .env file found"),
        };

        env_logger::init_from_env(Env::default().default_filter_or("info"));

        println!("Loading AWS configurations...");

        let config = load_from_env().await;
        let client = Client::new(&config);

        let encoding_service = SentenceEncoderService::new();
        let weaviate_service = WeaviateService::new().await.unwrap();
        let assistant_service = AssistantService::new();
        let dynamo_repository = DynamoRecordsRepository::new(client);
        let records_service = RecordsService::new(dynamo_repository);

        let question = encoding_service.encode_string(question.to_string()).await;

        let result = weaviate_service.query_records(question).await.unwrap();

        let result_records = result.get_records(&records_service).await.unwrap();

        let response_message = assistant_service
            .ask_question("What is the secret password?", &result_records)
            .await
            .unwrap();

        println!("Result: {:?}", response_message);
    }
}
