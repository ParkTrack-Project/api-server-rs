use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::ApiResult,
    http::{
        authorization::{
            Authenticated, AuthenticatedPrincipal, AuthenticatedService, AuthenticatedUser,
        },
        permissions::{Permission, ServicePermission},
    },
    state::ApiState,
};

use super::repository::CameraAccess;
use super::requests;
use super::responses;
use super::service;

// GET /cameras
#[debug_handler]
pub async fn list_cameras(
    AuthenticatedPrincipal(principal): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    Valid(Query(query)): Valid<Query<requests::ListCamerasQuery>>,
) -> ApiResult<responses::ListCameras> {
    principal.require(Permission::CamerasView)?;
    service::list_cameras_with_scope(&state, query, CameraAccess::for_read(&principal)).await
}

// GET /cameras/next
#[debug_handler]
pub async fn next_camera(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
) -> ApiResult<responses::CameraNext> {
    caller.require(ServicePermission::DetectionClaim)?;
    service::next_camera(&state, CameraAccess::All).await
}

// POST /cameras/new
#[debug_handler]
pub async fn create_camera(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Json(payload)): Valid<Json<requests::CreateCamera>>,
) -> ApiResult<responses::CreateCamera> {
    current_user.require(Permission::CamerasWrite)?;

    service::create_camera_with_scope(&state, payload, Some(current_user.user_id), None).await
}

// GET /cameras/{camera_id}
#[debug_handler]
pub async fn get_camera(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(camera_id)): Valid<Path<requests::CameraId>>,
) -> ApiResult<responses::Camera> {
    current_user.require(Permission::CamerasView)?;

    service::get_camera_with_scope(
        &state,
        camera_id,
        CameraAccess::for_user_read(&current_user),
    )
    .await
}

// PUT /cameras/{camera_id}
#[debug_handler]
pub async fn update_camera(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(camera_id)): Valid<Path<requests::CameraId>>,
    Valid(Json(payload)): Valid<Json<requests::UpdateCamera>>,
) -> ApiResult<responses::Camera> {
    current_user.require(Permission::CamerasWrite)?;

    service::update_camera_with_scope(
        &state,
        camera_id,
        payload,
        CameraAccess::for_user_write(&current_user),
    )
    .await
}

// DELETE /cameras/{camera_id}
#[debug_handler]
pub async fn delete_camera(
    AuthenticatedUser(current_user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(camera_id)): Valid<Path<requests::CameraId>>,
) -> ApiResult<impl IntoResponse> {
    current_user.require(Permission::CamerasDelete)?;

    service::delete_camera_with_scope(
        &state,
        camera_id,
        CameraAccess::for_user_write(&current_user),
    )
    .await
}

// GET /cameras/{camera_id}/snapshot
#[debug_handler]
pub async fn snapshot() {
    todo!();
}
