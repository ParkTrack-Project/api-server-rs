use axum::{
    Json,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Observation {
    pub observation_id: i32,
    pub zone_id: i32,
    pub camera_id: Option<i32>,
    pub partner_id: Option<i32>,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub capacity: i32,
    pub occupied: i32,
    pub free_count: i32,
    pub confidence: f64,
    pub confidence_level: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub ingested_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
    pub created_by_user_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CreatedObservation {
    pub observation_id: i32,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SeriesPoint {
    pub observed_at: DateTime<Utc>,
    pub occupied: i32,
    pub free_count: i32,
    pub capacity: i32,
    pub confidence: f64,
    pub confidence_level: Option<String>,
    pub source_type: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct MapItem {
    pub zone_id: i32,
    pub camera_id: Option<i32>,
    pub capacity: i32,
    pub occupied: i32,
    pub free_count: i32,
    pub confidence: f64,
    pub confidence_level: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub geometry: Value,
    pub pay: i32,
    pub zone_type: String,
    pub location_type: Option<String>,
    pub is_accessible: Option<bool>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ListObservations {
    Observations(Vec<Observation>),
    Series(Vec<SeriesPoint>),
    Map(Vec<MapItem>),
}

impl IntoResponse for ListObservations {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl IntoResponse for Observation {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl IntoResponse for CreatedObservation {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
