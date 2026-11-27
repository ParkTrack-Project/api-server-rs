use crate::{
    error::{ApiError, ApiResult},
    features::cameras::repository::CameraAccess,
    http::{
        authorization::{Principal, UserIdentity},
        permissions::GlobalRole,
    },
    state::ApiState,
    types::CameraView,
};

use super::repository;
use super::requests;
use super::responses;

#[tracing::instrument(skip(state), err)]
pub async fn next_camera(
    state: &ApiState,
    scope: CameraAccess,
) -> ApiResult<responses::CameraNext> {
    let mut cursor = state.next_camera_cursor.lock().await;
    let next = repository::find_next_camera(&state.pool, *cursor, scope).await?;
    let next = match next {
        None if cursor.is_some() => repository::find_next_camera(&state.pool, None, scope).await?,
        next => next,
    };
    *cursor = next.as_ref().map(|camera| camera.camera_id);
    next.map(Into::into)
        .ok_or_else(|| ApiError::NotFound("camera".into()))
}

#[tracing::instrument(skip(state), err)]
pub(crate) async fn create_camera_with_scope(
    state: &ApiState,
    payload: requests::CreateCamera,
    user_id: Option<i32>,
    partner_id: Option<i32>,
) -> ApiResult<responses::CreateCamera> {
    let result = repository::create_camera_with_partner(&state.pool, payload, user_id, partner_id)
        .await
        .map(responses::CreateCamera::from)?;

    tracing::info!(camera_id = result.camera_id, "camera created");

    Ok(result)
}

#[tracing::instrument(skip(state), err)]
pub(crate) async fn get_camera_with_scope(
    state: &ApiState,
    query: requests::CameraId,
    scope: CameraAccess,
) -> ApiResult<responses::Camera> {
    let result = repository::find_by_id(&state.pool, query.camera_id, scope)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("camera of id {} not found", query.camera_id)))
        .map(responses::Camera::from)?;

    Ok(result)
}

#[tracing::instrument(skip(state), err)]
pub(crate) async fn update_camera_with_scope(
    state: &ApiState,
    query: requests::CameraId,
    payload: requests::UpdateCamera,
    scope: CameraAccess,
) -> ApiResult<responses::Camera> {
    if payload.is_empty() {
        return Err(ApiError::BadRequest("missing body".to_string()));
    }

    let result = repository::update_camera(&state.pool, query.camera_id, payload, scope)
        .await
        .and_then(|res| {
            res.ok_or_else(|| ApiError::NotFound(format!("camera with id {}", query.camera_id)))
        })
        .map(responses::Camera::from)?;

    tracing::info!(camera_id = query.camera_id, "camera updated");

    Ok(result)
}

#[tracing::instrument(skip(state), err)]
pub(crate) async fn delete_camera_with_scope(
    state: &ApiState,
    query: requests::CameraId,
    scope: CameraAccess,
) -> ApiResult<()> {
    let rows_affected = repository::delete_camera(&state.pool, query.camera_id, scope).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!(
            "camera with id {}",
            query.camera_id
        )));
    }

    tracing::info!(camera_id = query.camera_id, "camera deleted");

    Ok(())
}

#[tracing::instrument(skip(state), err)]
pub(crate) async fn list_cameras_with_scope(
    state: &ApiState,
    query: requests::ListCamerasQuery,
    scope: CameraAccess,
) -> ApiResult<responses::ListCameras> {
    match query.view {
        CameraView::Full => {
            let cameras = repository::list_cameras(&state.pool, query, scope)
                .await?
                .into_iter()
                .map(responses::Camera::from)
                .collect();

            Ok(responses::ListCameras::Full(cameras))
        }
        CameraView::Map => {
            let cameras = repository::list_camera_map_items(&state.pool, query, scope)
                .await?
                .into_iter()
                .map(responses::CameraMapItem::from)
                .collect();

            Ok(responses::ListCameras::Map(cameras))
        }
    }
}

impl CameraAccess {
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
