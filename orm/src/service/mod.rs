use crate::error::RepositoryError;
use crate::repository::{CrudRepository, Entity};

/// Thin delegation layer so services can be written against any repository.
#[async_trait::async_trait]
pub trait CrudService<E: Entity, R: CrudRepository<E>> {
    fn get_repository(&self) -> &R;

    async fn create(&self, entity: E) -> Result<E, RepositoryError> {
        self.get_repository().create(entity).await
    }

    async fn upsert(&self, entity: E) -> Result<E, RepositoryError> {
        self.get_repository().upsert(entity).await
    }

    async fn find(&self, id: E::Pk) -> Result<Option<E>, RepositoryError> {
        self.get_repository().find(id).await
    }

    async fn get(&self, id: E::Pk) -> Result<E, RepositoryError> {
        self.get_repository().get(id).await
    }

    async fn all(&self) -> Result<Vec<E>, RepositoryError> {
        self.get_repository().all().await
    }

    async fn delete(&self, id: E::Pk) -> Result<u64, RepositoryError> {
        self.get_repository().delete(id).await
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        self.get_repository().count().await
    }
}
