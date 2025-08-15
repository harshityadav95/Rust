use std::sync::Arc;

use crate::error::AppError;
use crate::models::todo::{ListQuery, NewTodo, Todo, UpdateTodo};
use crate::repository::TodoRepository;

/// Service layer encapsulating business logic for Todos.
///
/// Keeps controllers thin and centralizes validations and cross-cutting rules.
/// Uses a repository implementing `TodoRepository` for persistence.
#[derive(Clone)]
pub struct TodoService<R: TodoRepository + 'static> {
    repo: Arc<R>,
}

impl<R: TodoRepository + 'static> TodoService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, new: NewTodo) -> Result<Todo, AppError> {
        // Example rule: trim whitespace on inputs
        let mut n = new;
        n.title = n.title.trim().to_string();
        if let Some(desc) = n.description.as_mut() {
            *desc = desc.trim().to_string();
        }
        self.repo.create(n).await
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Todo, AppError> {
        match self.repo.get_by_id(id).await? {
            Some(t) => Ok(t),
            None => Err(AppError::NotFound),
        }
    }

    pub async fn list(&self, q: ListQuery) -> Result<Vec<Todo>, AppError> {
        self.repo.list(q).await
    }

    pub async fn update(&self, id: i64, mut up: UpdateTodo) -> Result<Todo, AppError> {
        if let Some(title) = up.title.as_mut() {
            *title = title.trim().to_string();
        }
        if let Some(desc_opt) = up.description.as_mut() {
            if let Some(desc) = desc_opt.as_mut() {
                *desc = desc.trim().to_string();
            }
        }
        match self.repo.update(id, up).await? {
            Some(t) => Ok(t),
            None => Err(AppError::NotFound),
        }
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    // Simple in-memory mock repository for service tests
    struct MemRepo {
        inner: Mutex<Vec<Todo>>,
        next_id: Mutex<i64>,
    }

    impl MemRepo {
        fn new() -> Self {
            Self {
                inner: Mutex::new(vec![]),
                next_id: Mutex::new(1),
            }
        }
        fn now() -> OffsetDateTime {
            OffsetDateTime::now_utc()
        }
    }

    #[async_trait]
    impl TodoRepository for MemRepo {
        async fn create(&self, new: NewTodo) -> Result<Todo, AppError> {
            let mut id_guard = self.next_id.lock().unwrap();
            let id = *id_guard;
            *id_guard += 1;

            let now = Self::now();
            let todo = Todo {
                id,
                title: new.title,
                description: new.description,
                completed: false,
                due_date: new.due_date,
                created_at: now,
                updated_at: now,
            };
            self.inner.lock().unwrap().push(todo.clone());
            Ok(todo)
        }

        async fn get_by_id(&self, id: i64) -> Result<Option<Todo>, AppError> {
            Ok(self.inner.lock().unwrap().iter().cloned().find(|t| t.id == id))
        }

        async fn list(&self, _query: ListQuery) -> Result<Vec<Todo>, AppError> {
            Ok(self.inner.lock().unwrap().clone())
        }

        async fn update(&self, id: i64, update: UpdateTodo) -> Result<Option<Todo>, AppError> {
            let mut vec = self.inner.lock().unwrap();
            if let Some(t) = vec.iter_mut().find(|t| t.id == id) {
                if let Some(title) = update.title {
                    t.title = title;
                }
                if let Some(desc_opt) = update.description {
                    t.description = desc_opt;
                }
                if let Some(done) = update.completed {
                    t.completed = done;
                }
                if let Some(due_opt) = update.due_date {
                    t.due_date = due_opt;
                }
                t.updated_at = Self::now();
                return Ok(Some(t.clone()));
            }
            Ok(None)
        }

        async fn delete(&self, id: i64) -> Result<bool, AppError> {
            let mut vec = self.inner.lock().unwrap();
            let len_before = vec.len();
            vec.retain(|t| t.id != id);
            Ok(vec.len() != len_before)
        }
    }

    #[actix_rt::test]
    async fn service_create_trims_inputs() {
        let repo = Arc::new(MemRepo::new());
        let svc = TodoService::new(repo);

        let created = svc
            .create(NewTodo {
                title: "  Hello  ".into(),
                description: Some("  world  ".into()),
                due_date: None,
            })
            .await
            .unwrap();

        assert_eq!(created.title, "Hello");
        assert_eq!(created.description.as_deref(), Some("world"));
        assert!(!created.completed);
    }

    #[actix_rt::test]
    async fn service_get_update_delete_flow_and_not_found() {
        let repo = Arc::new(MemRepo::new());
        let svc = TodoService::new(repo);

        let created = svc
            .create(NewTodo {
                title: "Task".into(),
                description: None,
                due_date: None,
            })
            .await
            .unwrap();

        // get ok
        let got = svc.get_by_id(created.id).await.unwrap();
        assert_eq!(got.id, created.id);

        // get not found
        let e = svc.get_by_id(9999).await.unwrap_err();
        assert!(matches!(e, AppError::NotFound));

        // update ok
        let updated = svc
            .update(
                created.id,
                UpdateTodo {
                    title: Some("New".into()),
                    description: Some(None),
                    completed: Some(true),
                    due_date: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.title, "New");
        assert!(updated.description.is_none());
        assert!(updated.completed);

        // update not found
        let e = svc
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
            .unwrap_err();
        assert!(matches!(e, AppError::NotFound));

        // delete ok
        svc.delete(created.id).await.unwrap();

        // delete not found
        let e = svc.delete(created.id).await.unwrap_err();
        assert!(matches!(e, AppError::NotFound));
    }
}