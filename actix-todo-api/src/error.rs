use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

/// Standard API error type with mapping to HTTP responses.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("internal server error")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // Log server-side errors with stack/chain context if any
        match self {
            AppError::Db(e) => error!("Database error: {e:?}"),
            AppError::Internal(e) => error!("Internal error: {e:?}"),
            _ => {}
        }

        let status = self.status_code();
        let body = ErrorBody {
            code: status.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(status).json(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use actix_web::HttpResponse;

    async fn response_json(err: AppError) -> (StatusCode, String) {
        let resp: HttpResponse = err.error_response();
        let status = resp.status();
        let body = to_bytes(resp.into_body()).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[actix_rt::test]
    async fn not_found_maps_correctly() {
        let (status, body) = response_json(AppError::NotFound).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("resource not found"));
    }

    #[actix_rt::test]
    async fn validation_maps_correctly() {
        let (status, body) = response_json(AppError::Validation("title is required".into())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("validation error"));
        assert!(body.contains("title is required"));
    }

    #[actix_rt::test]
    async fn conflict_maps_correctly() {
        let (status, body) = response_json(AppError::Conflict("duplicate".into())).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(body.contains("conflict"));
    }

    #[actix_rt::test]
    async fn bad_request_maps_correctly() {
        let (status, body) = response_json(AppError::BadRequest("bad".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("bad request"));
    }

    #[actix_rt::test]
    async fn db_error_maps_to_500() {
        // fabricate a sqlx error via a simple Decode error string
        let e = sqlx::Error::Protocol("boom".into());
        let (status, _body) = response_json(AppError::Db(e)).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_rt::test]
    async fn internal_error_maps_to_500() {
        let e = anyhow::anyhow!("oops");
        let (status, body) = response_json(AppError::Internal(e)).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("internal server error"));
    }
}