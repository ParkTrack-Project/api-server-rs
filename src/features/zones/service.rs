use super::{
    repository::{self, ZoneAccess},
    requests, responses,
};
use crate::features::cameras::repository::CameraAccess;
use crate::{
    error::{ApiError, ApiResult},
    http::{
        authorization::{Principal, UserIdentity},
        permissions::GlobalRole,
    },
    state::ApiState,
    types::CameraView,
};
impl ZoneAccess {
    pub fn for_read(principal: &Principal) -> Self {
        match principal {
            Principal::User(user) => Self::for_user_read(user),
            Principal::Service(_) => Self::All,
        }
    }

    pub fn for_user_read(user: &UserIdentity) -> Self {
        match user.global_role {
            GlobalRole::User => Self::Public,
            GlobalRole::Admin => Self::All,
        }
    }

    pub fn for_user_write(user: &UserIdentity) -> Self {
        match user.global_role {
            GlobalRole::User => Self::PublicOwned {
                user_id: user.user_id,
            },
            GlobalRole::Admin => Self::All,
        }
    }
}

fn missing() -> ApiError {
    ApiError::NotFound("zone".into())
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
pub(crate) async fn update_with_scope(
    state: &ApiState,
    id: i32,
    input: requests::UpdateZone,
    access: ZoneAccess,
    camera_scope: CameraAccess,
) -> ApiResult<responses::Zone> {
    if input.is_empty() {
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

pub(crate) async fn observation_insert_error(
    state: &ApiState,
    id: i32,
    scope: ZoneAccess,
) -> ApiError {
    match repository::find(&state.pool, id, scope).await {
        Ok(Some(_)) => ApiError::BadRequest("occupied must not exceed capacity".into()),
        Ok(None) => missing(),
        Err(error) => error,
    }
}
