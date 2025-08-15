use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::info;

/// Create a new Sqlite connection pool.
pub async fn new_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

/// Run database migrations found under ./migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // The path is relative to the crate root where the binary is executed.
    // Ensure the "migrations" directory exists at project root.
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Database migrations ran successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn make_memory_pool() -> SqlitePool {
        new_pool("sqlite::memory:?cache=shared").await.unwrap()
    }

    #[actix_rt::test]
    async fn migrations_create_todos_table() {
        let pool = make_memory_pool().await;
        run_migrations(&pool).await.unwrap();

        // Check that the todos table exists
        let exists: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM sqlite_master
            WHERE type='table' AND name='todos'
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(exists.0, 1);
    }
}