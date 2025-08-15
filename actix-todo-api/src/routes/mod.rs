use actix_web::{get, web, HttpResponse, Responder};
use crate::repository::TodoRepository;

pub mod todos;

/// Simple health check endpoint.
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

/// Configure application routes under /api/v1, parameterized by repository type.
pub fn configure<R: TodoRepository + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health)
            .configure(todos::configure::<R>),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App};

    #[actix_rt::test]
    async fn health_ok() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}