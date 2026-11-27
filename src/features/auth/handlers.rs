use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::error::ApiResult;
use crate::features::auth::service;
use crate::http::{
    authorization::{Authenticated, AuthenticatedUser},
    permissions::Permission,
};
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
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    axum_extra::TypedHeader(authorization): axum_extra::TypedHeader<
        axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>,
    >,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::UsersMeManage)?;
    service::logout(&state, authorization.token()).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /auth/me
#[debug_handler]
pub async fn get_me(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
) -> ApiResult<responses::Me> {
    current_user.require(Permission::UsersMeManage)?;
    service::me(&state, current_user.user_id).await
}
