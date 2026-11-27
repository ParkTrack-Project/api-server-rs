use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::{ApiError, ApiResult},
    http::{
        authorization::{Authenticated, AuthenticatedUserOrAdmin, PublicAuthenticated},
        permissions::Permission,
    },
    state::ApiState,
};

use super::{repository::UserAccess, requests, responses, service};

fn user_id(current_user: &PublicAuthenticated) -> ApiResult<i32> {
    match current_user {
        PublicAuthenticated::User { user_id, .. } => Ok(*user_id),
        PublicAuthenticated::ApiToken => Err(ApiError::Unauthenticated),
    }
}

#[debug_handler]
pub async fn get_me(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
) -> ApiResult<responses::UserProfile> {
    current_user.require(Permission::UsersMeManage)?;
    let user_id = user_id(&current_user)?;
    service::get_profile(&state, user_id, UserAccess::Only { user_id }).await
}

#[debug_handler]
pub async fn update_me(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::UpdateProfile>>,
) -> ApiResult<responses::User> {
    current_user.require(Permission::UsersMeManage)?;
    let user_id = user_id(&current_user)?;
    let update = requests::UpdateUser {
        full_name: input.full_name,
        phone: input.phone,
        ..Default::default()
    };
    service::update_user(&state, user_id, update, UserAccess::Only { user_id }).await
}

#[debug_handler]
pub async fn change_my_password(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::ChangePassword>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::UsersMeManage)?;
    service::change_password(&state, user_id(&current_user)?, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn list_users(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListUsers>>,
) -> ApiResult<responses::ListUsers> {
    current_user.require(Permission::AdminUsersView)?;
    service::list_users(&state, input, UserAccess::All).await
}

#[debug_handler]
pub async fn get_user(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Path(path)): Valid<Path<requests::UserId>>,
) -> ApiResult<responses::UserProfile> {
    current_user.require(Permission::AdminUsersView)?;
    service::get_profile(&state, path.user_id, UserAccess::All).await
}

#[debug_handler]
pub async fn create_user(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateUser>>,
) -> ApiResult<(StatusCode, responses::CreatedUser)> {
    current_user.require(Permission::AdminUsersManage)?;
    service::create_user(&state, input)
        .await
        .map(|user| (StatusCode::CREATED, user))
}

#[debug_handler]
pub async fn update_user(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Path(path)): Valid<Path<requests::UserId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateUser>>,
) -> ApiResult<responses::User> {
    current_user.require(Permission::AdminUsersManage)?;
    service::update_user(&state, path.user_id, input, UserAccess::All).await
}

#[debug_handler]
pub async fn deactivate_user(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Path(path)): Valid<Path<requests::UserId>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::AdminUsersManage)?;
    service::deactivate_user(&state, path.user_id, UserAccess::All).await?;
    Ok(StatusCode::NO_CONTENT)
}
