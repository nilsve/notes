use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::prelude::DynamoRepositoryError;

impl ResponseError for DynamoRepositoryError {}


#[derive(thiserror::Error, Debug)]
pub enum ActixAnyhowError {
    #[error("an unspecified internal error occurred: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl ResponseError for ActixAnyhowError {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

// Short hand alias, which allows you to use just Result<T>
pub type ActixAnyhow<T> = std::result::Result<T, ActixAnyhowError>;