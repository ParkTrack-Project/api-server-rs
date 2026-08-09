use crate::{
    cameras::repository::CameraAccess,
    error::{ApiError, ApiResult},
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
    let result = list_cameras_with_scope(state, query, public_scope(current_user, ScopeType::View))
        .await?;

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

    let result = repository::create_camera(&state.pool, payload, user_id)
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
    let result = repository::get_camera(
        &state.pool,
        query.camera_id,
        public_scope(current_user, ScopeType::View),
    )
    .await
    .and_then(|res| {
        res.ok_or_else(|| ApiError::NotFound(format!("camera of id {} not found", query.camera_id)))
    })
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
    if payload.is_empty() {
        return Err(ApiError::BadRequest("missing body".to_string()));
    }

    let result = repository::update_camera(
        &state.pool,
        query.camera_id,
        payload,
        public_scope(current_user, ScopeType::Write),
    )
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
    let _ = repository::delete_camera(
        &state.pool,
        query.camera_id,
        public_scope(current_user, ScopeType::Delete),
    )
    .await?;

    tracing::info!(camera_id = query.camera_id, "camera deleted");

    Ok(())
}

// pub async fn list_partner_cameras(
//     state: &ApiState,
//     current_user: &PartnerAuthenticated,
//     partner_id: u32,
//     query: &ListCamerasQuery,
// ) -> ApiResult<ListCamerasResponse> {
//     current_user.require(&[Permission::CamerasView])?;

//     list_cameras_with_scope(state, query, partner_scope(current_user, partner_id)?).await
// }

async fn list_cameras_with_scope(
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

// fn partner_scope(current_user: &PartnerAuthenticated, partner_id: i32) -> ApiResult<CameraAccess> {
//     let scope = match current_user {
//         PartnerAuthenticated::Admin { .. } => CameraAccess::Partner { partner_id },
//         PartnerAuthenticated::Member {
//             user_id,
//             view_scope: Scope::Owned,
//             ..
//         } => {
//             CameraAccess::PartnerOwned {
//                 partner_id,
//                 user_id: *user_id,
//             }
//         }
//         PartnerAuthenticated::Member {
//             view_scope: Scope::PartnerAll,
//             ..
//         } => CameraAccess::Partner { partner_id },
//     };

//     Ok(scope)
// }
