use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::auth::service;
use crate::error::ApiError;
use crate::error::ApiResult;
use crate::http::authorization::AuthenticatedUserOrAdmin;
use crate::http::authorization::PublicAuthenticated;
use crate::state::ApiState;

use super::requests;
use super::responses;

// POST /auth/register
#[debug_handler]
pub async fn register(
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::Register>>,
) -> ApiResult<responses::Access> {
    service::register(&state, payload).await
}

// POST /auth/login
#[debug_handler]
pub async fn login(
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::Login>>,
) -> ApiResult<responses::Access> {
    service::login(&state, payload).await
}

// POST /auth/password-reset/request
#[debug_handler]
pub async fn reset_password_request(
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::PasswordReset>>,
) -> ApiResult<impl IntoResponse> {
    service::request_password_reset(&state, payload)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

// POST /auth/password-reset/confirm
#[debug_handler]
pub async fn reset_password_confirm(
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::PasswordResetConfirm>>,
) -> ApiResult<StatusCode> {
    service::confirm_password_reset(&state, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

// POST /auth/logout
#[debug_handler]
pub async fn logout(
    AuthenticatedUserOrAdmin(_current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    axum_extra::TypedHeader(authorization): axum_extra::TypedHeader<
        axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>,
    >,
) -> ApiResult<StatusCode> {
    service::logout(&state, authorization.token()).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /auth/me
#[debug_handler]
pub async fn get_me(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
) -> ApiResult<responses::Me> {
    // Возможно есть более правильный способ сделать это...
    let PublicAuthenticated::User { user_id, .. } = current_user else {
        return Err(ApiError::Unauthenticated);
    };

    service::me(&state, user_id).await
}
