use async_trait::async_trait;
use uuid::Uuid;

use orm::prelude::{CrudRepository, Db, RepositoryError};

use crate::records::entities::RecordEntity;

// The table name is a literal (not interpolated at runtime) so every query stays a
// safe, prepared statement.
const INSERT_SQL: &str =
    "INSERT INTO records (id, title, body, encoded) VALUES ($1, $2, $3, $4) RETURNING *";

const UPSERT_SQL: &str = "INSERT INTO records (id, title, body, encoded) \
    VALUES ($1, $2, $3, $4) \
    ON CONFLICT (id) DO UPDATE \
      SET title = EXCLUDED.title, \
          body = EXCLUDED.body, \
          encoded = EXCLUDED.encoded, \
          updated_at = now() \
    RETURNING *";

const FIND_SQL: &str = "SELECT * FROM records WHERE id = $1";

const ALL_SQL: &str = "SELECT * FROM records ORDER BY created_at DESC";

const DELETE_SQL: &str = "DELETE FROM records WHERE id = $1";

const COUNT_SQL: &str = "SELECT COUNT(*) FROM records";

/// Postgres backed repository for [`RecordEntity`].
#[derive(Clone)]
pub struct PostgresRecordRepository {
    pool: Db,
}

impl PostgresRecordRepository {
    pub fn new(pool: Db) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CrudRepository<RecordEntity> for PostgresRecordRepository {
    fn pool(&self) -> &Db {
        &self.pool
    }

    async fn create(&self, record: RecordEntity) -> Result<RecordEntity, RepositoryError> {
        let created = sqlx::query_as::<_, RecordEntity>(INSERT_SQL)
            .bind(record.id)
            .bind(record.title)
            .bind(record.body)
            .bind(record.encoded.map(sqlx::types::Json))
            .fetch_one(&self.pool)
            .await?;

        Ok(created)
    }

    async fn upsert(&self, record: RecordEntity) -> Result<RecordEntity, RepositoryError> {
        let saved = sqlx::query_as::<_, RecordEntity>(UPSERT_SQL)
            .bind(record.id)
            .bind(record.title)
            .bind(record.body)
            .bind(record.encoded.map(sqlx::types::Json))
            .fetch_one(&self.pool)
            .await?;

        Ok(saved)
    }

    async fn find(&self, id: Uuid) -> Result<Option<RecordEntity>, RepositoryError> {
        Ok(sqlx::query_as::<_, RecordEntity>(FIND_SQL)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn all(&self) -> Result<Vec<RecordEntity>, RepositoryError> {
        Ok(sqlx::query_as::<_, RecordEntity>(ALL_SQL)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn delete(&self, id: Uuid) -> Result<u64, RepositoryError> {
        let result = sqlx::query(DELETE_SQL).bind(id).execute(&self.pool).await?;

        Ok(result.rows_affected())
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        Ok(sqlx::query_scalar::<_, i64>(COUNT_SQL)
            .fetch_one(&self.pool)
            .await?)
    }
}
