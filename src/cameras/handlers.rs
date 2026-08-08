use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::ApiResult,
    http::{
        authorization::{ApiToken, Authenticated, AuthenticatedOrApiToken},
        permissions::Permission,
    },
    state::ApiState,
};

use super::requests;
use super::responses;
use super::service;

// GET /cameras
#[debug_handler]
pub async fn list_cameras(
    AuthenticatedOrApiToken(current_user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Query(query)): Valid<Query<requests::ListCamerasQuery>>,
) -> ApiResult<responses::ListCameras> {
    current_user.require(&[Permission::CamerasView])?;

    service::list_public_cameras(&state, &current_user, query).await
}

// GET /cameras/next
#[debug_handler]
pub async fn next_camera(
    _: ApiToken,
    State(state): State<ApiState>,
) -> ApiResult<responses::CameraNext> {
    service::next_camera(&state).await
}

// POST /cameras/new
#[debug_handler]
pub async fn create_camera(
    AuthenticatedOrApiToken(current_user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::CreateCamera>>,
) -> ApiResult<responses::CreateCamera> {
    current_user.require(&[Permission::CamerasWrite])?;

    service::create_camera(&state, &current_user, payload).await
}

// GET /cameras/{camera_id}
#[debug_handler]
pub async fn get_camera(
    AuthenticatedOrApiToken(current_user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Path(camera_id)): Valid<Path<requests::CameraId>>,
) -> ApiResult<responses::Camera> {
    current_user.require(&[Permission::CamerasView])?;

    service::get_camera(&state, &current_user, camera_id).await
}

// PUT /cameras/{camera_id}
#[debug_handler]
pub async fn update_camera(
    AuthenticatedOrApiToken(current_user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Path(camera_id)): Valid<Path<requests::CameraId>>,
    Valid(Json(payload)): Valid<Json<requests::UpdateCamera>>,
) -> ApiResult<responses::Camera> {
    current_user.require(&[Permission::CamerasWrite])?;

    service::update_camera(&state, &current_user, camera_id, payload).await
}

// DELETE /cameras/{camera_id}
#[debug_handler]
pub async fn delete_camera(
    AuthenticatedOrApiToken(current_user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Path(camera_id)): Valid<Path<requests::CameraId>>,
) -> ApiResult<impl IntoResponse> {
    current_user.require(&[Permission::CamerasDelete])?;

    service::delete_camera(&state, &current_user, camera_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

// GET /cameras/{camera_id}/snapshot
#[debug_handler]
pub async fn snapshot() {
    todo!();
}
