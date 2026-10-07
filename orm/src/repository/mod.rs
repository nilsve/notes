use async_trait::async_trait;

use crate::db::Db;
use crate::error::RepositoryError;

/// A row backed entity.
pub trait Entity: Send + Sync + Sized + 'static {
    /// Primary key type.
    type Pk: Send + Sync + Clone;

    fn primary_key(&self) -> Self::Pk;
}

/// Minimal CRUD contract implemented by every Postgres backed repository.
#[async_trait]
pub trait CrudRepository<E: Entity>: Clone + Send + Sync + 'static {
    fn pool(&self) -> &Db;

    /// Inserts a new row. Fails when the primary key already exists.
    async fn create(&self, entity: E) -> Result<E, RepositoryError>;

    /// Inserts the row, replacing any existing row with the same primary key.
    async fn upsert(&self, entity: E) -> Result<E, RepositoryError>;

    async fn find(&self, id: E::Pk) -> Result<Option<E>, RepositoryError>;

    async fn get(&self, id: E::Pk) -> Result<E, RepositoryError> {
        self.find(id).await?.ok_or(RepositoryError::ItemNotFoundError)
    }

    async fn all(&self) -> Result<Vec<E>, RepositoryError>;

    /// Deletes by primary key, returning the number of removed rows.
    async fn delete(&self, id: E::Pk) -> Result<u64, RepositoryError>;

    async fn count(&self) -> Result<i64, RepositoryError>;
}
