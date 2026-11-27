use super::models::ZoneRow;
use axum::{Json, response::IntoResponse};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Zone {
    pub zone_id: i32,
    pub camera_id: i32,
    pub zone_type: String,
    pub capacity: i32,
    pub occupied: i32,
    pub free_count: i32,
    pub confidence: Option<f64>,
    pub confidence_level: Option<String>,
    pub pay: i32,
    pub geometry: serde_json::Value,
    pub image_polygon: serde_json::Value,
    pub partner_id: Option<i32>,
    pub created_by_user_id: Option<i32>,
    pub is_active: Option<bool>,
    pub location_type: Option<String>,
    pub is_private: Option<bool>,
    pub is_accessible: Option<bool>,
    pub occupancy_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl IntoResponse for Zone {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
impl From<ZoneRow> for Zone {
    fn from(row: ZoneRow) -> Self {
        Self {
            zone_id: row.parking_zone_id,
            camera_id: row.camera_id,
            zone_type: row.zone_type,
            capacity: row.capacity,
            occupied: row.occupied,
            free_count: row.free_count,
            confidence: row.confidence,
            confidence_level: row.confidence_level,
            pay: row.pay,
            geometry: row.geometry,
            image_polygon: row.image_polygon,
            partner_id: row.partner_id,
            created_by_user_id: row.created_by_user_id,
            is_active: row.is_active,
            location_type: row.location_type,
            is_private: row.is_private,
            is_accessible: row.is_accessible,
            occupancy_updated_at: row.occupancy_updated_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Created {
    pub zone_id: i32,
}

impl IntoResponse for Created {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ZoneMapItem {
    pub zone_id: i32,
    pub zone_type: String,
    pub capacity: i32,
    pub occupied: i32,
    pub free_count: i32,
    pub confidence: Option<f64>,
    pub confidence_level: Option<String>,
    pub pay: i32,
    pub geometry: serde_json::Value,
    pub is_active: Option<bool>,
    pub location_type: Option<String>,
    pub is_private: Option<bool>,
    pub is_accessible: Option<bool>,
    pub occupancy_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl From<ZoneRow> for ZoneMapItem {
    fn from(row: ZoneRow) -> Self {
        Self {
            zone_id: row.parking_zone_id,
            zone_type: row.zone_type,
            capacity: row.capacity,
            occupied: row.occupied,
            free_count: row.free_count,
            confidence: row.confidence,
            confidence_level: row.confidence_level,
            pay: row.pay,
            geometry: row.geometry,
            is_active: row.is_active,
            location_type: row.location_type,
            is_private: row.is_private,
            is_accessible: row.is_accessible,
            occupancy_updated_at: row.occupancy_updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ListZones {
    Full(Vec<Zone>),
    Map(Vec<ZoneMapItem>),
}

impl IntoResponse for ListZones {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
