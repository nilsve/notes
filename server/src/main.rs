use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::dev::{fn_service, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
use actix_web::web::scope;
use actix_web::{HttpResponse, Responder};
use dotenvy::dotenv;
use env_logger::Env;
use std::env;
use std::path::PathBuf;

use orm::prelude::{connect, migrate, Db};

use crate::ai::service::assistant::AssistantService;
use crate::ai::service::encoder::SentenceEncoderService;
use crate::ai::service::weaviate::WeaviateService;

use crate::records::entities::migration as records_migration;
use crate::records::repository::PostgresRecordRepository;
use crate::records::service::RecordsService;

mod ai;
mod helpers;
mod records;

/// Connection string for the Postgres instance. Override with `DATABASE_URL`.
pub fn database_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://incidents:incidents@localhost:5432/incidents".to_string())
}

/// Port the app listens on. Override with `PORT`.
pub fn listen_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .unwrap_or(1337)
}

/// Connects to Postgres and applies the (idempotent) schema.
pub async fn setup_database() -> anyhow::Result<Db> {
    let url = database_url();

    println!("Connecting to Postgres at {}", redact(&url));

    let db = connect(&url).await?;

    println!("Applying schema...");

    let statements = records_migration();
    let statements: Vec<&str> = statements.iter().map(String::as_str).collect();
    migrate(&db, &statements).await?;

    Ok(db)
}

/// Hides credentials when logging a connection string.
fn redact(url: &str) -> String {
    match (url.find("://"), url.find('@')) {
        (Some(scheme_end), Some(at)) if at > scheme_end => {
            format!("{}://***{}", &url[..scheme_end], &url[at..])
        }
        _ => url.to_string(),
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting server...");

    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    };

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let db = setup_database().await?;

    let repository = PostgresRecordRepository::new(db);
    let records_service = RecordsService::new(repository);
    let weaviate_service = WeaviateService::new().await?;
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

    let port = listen_port();

    println!("Listening on 0.0.0.0:{} (API under /api)", port);

    actix_web::HttpServer::new(move || {
        let mut app = actix_web::App::new()
            .app_data(actix_web::web::Data::new(records_service.clone()))
            .app_data(actix_web::web::Data::new(ai_service.clone()))
            .app_data(actix_web::web::Data::new(weaviate_service.clone()))
            .app_data(actix_web::web::Data::new(assistant_service.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(
                scope("/api")
                    .service(records::routes::get_routes())
                    .service(ai::routes::get_routes()),
            );

        if let Some(path) = &frontend_path {
            // Single-port deployment: the API lives under /api (registered above) and
            // everything else is the SPA. Unknown paths fall back to index.html so
            // that deep links like /<incident-id> keep working after a refresh.
            let index_file = path.join("index.html");

            app = app.service(
                Files::new("/", path)
                    .index_file("index.html")
                    .default_handler(fn_service(move |req: ServiceRequest| {
                        let index_file = index_file.clone();

                        async move {
                            let (req, _payload) = req.into_parts();

                            match NamedFile::open(&index_file) {
                                // NamedFile derives text/html from the .html extension.
                                Ok(file) => {
                                    let res = file.into_response(&req);
                                    Ok(ServiceResponse::new(req, res))
                                }
                                Err(_) => Ok(ServiceResponse::new(
                                    req,
                                    HttpResponse::NotFound()
                                        .body("Frontend assets are not installed"),
                                )),
                            }
                        }
                    })),
            );
        }

        app
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::ai::service::assistant::AssistantService;
    use crate::ai::service::encoder::SentenceEncoderService;
    use crate::ai::service::weaviate::WeaviateService;
    use crate::records::service::RecordsService;
    use crate::setup_database;
    use chrono::Utc;
    use dotenvy::dotenv;
    use env_logger::Env;
    use orm::prelude::CrudService;
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::records::entities::RecordEntity;
    use crate::records::repository::PostgresRecordRepository;

    // Requires a running Postgres + Weaviate (see docker-compose.yml).
    #[tokio::test]
    async fn test_roundtrip() {
        match dotenv() {
            Ok(_) => println!("Loaded .env file"),
            Err(_) => println!("No .env file found"),
        };

        env_logger::init_from_env(Env::default().default_filter_or("info"));

        let db = setup_database().await.unwrap();
        let repository = PostgresRecordRepository::new(db);
        let records_service = RecordsService::new(repository);

        let record = RecordEntity {
            id: Uuid::from_str("5e9177dc-fd5b-4db5-b1e5-f108cd84a93c").unwrap(),
            title: "title".to_string(),
            body: "content".to_string(),
            encoded: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = records_service.create(record.clone()).await.unwrap();
        assert_eq!(created.id, record.id);

        let found = records_service.find(record.id).await.unwrap().unwrap();
        assert_eq!(found.title, "title");

        records_service.delete(record.id).await.unwrap();
        assert!(records_service.find(record.id).await.unwrap().is_none());
    }

    // Requires a running Postgres + Weaviate + OpenAI compatible endpoint.
    #[tokio::test]
    async fn test_querying() {
        match dotenv() {
            Ok(_) => println!("Loaded .env file"),
            Err(_) => println!("No .env file found"),
        };

        env_logger::init_from_env(Env::default().default_filter_or("info"));

        let db = setup_database().await.unwrap();
        let records_service = RecordsService::new(PostgresRecordRepository::new(db));

        let question = "What is the secret password?";

        let encoding_service = SentenceEncoderService::new();
        let weaviate_service = WeaviateService::new().await.unwrap();
        let assistant_service = AssistantService::new();

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
