use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;
use utoipa::ToSchema;

use crate::error::AppError;
use crate::models::todo::{ListQuery, NewTodo, Todo, UpdateTodo};
use crate::repository::TodoRepository;
use crate::services::todo_service::TodoService;

/// Shared application state injected into handlers.
#[derive(Clone)]
pub struct AppState<R: TodoRepository + 'static> {
    pub service: Arc<TodoService<R>>,
}

impl<R: TodoRepository + 'static> AppState<R> {
    pub fn new(service: Arc<TodoService<R>>) -> Self {
        Self { service }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/todos",
    request_body = NewTodo,
    responses(
        (status = 201, description = "Todo created", body = Todo),
        (status = 422, description = "Validation error"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Server error")
    )
)]
#[post("/todos")]
pub async fn create_todo<R: TodoRepository + 'static>(
    state: web::Data<AppState<R>>,
    payload: web::Json<NewTodo>,
) -> Result<impl Responder, AppError> {
    let created = state.service.create(payload.into_inner()).await?;
    Ok(HttpResponse::Created().json(created))
}

#[utoipa::path(
    get,
    path = "/api/v1/todos",
    params(ListQuery),
    responses(
        (status = 200, description = "List of todos", body = [Todo]),
        (status = 500, description = "Server error")
    )
)]
#[get("/todos")]
pub async fn list_todos<R: TodoRepository + 'static>(
    state: web::Data<AppState<R>>,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, AppError> {
    let items = state.service.list(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(items))
}

#[utoipa::path(
    get,
    path = "/api/v1/todos/{id}",
    params(
        ("id" = i64, Path, description = "Todo id")
    ),
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 404, description = "Todo not found")
    )
)]
#[get("/todos/{id}")]
pub async fn get_todo<R: TodoRepository + 'static>(
    state: web::Data<AppState<R>>,
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let todo = state.service.get_by_id(id).await?;
    Ok(HttpResponse::Ok().json(todo))
}

#[utoipa::path(
    put,
    path = "/api/v1/todos/{id}",
    request_body = UpdateTodo,
    params(
        ("id" = i64, Path, description = "Todo id")
    ),
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 404, description = "Todo not found"),
        (status = 422, description = "Validation error")
    )
)]
#[put("/todos/{id}")]
pub async fn update_todo<R: TodoRepository + 'static>(
    state: web::Data<AppState<R>>,
    path: web::Path<i64>,
    payload: web::Json<UpdateTodo>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let updated = state.service.update(id, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/todos/{id}",
    params(
        ("id" = i64, Path, description = "Todo id")
    ),
    responses(
        (status = 204, description = "Todo deleted"),
        (status = 404, description = "Todo not found")
    )
)]
#[delete("/todos/{id}")]
pub async fn delete_todo<R: TodoRepository + 'static>(
    state: web::Data<AppState<R>>,
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    state.service.delete(id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Configure todos scoped routes under /api/v1.
pub fn configure<R: TodoRepository + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create_todo::<R>)
            .service(list_todos::<R>)
            .service(get_todo::<R>)
            .service(update_todo::<R>)
            .service(delete_todo::<R>),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App};
    use sqlx::SqlitePool;

    use crate::db::run_migrations;
    use crate::models::todo::{ListQuery, NewTodo, UpdateTodo};
    use crate::repository::sqlite::SqliteTodoRepo;
    use crate::services::todo_service::TodoService;

    async fn test_state() -> web::Data<AppState<SqliteTodoRepo>> {
        let pool = SqlitePool::connect("sqlite::memory:?cache=shared").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = SqliteTodoRepo::new(pool);
        let service = Arc::new(TodoService::new(repo.into_arc()));
        web::Data::new(AppState::new(service))
    }

    #[actix_rt::test]
    async fn http_flow_works() {
        let state = test_state().await;

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::scope("/api/v1")
                    .service(super::super::health)
                    .configure(|cfg| super::configure::<SqliteTodoRepo>(cfg)),
            ),
        )
        .await;

        // Create
        let req = test::TestRequest::post()
            .uri("/api/v1/todos")
            .set_json(&NewTodo {
                title: "Task".into(),
                description: Some("desc".into()),
                due_date: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: Todo = test::read_body_json(resp).await;
        assert_eq!(created.title, "Task");

        // Get by id
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/todos/{}", created.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let got: Todo = test::read_body_json(resp).await;
        assert_eq!(got.id, created.id);

        // List
        let req = test::TestRequest::get().uri("/api/v1/todos").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: Vec<Todo> = test::read_body_json(resp).await;
        assert!(!list.is_empty());

        // Update
        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/todos/{}", created.id))
            .set_json(&UpdateTodo {
                title: Some("Updated".into()),
                description: Some(None),
                completed: Some(true),
                due_date: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let upd: Todo = test::read_body_json(resp).await;
        assert_eq!(upd.title, "Updated");
        assert!(upd.completed);

        // Delete
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/todos/{}", created.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Not found after delete
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/todos/{}", created.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}