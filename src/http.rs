use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{error::{ApiError, ApiResult}, state::ApiState};

pub mod authorization;
mod middleware;
pub mod permissions;
mod token;
mod validation;

#[debug_handler]
pub async fn health_check(
    State(state): State<ApiState>
) -> ApiResult<()> {
    sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::ServiceUnavailable("unhealthy".to_string()))?;
    
    Ok(())
}

#[debug_handler]
pub async fn version() -> impl IntoResponse {
    Json(json!({ "api_version": "3.0" }))
}