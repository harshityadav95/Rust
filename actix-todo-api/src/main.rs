mod config;
mod db;
mod error;
mod models;
mod repository;
mod routes;
mod services;

use std::net::Ipv4Addr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use repository::sqlite::SqliteTodoRepo;
use services::todo_service::TodoService;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::AppConfig;
use crate::db::{new_pool, run_migrations};
use crate::routes::todos::AppState;

// OpenAPI (utoipa) setup
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health,
        routes::todos::create_todo,
        routes::todos::list_todos,
        routes::todos::get_todo,
        routes::todos::update_todo,
        routes::todos::delete_todo,
    ),
    components(
        schemas(
            models::todo::Todo,
            models::todo::NewTodo,
            models::todo::UpdateTodo,
            models::todo::ListQuery
        )
    ),
    tags(
        (name = "todos", description = "Todo management endpoints"),
        (name = "health", description = "Health check")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging/tracing
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,actix_web=info")) // default if RUST_LOG not set
        .unwrap();
    fmt().with_env_filter(env_filter).init();

    // Load configuration
    let cfg = AppConfig::from_env();
    info!("Starting service on port {} with database {}", cfg.port, cfg.database_url);

    // Database pool and migrations
    let pool = new_pool(&cfg.database_url)
        .await
        .expect("failed to create database pool");
    run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    // Repository and service wiring
    let repo = SqliteTodoRepo::new(pool);
    let service = Arc::new(TodoService::new(repo.into_arc()));
    let state = web::Data::new(AppState::new(service));

    // Build OpenAPI doc once
    let openapi = ApiDoc::openapi();

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(state.clone())
            .configure(routes::configure::<SqliteTodoRepo>)
            // Serve Swagger UI at /docs with OpenAPI JSON at /api-docs/openapi.json
            .service(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi.clone()))
    })
    .bind((Ipv4Addr::UNSPECIFIED, cfg.port))?
    .run()
    .await
}