use serde::Deserialize;
use validator::Validate;

use crate::types::{Bbox, CameraView};

#[derive(Debug, Validate, Deserialize)]
pub struct ListCamerasQuery {
    pub q: Option<String>,
    pub is_active: Option<bool>,
    #[validate(nested)]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    pub view: CameraView,
}

#[derive(Debug, Validate, Deserialize)]
pub struct GetCamerasSnapshotQuery {
    #[serde(default)]
    pub annotated: bool,
    #[serde(default)]
    pub last_detection: bool,
    #[serde(default)]
    pub fallback_to_raw: bool,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateCamera {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub source: String,
    #[validate(range(exclusive_min = 0))]
    pub image_width: i32,
    #[validate(range(exclusive_min = 0))]
    pub image_height: i32,
    pub calib: Option<serde_json::Value>,
    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: f32,
    #[validate(range(min = -90.0, max = 90.0))]
    pub longitude: f32,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateCamera {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub source: Option<String>,
    #[validate(range(exclusive_min = 0))]
    pub image_width: Option<i32>,
    #[validate(range(exclusive_min = 0))]
    pub image_height: Option<i32>,
    pub calib: Option<serde_json::Value>,
    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: Option<f32>,
    #[validate(range(min = -90.0, max = 90.0))]
    pub longitude: Option<f32>,
}

impl UpdateCamera {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.source.is_none()
            && self.image_width.is_none()
            && self.image_height.is_none()
            && self.calib.is_none()
            && self.latitude.is_none()
            && self.longitude.is_none()
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct CameraId {
    #[validate(range(exclusive_min = 0))]
    pub camera_id: i32
}