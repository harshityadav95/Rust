use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_users,
        get_user,
        create_user,
        update_user,
        delete_user,
    ),
    components(
        schemas(User, CreateUserRequest, UpdateUserRequest)
    ),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize, ToSchema)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

type UserStore = Mutex<HashMap<u64, User>>;

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List all users", body = Vec<User>)
    )
)]
async fn get_users(data: web::Data<UserStore>) -> Result<HttpResponse> {
    let users = data.lock().unwrap();
    let users_vec: Vec<User> = users.values().cloned().collect();
    Ok(HttpResponse::Ok().json(users_vec))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn get_user(path: web::Path<u64>, data: web::Data<UserStore>) -> Result<HttpResponse> {
    let id = path.into_inner();
    let users = data.lock().unwrap();
    if let Some(user) = users.get(&id) {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::NotFound().body("User not found"))
    }
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Invalid input")
    )
)]
async fn create_user(req: web::Json<CreateUserRequest>, data: web::Data<UserStore>) -> Result<HttpResponse> {
    let mut users = data.lock().unwrap();
    let id = users.keys().max().unwrap_or(&0) + 1;
    let user = User {
        id,
        name: req.name.clone(),
        email: req.email.clone(),
    };
    users.insert(id, user.clone());
    Ok(HttpResponse::Created().json(user))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn update_user(path: web::Path<u64>, req: web::Json<UpdateUserRequest>, data: web::Data<UserStore>) -> Result<HttpResponse> {
    let id = path.into_inner();
    let mut users = data.lock().unwrap();
    if let Some(user) = users.get_mut(&id) {
        if let Some(name) = &req.name {
            user.name = name.clone();
        }
        if let Some(email) = &req.email {
            user.email = email.clone();
        }
        Ok(HttpResponse::Ok().json(user.clone()))
    } else {
        Ok(HttpResponse::NotFound().body("User not found"))
    }
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found")
    )
)]
async fn delete_user(path: web::Path<u64>, data: web::Data<UserStore>) -> Result<HttpResponse> {
    let id = path.into_inner();
    let mut users = data.lock().unwrap();
    if users.remove(&id).is_some() {
        Ok(HttpResponse::Ok().body("User deleted"))
    } else {
        Ok(HttpResponse::NotFound().body("User not found"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let user_store = web::Data::new(UserStore::new(HashMap::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(user_store.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{id}", web::put().to(update_user))
            .route("/users/{id}", web::delete().to(delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}