use axum::Json;
use axum::extract::State;
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::auth::service;
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
    Valid(Json(payload)): Valid<Json<requests::Register>>
) -> ApiResult<responses::Access> {
    service::register(&state, payload).await
}

// POST /auth/login
#[debug_handler]
pub async fn login(
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::Login>>
) -> ApiResult<responses::Access> {
    service::login(&state, payload).await
}

// POST /auth/password-reset/request
#[debug_handler]
pub async fn reset_password_request(
    State(state): State<ApiState>,
    Valid(Json(body)): Valid<Json<requests::PasswordReset>>
) {
    todo!()
}

// POST /auth/password-reset/confirm
#[debug_handler]
pub async fn reset_password_confirm(
    State(state): State<ApiState>,
    Valid(Json(body)): Valid<Json<requests::PasswordResetConfirm>>,
) {
    todo!()
}

// POST /auth/logout
#[debug_handler]
pub async fn logout(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>
) {
    // хз надо чето делать
    todo!()
}

// GET /auth/me
#[debug_handler]
pub async fn get_me(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>
) -> ApiResult<responses::Me>
{
    let PublicAuthenticated::User { user_id, .. } = current_user else {
        unreachable!();
    };

    service::me(&state, user_id).await
}
