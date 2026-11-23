use crate::{
    error::{ApiError, ApiResult},
    features::cameras::repository::CameraAccess,
    http::{
        authorization::PublicAuthenticated,
        permissions::{GlobalRole, ScopeType},
    },
    state::ApiState,
    types::CameraView,
};
use std::sync::LazyLock;

use super::repository;
use super::requests;
use super::responses;

#[tracing::instrument(skip(state), err)]
pub async fn list_public_cameras(
    state: &ApiState,
    current_user: &PublicAuthenticated,
    query: requests::ListCamerasQuery,
) -> ApiResult<responses::ListCameras> {
    let result =
        list_cameras_with_scope(state, query, public_scope(current_user, ScopeType::View)).await?;

    Ok(result)
}

static CAMERA_CURSOR: LazyLock<tokio::sync::Mutex<i32>> =
    LazyLock::new(|| tokio::sync::Mutex::new(0));

#[tracing::instrument(skip(state), err)]
pub async fn next_camera(state: &ApiState) -> ApiResult<responses::CameraNext> {
    let idx = {
        let mut cursor = CAMERA_CURSOR.lock().await;
        let count = repository::count_cameras(&state.pool).await?;
        let idx = *cursor % count as i32;
        *cursor += 1;
        idx
    };

    let result = repository::get_next_camera(&state.pool, idx)
        .await
        .map(responses::CameraNext::from)?;

    Ok(result)
}

#[tracing::instrument(skip(state), err)]
pub async fn create_camera(
    state: &ApiState,
    current_user: &PublicAuthenticated,
    payload: requests::CreateCamera,
) -> ApiResult<responses::CreateCamera> {
    let user_id: Option<i32> = match current_user {
        PublicAuthenticated::User { user_id, .. } => Some(*user_id),
        PublicAuthenticated::ApiToken => None,
    };

    create_camera_with_scope(state, payload, user_id, None).await
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
pub async fn get_camera(
    state: &ApiState,
    current_user: &PublicAuthenticated,
    query: requests::CameraId,
) -> ApiResult<responses::Camera> {
    get_camera_with_scope(state, query, public_scope(current_user, ScopeType::View)).await
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
pub async fn update_camera(
    state: &ApiState,
    current_user: &PublicAuthenticated,
    query: requests::CameraId,
    payload: requests::UpdateCamera,
) -> ApiResult<responses::Camera> {
    update_camera_with_scope(
        state,
        query,
        payload,
        public_scope(current_user, ScopeType::Write),
    )
    .await
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
pub async fn delete_camera(
    state: &ApiState,
    current_user: &PublicAuthenticated,
    query: requests::CameraId,
) -> ApiResult<()> {
    delete_camera_with_scope(state, query, public_scope(current_user, ScopeType::Delete)).await
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

fn public_scope(current_user: &PublicAuthenticated, scope: ScopeType) -> CameraAccess {
    match current_user {
        PublicAuthenticated::User {
            global_role: GlobalRole::User,
            user_id,
        } => match scope {
            ScopeType::Write | ScopeType::Delete => CameraAccess::PublicOwned { user_id: *user_id },
            ScopeType::View => CameraAccess::Public,
        },
        PublicAuthenticated::User {
            global_role: GlobalRole::Admin,
            ..
        }
        | PublicAuthenticated::ApiToken => CameraAccess::All,
    }
}
