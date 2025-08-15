use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

/// Core Todo model as stored/returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub due_date: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Payload for creating a new Todo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct NewTodo {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<OffsetDateTime>,
}

/// Payload for updating an existing Todo.
/// - Fields are optional to allow partial updates.
/// - For nullable fields (description, due_date) we use Option<Option<T>>
///   - None => do not change
///   - Some(None) => set to NULL
///   - Some(Some(v)) => update to v
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub completed: Option<bool>,
    pub due_date: Option<Option<OffsetDateTime>>,
}

/// Query params for listing Todos with pagination and filtering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ListQuery {
    /// Max number of items to return; default 50; max 200.
    pub limit: Option<i64>,
    /// Number of items to skip; default 0.
    pub offset: Option<i64>,
    /// Optional filter by completion state.
    pub completed: Option<bool>,
}

impl ListQuery {
    pub fn limit_or_default(&self) -> i64 {
        let lim = self.limit.unwrap_or(50);
        lim.min(200).max(1)
    }
    pub fn offset_or_default(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use time::macros::datetime;

    #[test]
    fn new_todo_serialization_roundtrip() {
        let due = Some(datetime!(2025-01-02 03:04:05 UTC));
        let new = NewTodo {
            title: "Test".into(),
            description: Some("Desc".into()),
            due_date: due,
        };
        let s = serde_json::to_string(&new).unwrap();
        let back: NewTodo = serde_json::from_str(&s).unwrap();
        assert_eq!(new, back);
    }

    #[test]
    fn todo_serializes_dates_as_rfc3339() {
        let dt = datetime!(2025-01-02 03:04:05 UTC);
        let todo = Todo {
            id: 1,
            title: "A".into(),
            description: None,
            completed: false,
            due_date: Some(dt),
            created_at: dt,
            updated_at: dt,
        };
        let v: Value = serde_json::to_value(todo).unwrap();
        let due = v.get("due_date").and_then(|x| x.as_str()).unwrap();
        assert!(due.contains("2025-01-02T03:04:05Z"));

        let created = v.get("created_at").and_then(|x| x.as_str()).unwrap();
        assert!(created.contains("2025-01-02T03:04:05Z"));
    }

    #[test]
    fn list_query_defaults_are_enforced() {
        let q = ListQuery { limit: None, offset: None, completed: None };
        assert_eq!(q.limit_or_default(), 50);
        assert_eq!(q.offset_or_default(), 0);

        let q = ListQuery { limit: Some(500), offset: Some(-10), completed: None };
        assert_eq!(q.limit_or_default(), 200);
        assert_eq!(q.offset_or_default(), 0);

        let q = ListQuery { limit: Some(0), offset: Some(5), completed: None };
        assert_eq!(q.limit_or_default(), 1);
        assert_eq!(q.offset_or_default(), 5);
    }

    #[test]
    fn update_todo_nullable_fields() {
        let up1 = UpdateTodo {
            title: None,
            description: None,
            completed: None,
            due_date: None,
        };
        let s = serde_json::to_string(&up1).unwrap();
        let back: UpdateTodo = serde_json::from_str(&s).unwrap();
        assert_eq!(up1, back);

        // Some(None) should survive roundtrip to signal clearing a field
        let up2 = UpdateTodo {
            title: Some("New".into()),
            description: Some(None),
            completed: Some(true),
            due_date: Some(None),
        };
        let s = serde_json::to_string(&up2).unwrap();
        let back: UpdateTodo = serde_json::from_str(&s).unwrap();
        assert_eq!(up2, back);
    }

    #[test]
    fn new_todo_json_shape() {
        let due = "2025-01-02T03:04:05Z";
        let v = json!({
            "title": "Task",
            "description": "D",
            "due_date": due
        });
        let nt: NewTodo = serde_json::from_value(v).unwrap();
        assert_eq!(nt.title, "Task");
        assert_eq!(nt.description.as_deref(), Some("D"));
        assert!(nt.due_date.is_some());
    }
}