use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::ApiResult,
    http::{
        authorization::{Authenticated, AuthenticatedUser},
        permissions::Permission,
    },
    state::ApiState,
};

use super::{repository::UserAccess, requests, responses, service};

// GET /users/me
#[debug_handler]
pub async fn get_me(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
) -> ApiResult<responses::UserProfile> {
    current_user.require(Permission::UsersMeManage)?;
    let user_id = current_user.user_id;
    service::get_profile(&state, user_id, UserAccess::Only { user_id }).await
}

// PUT /users/me
#[debug_handler]
pub async fn update_me(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::UpdateProfile>>,
) -> ApiResult<responses::User> {
    current_user.require(Permission::UsersMeManage)?;
    let user_id = current_user.user_id;
    let update = requests::UpdateUser {
        full_name: input.full_name,
        phone: input.phone,
        ..Default::default()
    };
    service::update_user(&state, user_id, update, UserAccess::Only { user_id }).await
}

// PUT /users/me/password
#[debug_handler]
pub async fn change_my_password(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::ChangePassword>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::UsersMeManage)?;
    service::change_password(&state, current_user.user_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /users
#[debug_handler]
pub async fn list_users(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListUsers>>,
) -> ApiResult<responses::ListUsers> {
    current_user.require(Permission::AdminUsersView)?;
    service::list_users(&state, input, UserAccess::All).await
}

// GET /users/{user_id}
#[debug_handler]
pub async fn get_user(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(path)): Valid<Path<requests::UserId>>,
) -> ApiResult<responses::UserProfile> {
    current_user.require(Permission::AdminUsersView)?;
    service::get_profile(&state, path.user_id, UserAccess::All).await
}

// POST /users
#[debug_handler]
pub async fn create_user(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateUser>>,
) -> ApiResult<(StatusCode, responses::CreatedUser)> {
    current_user.require(Permission::AdminUsersManage)?;
    service::create_user(&state, input)
        .await
        .map(|user| (StatusCode::CREATED, user))
}

// PUT /users/{user_id}
#[debug_handler]
pub async fn update_user(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(path)): Valid<Path<requests::UserId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateUser>>,
) -> ApiResult<responses::User> {
    current_user.require(Permission::AdminUsersManage)?;
    service::update_user(&state, path.user_id, input, UserAccess::All).await
}

// DELETE /users/{user_id}
#[debug_handler]
pub async fn deactivate_user(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(path)): Valid<Path<requests::UserId>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::AdminUsersManage)?;
    service::deactivate_user(&state, path.user_id, UserAccess::All).await?;
    Ok(StatusCode::NO_CONTENT)
}
