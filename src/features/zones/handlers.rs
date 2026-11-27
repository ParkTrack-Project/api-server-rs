use super::{repository::ZoneAccess, requests, responses, service};
use crate::{
    error::{ApiError, ApiResult},
    features::cameras::repository::CameraAccess,
    http::{
        authorization::{Authenticated, AuthenticatedPrincipal, AuthenticatedUser, Principal},
        permissions::{Permission, ServicePermission},
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

// GET /zones
#[debug_handler]
pub async fn list_zones(
    AuthenticatedPrincipal(principal): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListZones>>,
) -> ApiResult<responses::ListZones> {
    principal.require(Permission::ZonesView)?;

    service::list_with_scope(&state, input, ZoneAccess::for_read(&principal)).await
}

// GET /zones/{zone_id}
#[debug_handler]
pub async fn get_zone(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
) -> ApiResult<responses::Zone> {
    user.require(Permission::ZonesView)?;
    service::get_with_scope(&state, id.zone_id, ZoneAccess::for_user_read(&user)).await
}

// POST /zones/new
#[debug_handler]
pub async fn create_zone(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateZone>>,
) -> ApiResult<(StatusCode, responses::Created)> {
    user.require(Permission::ZonesWrite)?;
    let camera_scope = CameraAccess::for_user_read(&user);
    let created =
        service::create_with_scope(&state, input, Some(user.user_id), camera_scope).await?;
    Ok((StatusCode::CREATED, created))
}

// PUT /zones/{zone_id}
#[debug_handler]
pub async fn update_zone(
    AuthenticatedPrincipal(principal): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateZone>>,
) -> ApiResult<responses::Zone> {
    let (input, scope, camera_scope) = match principal {
        Principal::User(user) => {
            user.require(Permission::ZonesWrite)?;
            (
                input,
                ZoneAccess::for_user_write(&user),
                CameraAccess::for_user_read(&user),
            )
        }
        Principal::Service(caller) => {
            caller.require(ServicePermission::ZoneOccupancyWrite)?;
            if input.has_configuration_changes() {
                return Err(ApiError::AccessDenied);
            }
            (
                requests::UpdateZone {
                    occupied: input.occupied,
                    confidence: input.confidence,
                    ..Default::default()
                },
                ZoneAccess::All,
                CameraAccess::All,
            )
        }
    };

    service::update_with_scope(&state, id.zone_id, input, scope, camera_scope).await
}

// DELETE /zones/{zone_id}
#[debug_handler]
pub async fn delete_zone(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
) -> ApiResult<StatusCode> {
    user.require(Permission::ZonesDelete)?;
    service::delete_with_scope(&state, id.zone_id, ZoneAccess::for_user_write(&user)).await?;
    Ok(StatusCode::NO_CONTENT)
}
