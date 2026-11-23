use super::{
    repository::{self, ZoneAccess},
    requests, responses,
};
use crate::cameras::repository::CameraAccess;
use crate::{
    error::{ApiError, ApiResult},
    http::{authorization::PublicAuthenticated, permissions::GlobalRole},
    state::ApiState,
    types::CameraView,
};
fn scope(user: &PublicAuthenticated, write: bool) -> ZoneAccess {
    match user {
        PublicAuthenticated::User {
            global_role: GlobalRole::User,
            user_id,
        } => {
            if write {
                ZoneAccess::PublicOwned { user_id: *user_id }
            } else {
                ZoneAccess::Public
            }
        }
        _ => ZoneAccess::All,
    }
}
fn camera_access(user: &PublicAuthenticated) -> CameraAccess {
    match user {
        PublicAuthenticated::User {
            global_role: GlobalRole::User,
            ..
        } => CameraAccess::Public,
        _ => CameraAccess::All,
    }
}
fn missing() -> ApiError {
    ApiError::NotFound("zone".into())
}
#[tracing::instrument(skip(state), err)]
pub async fn list(
    state: &ApiState,
    user: &PublicAuthenticated,
    input: requests::ListZones,
) -> ApiResult<responses::ListZones> {
    list_with_scope(state, input, scope(user, false)).await
}
#[tracing::instrument(skip(state), err)]
pub(crate) async fn list_with_scope(
    state: &ApiState,
    input: requests::ListZones,
    access: ZoneAccess,
) -> ApiResult<responses::ListZones> {
    if let Some(b) = &input.bbox
        && (b.min_longitude > b.max_longitude || b.min_latitude > b.max_latitude)
    {
        return Err(ApiError::BadRequest("invalid bbox bounds".into()));
    }
    let view = input.view;
    let rows = repository::list_zones(&state.pool, input, access).await?;
    Ok(match view {
        CameraView::Full => responses::ListZones::Full(rows.into_iter().map(Into::into).collect()),
        CameraView::Map => responses::ListZones::Map(rows.into_iter().map(Into::into).collect()),
    })
}
#[tracing::instrument(skip(state), err)]
pub async fn get(
    state: &ApiState,
    user: &PublicAuthenticated,
    id: i32,
) -> ApiResult<responses::Zone> {
    get_with_scope(state, id, scope(user, false)).await
}
#[tracing::instrument(skip(state), err)]
pub(crate) async fn get_with_scope(
    state: &ApiState,
    id: i32,
    access: ZoneAccess,
) -> ApiResult<responses::Zone> {
    repository::find(&state.pool, id, access)
        .await?
        .ok_or_else(missing)
        .map(Into::into)
}
#[tracing::instrument(skip(state), err)]
pub async fn create(
    state: &ApiState,
    user: &PublicAuthenticated,
    input: requests::CreateZone,
) -> ApiResult<responses::Created> {
    let owner = match user {
        PublicAuthenticated::User { user_id, .. } => Some(*user_id),
        _ => None,
    };
    create_with_scope(state, input, owner, camera_access(user)).await
}
#[tracing::instrument(skip(state), err)]
pub(crate) async fn create_with_scope(
    state: &ApiState,
    input: requests::CreateZone,
    owner: Option<i32>,
    camera_scope: CameraAccess,
) -> ApiResult<responses::Created> {
    let partner = repository::camera_partner(&state.pool, input.camera_id, camera_scope)
        .await?
        .ok_or_else(|| ApiError::NotFound("camera".into()))?;
    let zone_id = repository::create_zone(&state.pool, input, owner, partner).await?;
    Ok(responses::Created { zone_id })
}
#[tracing::instrument(skip(state), err)]
pub async fn update(
    state: &ApiState,
    user: &PublicAuthenticated,
    id: i32,
    input: requests::UpdateZone,
) -> ApiResult<responses::Zone> {
    update_with_scope(state, id, input, scope(user, true), camera_access(user)).await
}
#[tracing::instrument(skip(state), err)]
pub(crate) async fn update_with_scope(
    state: &ApiState,
    id: i32,
    input: requests::UpdateZone,
    access: ZoneAccess,
    camera_scope: CameraAccess,
) -> ApiResult<responses::Zone> {
    if input.camera_id.is_none()
        && input.zone_type.is_none()
        && input.capacity.is_none()
        && input.pay.is_none()
        && input.occupied.is_none()
        && input.confidence.is_none()
        && input.geometry.is_none()
        && input.image_polygon.is_none()
        && input.location_type.is_none()
        && input.is_private.is_none()
        && input.is_accessible.is_none()
        && input.is_active.is_none()
    {
        return Err(ApiError::BadRequest("missing body".into()));
    }
    repository::find(&state.pool, id, access)
        .await?
        .ok_or_else(missing)?;
    let camera = if let Some(camera_id) = input.camera_id {
        Some(
            repository::camera_partner(&state.pool, camera_id, camera_scope)
                .await?
                .ok_or_else(|| ApiError::NotFound("camera".into()))?,
        )
    } else {
        None
    };
    if repository::update_zone(&state.pool, id, input, access, camera).await? == 0 {
        repository::find(&state.pool, id, access)
            .await?
            .ok_or_else(missing)?;
        return Err(ApiError::BadRequest(
            "occupied must not exceed capacity".into(),
        ));
    }
    repository::find(&state.pool, id, access)
        .await?
        .ok_or_else(missing)
        .map(Into::into)
}
#[tracing::instrument(skip(state), err)]
pub async fn delete(state: &ApiState, user: &PublicAuthenticated, id: i32) -> ApiResult<()> {
    delete_with_scope(state, id, scope(user, true)).await
}
#[tracing::instrument(skip(state), err)]
pub(crate) async fn delete_with_scope(
    state: &ApiState,
    id: i32,
    access: ZoneAccess,
) -> ApiResult<()> {
    if repository::delete_zone(&state.pool, id, access).await? == 0 {
        return Err(missing());
    }
    Ok(())
}
