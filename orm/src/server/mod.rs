use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::error::RepositoryError;

impl ResponseError for RepositoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            RepositoryError::ItemNotFoundError => StatusCode::NOT_FOUND,
            RepositoryError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RepositoryError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ActixAnyhowError {
    #[error("an unspecified internal error occurred: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl ResponseError for ActixAnyhowError {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Short hand alias, which allows you to use just `ActixAnyhow<T>`.
pub type ActixAnyhow<T> = std::result::Result<T, ActixAnyhowError>;
