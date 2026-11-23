use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

use crate::http::permissions::Permission;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApiError {
    // 400
    #[error("invalid request body")]
    BadRequest(String),

    // 404
    #[error("{0} not found")]
    NotFound(String),

    // 409
    #[error("entity already exists")]
    Conflict(String),

    // 401
    #[error("missing or invalid authorization header")]
    Unauthenticated,

    // 401
    #[error("Account is disabled")]
    AccountDisabled,

    // 401
    #[error("invalid login or password")]
    InvalidCredentials,

    // 401
    #[error("token expired")]
    TokenExpired,

    // 401
    #[error("invalid token")]
    InvalidToken,

    // 403
    #[error("missing permission: {0}")]
    Forbidden(Permission),

    // 400
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),

    // 500
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),

    // 503
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
