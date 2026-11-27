use super::{repository::CameraAccess, requests, responses, service};
use crate::{
    error::ApiResult,
    http::{
        authorization::{Authenticated, PartnerContext},
        permissions::{Permission, Scope, ScopeType},
    },
    state::ApiState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use axum_valid::Valid;

fn access(user: &PartnerContext, operation: ScopeType) -> ApiResult<CameraAccess> {
    Ok(match user.resource_scope(operation)? {
        Scope::Own => CameraAccess::PartnerOwned {
            partner_id: user.partner_id,
            user_id: user.user_id,
        },
        Scope::PartnerAll => CameraAccess::Partner {
            partner_id: user.partner_id,
        },
        _ => return Err(crate::error::ApiError::AccessDenied),
    })
}

#[debug_handler]
pub async fn list(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListCamerasQuery>>,
) -> ApiResult<responses::ListCameras> {
    current_user.require(Permission::CamerasView)?;
    let scope = access(&current_user, ScopeType::View)?;
    service::list_cameras_with_scope(&state, input, scope).await
}

#[debug_handler]
pub async fn get_one(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::CameraId>>,
) -> ApiResult<responses::Camera> {
    current_user.require(Permission::CamerasView)?;
    let scope = access(&current_user, ScopeType::View)?;
    service::get_camera_with_scope(&state, id, scope).await
}

#[debug_handler]
pub async fn create(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateCamera>>,
) -> ApiResult<(StatusCode, responses::CreateCamera)> {
    current_user.require(Permission::CamerasWrite)?;
    access(&current_user, ScopeType::Write)?;
    Ok((
        StatusCode::CREATED,
        service::create_camera_with_scope(
            &state,
            input,
            Some(current_user.user_id),
            Some(current_user.partner_id),
        )
        .await?,
    ))
}

#[debug_handler]
pub async fn update(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::CameraId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateCamera>>,
) -> ApiResult<responses::Camera> {
    current_user.require(Permission::CamerasWrite)?;
    let scope = access(&current_user, ScopeType::Write)?;
    service::update_camera_with_scope(&state, id, input, scope).await
}

#[debug_handler]
pub async fn delete(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::CameraId>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::CamerasDelete)?;
    let scope = access(&current_user, ScopeType::Delete)?;
    service::delete_camera_with_scope(&state, id, scope).await?;
    Ok(StatusCode::NO_CONTENT)
}
