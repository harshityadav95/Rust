# Actix Todo API

Todo CRUD REST API built with Actix Web, SQLite (via SQLx), and documented using OpenAPI (utoipa) with Swagger UI.

Features
- Actix Web 4 HTTP server with CORS and request logging
- CRUD endpoints for Todos with pagination and filtering
- SQLite persistence using SQLx with migrations
- Layered architecture: Routes -> Service -> Repository -> DB
- OpenAPI 3 schema via utoipa and Swagger UI at /docs
- Unit and integration tests targeting 90%+ coverage using cargo-llvm-cov
- Dockerfile and Makefile for reproducible builds and developer ergonomics

Tech stack
- actix-web, actix-cors
- sqlx (SQLite, time)
- utoipa + utoipa-swagger-ui
- serde, time, tracing
- tokio
- thiserror, anyhow
- cargo-llvm-cov for coverage

Project structure
- src/
  - config.rs: Environment configuration loader (PORT, DATABASE_URL)
  - error.rs: Unified AppError mapped to HTTP responses
  - db/mod.rs: Sqlite pool creation and migration runner
  - models/
    - mod.rs: Re-exports
    - todo.rs: Todo domain model and DTOs (NewTodo, UpdateTodo, ListQuery)
  - repository/
    - mod.rs: Repository trait + validators
    - sqlite.rs: SQLx-based SQLite implementation
  - services/
    - todo_service.rs: Business logic layer
  - routes/
    - mod.rs: /api/v1 scope and health endpoint
    - todos.rs: CRUD handlers and AppState wiring
  - main.rs: App wiring, middleware, Swagger UI, server startup
- migrations/
  - 0001_create_todos.sql: Database schema and indexes
- Makefile: Common dev commands
- Dockerfile, .dockerignore
- Cargo.toml

Prerequisites
- Rust stable (1.75+ recommended)
- SQLite (runtime not required in container; uses bundled lib via libsqlite3-sys)
- For coverage: cargo-llvm-cov installed

Quick start (local)
1) Install dependencies
- Rust toolchain (rustup)
- Optionally cargo-llvm-cov for coverage: cargo install cargo-llvm-cov

2) Run migrations and start the service
- Default env:
  - PORT=8080
  - DATABASE_URL=sqlite:data/todos.db
- Commands:
  - make run
    - or: RUST_LOG=info PORT=8080 DATABASE_URL=sqlite:data/todos.db cargo run

3) Open API docs
- Navigate to http://localhost:8080/docs for Swagger UI
- OpenAPI JSON is served at /api-docs/openapi.json

Configuration
- Environment variables:
  - PORT: u16, default 8080
  - DATABASE_URL: SQLite URL, default sqlite:data/todos.db
- .env (optional) is loaded if present
- See src/config.rs for details

Database and migrations
- SQLite database file defaults to data/todos.db (created at runtime)
- SQLx migrations are located under ./migrations
- Migrations are applied on startup via sqlx::migrate!("./migrations")
- To use in-memory DB for testing: DATABASE_URL=sqlite::memory:?cache=shared

API endpoints
Base path: /api/v1
- Health
  - GET /health -> "OK"
- Todos
  - POST /api/v1/todos
    - Request: { "title": "Task", "description": "optional", "due_date": "RFC3339 optional" }
    - Response 201: Todo
  - GET /api/v1/todos
    - Query: limit (default 50, max 200), offset (default 0), completed (optional bool)
    - Response 200: [Todo]
  - GET /api/v1/todos/{id}
    - Response 200: Todo
    - Response 404: not found
  - PUT /api/v1/todos/{id}
    - Request: partial UpdateTodo
      - title: Option<String>
      - description: Option<Option<String>> (Some(None) clears field)
      - completed: Option<bool>
      - due_date: Option<Option<OffsetDateTime>> (Some(None) clears field)
    - Response 200: Todo
    - Response 404: not found
    - Response 422: validation error
  - DELETE /api/v1/todos/{id}
    - Response 204: no content
    - Response 404: not found

Model
- Todo
  - id: i64
  - title: String (1..=200)
  - description: Option<String> (<=2000)
  - completed: bool (default false)
  - due_date: Option<OffsetDateTime>
  - created_at: OffsetDateTime
  - updated_at: OffsetDateTime

Testing and coverage
- Unit tests cover:
  - config parsing
  - error mapping
  - db migrations
  - models serialization/validation
  - repository validators and SQLite CRUD
  - service logic including error cases
  - health and HTTP flow via actix_web::test
- Commands:
  - make test
  - make coverage
    - Generates an HTML report and lcov output
    - Example exclusions applied for minimal boilerplate (main.rs)
- Goal: 90%+ coverage
  - The repository, services, routes, and model tests are designed to cover success and error paths
  - Integration tests exercise HTTP paths, including 404/422 flows

Run with Docker
- Build:
  - make docker-build
  - or: docker build -t actix-todo-api:latest .
- Run:
  - make docker-run
  - or: docker run --rm -p 8080:8080 -e PORT=8080 -e DATABASE_URL=sqlite:data/todos.db actix-todo-api:latest
- Notes:
  - Image uses non-root user
  - Database file will be inside container at /app/data unless volume-mounted

Development tips
- Formatting: make fmt
- Linting: make clippy
- Logs: controlled by RUST_LOG (e.g., RUST_LOG=debug,actix_web=info)
- Use sqlite::memory:?cache=shared for ephemeral DB in tests or local experiments

Troubleshooting
- Binding error on port:
  - Change PORT environment variable (e.g., PORT=3000 make run)
- SQLite locked errors during development:
  - Ensure only one running process uses the on-disk DB file, or switch to in-memory DB for testing
- OpenAPI/Swagger UI not loading:
  - Verify service is running and visit http://localhost:8080/docs
- SQLx build errors about time types:
  - Ensure features are enabled: sqlx features include "sqlite" and "time"; time crate includes "serde"

License
- Dual-licensed under MIT or Apache-2.0