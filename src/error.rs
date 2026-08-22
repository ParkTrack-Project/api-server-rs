use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

use crate::http::permissions::Permission;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApiError {
    #[error("invalid request body")]
    BadRequest(String),

    #[error("{0} not found")]
    NotFound(String),

    #[error("entity already exists")]
    Conflict(String),

    #[error("missing or invalid authorization header")]
    Unauthenticated,

    #[error("Account is disabled")]
    AccountDisabled,

    #[error("invalid login or password")]
    InvalidCredentials,

    #[error("token expired")]
    TokenExpired,

    #[error("invalid token")]
    InvalidToken,

    #[error("missing permission: {0}")]
    Forbidden(Permission),

    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),

    #[error(transparent)]
    InternalError(#[from] anyhow::Error),

    #[error("{0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unauthenticated
            | ApiError::AccountDisabled
            | ApiError::InvalidCredentials
            | ApiError::TokenExpired
            | ApiError::InvalidToken => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };

        let body = Json(json!({
            "error": status.canonical_reason().unwrap_or("error"),
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

// Common case anyhow error aliases
pub const BEGIN_TRANSACTION_ERROR: &str = "Unable to begin transaction";