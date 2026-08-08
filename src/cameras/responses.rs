use axum::{Json, response::IntoResponse};
use serde::Serialize;
use time::OffsetDateTime;

use crate::cameras::models::{CameraMapItemRow, CameraNextRow, CameraRow, CreateCameraRow};

#[derive(Debug, Serialize)]
pub struct Camera {
    camera_id: u32,
    title: String,
    source: String,
    image_width: u32,
    image_height: u32,
    calib: Option<serde_json::Value>,
    latitude: f32,
    longitude: f32,
    partner_id: Option<u32>,
    created_by_user_id: Option<u32>,
    is_active: bool,
    last_snapshot_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl IntoResponse for Camera {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl From<CameraRow> for Camera {
    fn from(row: CameraRow) -> Self {
        Self {
            camera_id: row.camera_id as u32,
            title: row.title,
            source: row.source,
            image_width: row.image_width as u32,
            image_height: row.image_height as u32,
            calib: row.calib,
            latitude: row.latitude,
            longitude: row.longitude,
            partner_id: row.partner_id.map(|x| x as u32),
            created_by_user_id: row.created_by_user_id.map(|x| x as u32),
            is_active: row.is_active,
            last_snapshot_at: row.last_snapshot_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CameraMapItem {
    camera_id: u32,
    title: String,
    latitude: f32,
    longitude: f32,
    partner_id: Option<u32>,
    is_active: bool,
}

impl IntoResponse for CameraMapItem {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl From<CameraMapItemRow> for CameraMapItem {
    fn from(row: CameraMapItemRow) -> Self {
        Self {
            camera_id: row.camera_id as u32,
            title: row.title,
            latitude: row.latitude,
            longitude: row.longitude,
            partner_id: row.partner_id.map(|x| x as u32),
            is_active: row.is_active,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CameraNext {
    camera_id: u32,
    source: String,
    image_width: u32,
    image_height: u32,
    calib: Option<serde_json::Value>,
    partner_id: Option<u32>,
    is_active: bool,
}

impl From<CameraNextRow> for CameraNext {
    fn from(row: CameraNextRow) -> Self {
        Self {
            camera_id: row.camera_id as u32,
            source: row.source,
            image_height: row.image_height as u32,
            image_width: row.image_width as u32,
            calib: row.calib,
            partner_id: row.partner_id.map(|x| x as u32),
            is_active: row.is_active,
        }
    }
}

impl IntoResponse for CameraNext {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub enum ListCameras {
    Full(Vec<Camera>),
    Map(Vec<CameraMapItem>),
}

impl IntoResponse for ListCameras {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct CreateCamera {
    pub camera_id: i32,
}

impl From<CreateCameraRow> for CreateCamera {
    fn from(row: CreateCameraRow) -> Self {
        Self {
            camera_id: row.camera_id,
        }
    }
}

impl IntoResponse for CreateCamera {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
