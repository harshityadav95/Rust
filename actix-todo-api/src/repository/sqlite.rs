use std::sync::Arc;

use sqlx::{QueryBuilder, SqlitePool};
use time::OffsetDateTime;

use crate::error::AppError;
use crate::models::todo::{ListQuery, NewTodo, Todo, UpdateTodo};
use crate::repository::{validate_new_todo, validate_update_todo, RepoResult, TodoRepository};

/// SQLite repository implementation for Todos backed by sqlx.
#[derive(Clone)]
pub struct SqliteTodoRepo {
    pool: SqlitePool,
}

impl SqliteTodoRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    async fn fetch_by_id(&self, id: i64) -> Result<Option<Todo>, sqlx::Error> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
            SELECT
                id,
                title,
                description,
                completed,
                due_date,
                created_at,
                updated_at
            FROM todos
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(todo)
    }
}

#[async_trait::async_trait]
impl TodoRepository for SqliteTodoRepo {
    async fn create(&self, new: NewTodo) -> RepoResult<Todo> {
        validate_new_todo(&new)?;

        let now = Self::now();

        // Insert new row
        sqlx::query(
            r#"
            INSERT INTO todos (title, description, completed, due_date, created_at, updated_at)
            VALUES (?, ?, 0, ?, ?, ?)
            "#,
        )
        .bind(new.title)
        .bind(new.description)
        .bind(new.due_date)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        // Fetch last inserted id
        let (id,) = sqlx::query_as::<_, (i64,)>("SELECT last_insert_rowid()")
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?;

        // Fetch and return full Todo
        let todo = self
            .fetch_by_id(id)
            .await
            .map_err(AppError::from)?
            .expect("inserted row must exist");
        Ok(todo)
    }

    async fn get_by_id(&self, id: i64) -> RepoResult<Option<Todo>> {
        let todo = self.fetch_by_id(id).await.map_err(AppError::from)?;
        Ok(todo)
    }

    async fn list(&self, query: ListQuery) -> RepoResult<Vec<Todo>> {
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                id,
                title,
                description,
                completed,
                due_date,
                created_at,
                updated_at
            FROM todos
            "#,
        );

        let mut has_where = false;
        if let Some(done) = query.completed {
            qb.push(" WHERE completed = ").push_bind(done);
            has_where = true;
        }

        if has_where {
            qb.push(" ");
        }

        qb.push(" ORDER BY created_at DESC ");
        qb.push(" LIMIT ").push_bind(query.limit_or_default());
        qb.push(" OFFSET ").push_bind(query.offset_or_default());

        let rows = qb
            .build_query_as::<Todo>()
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn update(&self, id: i64, update: UpdateTodo) -> RepoResult<Option<Todo>> {
        validate_update_todo(&update)?;

        let mut qb = QueryBuilder::new("UPDATE todos SET ");
        let mut sep = qb.separated(", ");

        let now = Self::now();
        sep.push("updated_at = ").push_bind(now);

        if let Some(title) = update.title {
            sep.push("title = ").push_bind(title);
        }
        if let Some(desc_opt) = update.description {
            match desc_opt {
                Some(desc) => {
                    sep.push("description = ").push_bind(desc);
                }
                None => {
                    sep.push("description = NULL");
                }
            }
        }
        if let Some(done) = update.completed {
            sep.push("completed = ").push_bind(done);
        }
        if let Some(due_opt) = update.due_date {
            match due_opt {
                Some(due) => sep.push("due_date = ").push_bind(due),
                None => sep.push("due_date = NULL"),
            };
        }

        qb.push(" WHERE id = ").push_bind(id);

        let result = qb.build().execute(&self.pool).await.map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        let todo = self.fetch_by_id(id).await.map_err(AppError::from)?;
        Ok(todo)
    }

    async fn delete(&self, id: i64) -> RepoResult<bool> {
        let res = sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(res.rows_affected() > 0)
    }
}

impl From<SqlitePool> for SqliteTodoRepo {
    fn from(pool: SqlitePool) -> Self {
        Self::new(pool)
    }
}

impl SqliteTodoRepo {
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:?cache=shared").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[actix_rt::test]
    async fn crud_flow() {
        let repo = SqliteTodoRepo::new(test_pool().await);

        // Create
        let created = repo
            .create(NewTodo {
                title: "Task".into(),
                description: Some("desc".into()),
                due_date: None,
            })
            .await
            .unwrap();
        assert_eq!(created.title, "Task");
        assert!(!created.completed);
        assert!(created.id > 0);

        // Get by id
        let fetched = repo.get_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, created.id);

        // List
        let list = repo
            .list(ListQuery {
                limit: Some(10),
                offset: Some(0),
                completed: Some(false),
            })
            .await
            .unwrap();
        assert!(!list.is_empty());

        // Update title and mark completed
        let updated = repo
            .update(
                created.id,
                UpdateTodo {
                    title: Some("Updated".into()),
                    description: Some(None), // clear
                    completed: Some(true),
                    due_date: None,
                },
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.title, "Updated");
        assert!(updated.description.is_none());
        assert!(updated.completed);

        // Delete
        let deleted = repo.delete(created.id).await.unwrap();
        assert!(deleted);

        // Ensure not found after delete
        let gone = repo.get_by_id(created.id).await.unwrap();
        assert!(gone.is_none());
    }

    #[actix_rt::test]
    async fn update_nonexistent_returns_none() {
        let repo = SqliteTodoRepo::new(test_pool().await);
        let res = repo
            .update(
                9999,
                UpdateTodo {
                    title: Some("x".into()),
                    description: None,
                    completed: None,
                    due_date: None,
                },
            )
            .await
            .unwrap();
        assert!(res.is_none());
    }
}