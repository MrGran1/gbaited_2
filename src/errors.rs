use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Hashing error")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("Token error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(m)     => (StatusCode::NOT_FOUND,             m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED,          m.clone()),
            AppError::BadRequest(m)   => (StatusCode::BAD_REQUEST,           m.clone()),
            AppError::Conflict(m)     => (StatusCode::CONFLICT,              m.clone()),
            AppError::Database(e) => {
                tracing::error!("DB error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            AppError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Hashing error".into()),
            AppError::Jwt(_)    => (StatusCode::UNAUTHORIZED, "Invalid or expired token".into()),
            AppError::Internal(m) => {
                tracing::error!("Internal: {}", m);
                (StatusCode::INTERNAL_SERVER_ERROR, m.clone())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
