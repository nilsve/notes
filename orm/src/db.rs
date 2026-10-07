use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::error::RepositoryError;

pub type Db = PgPool;

/// Connects to Postgres using `DATABASE_URL`
/// (e.g. `postgres://incidents:incidents@postgres:5432/incidents`).
pub async fn connect(url: &str) -> Result<Db, RepositoryError> {
    if url.trim().is_empty() {
        return Err(RepositoryError::Config(
            "DATABASE_URL is not set; cannot connect to Postgres".to_string(),
        ));
    }

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(url)
        .await?;

    Ok(pool)
}

/// Runs the given DDL statements in order. Idempotent statements are expected, so
/// the app can call this on every boot.
pub async fn migrate(db: &Db, statements: &[&str]) -> Result<(), RepositoryError> {
    for statement in statements {
        // DDL is dynamic by nature (migrations), so it runs as raw SQL instead of
        // a prepared statement. Statements here are authored in code, not derived
        // from user input.
        sqlx::raw_sql(sqlx::AssertSqlSafe(statement.to_string()))
            .execute(db)
            .await?;
    }

    Ok(())
}
