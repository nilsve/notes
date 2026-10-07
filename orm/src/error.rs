use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("item wasn't found during get operation")]
    ItemNotFoundError,

    #[error("invalid configuration: {0}")]
    Config(String),
}
