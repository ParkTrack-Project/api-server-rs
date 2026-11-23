use super::{
    repository::{self, ZoneAccess},
    requests, responses,
};
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
fn missing() -> ApiError {
    ApiError::NotFound("zone".into())
}
#[tracing::instrument(skip(state), err)]
pub async fn list(
    state: &ApiState,
    user: &PublicAuthenticated,
    input: requests::ListZones,
) -> ApiResult<responses::ListZones> {
    if let Some(b) = &input.bbox
        && (b.min_longitude > b.max_longitude || b.min_latitude > b.max_latitude)
    {
        return Err(ApiError::BadRequest("invalid bbox bounds".into()));
    }
    let view = input.view;
    let rows = repository::list_zones(&state.pool, input, scope(user, false)).await?;
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
    repository::find(&state.pool, id, scope(user, false))
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
    let public_only = matches!(
        user,
        PublicAuthenticated::User {
            global_role: GlobalRole::User,
            ..
        }
    );
    let partner = repository::camera_partner(&state.pool, input.camera_id, public_only)
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
    let access = scope(user, true);
    repository::find(&state.pool, id, access)
        .await?
        .ok_or_else(missing)?;
    let camera = if let Some(camera_id) = input.camera_id {
        Some(
            repository::camera_partner(
                &state.pool,
                camera_id,
                matches!(
                    user,
                    PublicAuthenticated::User {
                        global_role: GlobalRole::User,
                        ..
                    }
                ),
            )
            .await?
            .ok_or_else(|| ApiError::NotFound("camera".into()))?,
        )
    } else {
        None
    };
    if repository::update_zone(&state.pool, id, input, access, camera).await? == 0 {
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
    if repository::delete_zone(&state.pool, id, scope(user, true)).await? == 0 {
        return Err(missing());
    }
    Ok(())
}
