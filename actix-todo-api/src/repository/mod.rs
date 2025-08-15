pub mod sqlite;
use crate::error::AppError;
use crate::models::todo::{ListQuery, NewTodo, Todo, UpdateTodo};
use async_trait::async_trait;

/// Common result alias for repository operations.
pub type RepoResult<T> = Result<T, AppError>;

/// Trait describing CRUD operations for Todos.
#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn create(&self, new: NewTodo) -> RepoResult<Todo>;
    async fn get_by_id(&self, id: i64) -> RepoResult<Option<Todo>>;
    async fn list(&self, query: ListQuery) -> RepoResult<Vec<Todo>>;
    async fn update(&self, id: i64, update: UpdateTodo) -> RepoResult<Option<Todo>>;
    async fn delete(&self, id: i64) -> RepoResult<bool>;
}

/// Basic validation for payloads used by service/repository layers.
pub fn validate_new_todo(new: &NewTodo) -> RepoResult<()> {
    let title_len = new.title.trim().chars().count();
    if title_len == 0 || title_len > 200 {
        return Err(AppError::Validation("title must be 1..=200 characters".into()));
    }
    if let Some(desc) = &new.description {
        if desc.chars().count() > 2000 {
            return Err(AppError::Validation("description must be ≤ 2000 characters".into()));
        }
    }
    Ok(())
}

pub fn validate_update_todo(up: &UpdateTodo) -> RepoResult<()> {
    if let Some(title) = &up.title {
        let title_len = title.trim().chars().count();
        if title_len == 0 || title_len > 200 {
            return Err(AppError::Validation("title must be 1..=200 characters".into()));
        }
    }
    if let Some(opt) = &up.description {
        if let Some(desc) = opt {
            if desc.chars().count() > 2000 {
                return Err(AppError::Validation("description must be ≤ 2000 characters".into()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn validate_new_ok() {
        let new = NewTodo {
            title: "abc".into(),
            description: Some("desc".into()),
            due_date: Some(datetime!(2025-01-02 03:04:05 UTC)),
        };
        assert!(validate_new_todo(&new).is_ok());
    }

    #[test]
    fn validate_new_title_len() {
        let new = NewTodo {
            title: "".into(),
            description: None,
            due_date: None,
        };
        let e = validate_new_todo(&new).unwrap_err();
        assert!(format!("{e}").contains("title must be 1..=200"));
    }

    #[test]
    fn validate_new_desc_len() {
        let new = NewTodo {
            title: "ok".into(),
            description: Some("x".repeat(2001)),
            due_date: None,
        };
        let e = validate_new_todo(&new).unwrap_err();
        assert!(format!("{e}").contains("description"));
    }

    #[test]
    fn validate_update_ok() {
        let up = UpdateTodo {
            title: Some("New".into()),
            description: Some(Some("short".into())),
            completed: Some(true),
            due_date: Some(None),
        };
        assert!(validate_update_todo(&up).is_ok());
    }

    #[test]
    fn validate_update_title_bad() {
        let up = UpdateTodo {
            title: Some("".into()),
            description: None,
            completed: None,
            due_date: None,
        };
        let e = validate_update_todo(&up).unwrap_err();
        assert!(format!("{e}").contains("title must be 1..=200"));
    }

    #[test]
    fn validate_update_desc_too_long() {
        let up = UpdateTodo {
            title: None,
            description: Some(Some("x".repeat(2001))),
            completed: None,
            due_date: None,
        };
        let e = validate_update_todo(&up).unwrap_err();
        assert!(format!("{e}").contains("description"));
    }
}