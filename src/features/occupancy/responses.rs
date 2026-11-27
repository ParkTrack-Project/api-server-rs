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
}

#[derive(Debug, Serialize)]
pub struct CreatedObservation {
    pub observation_id: i32,
}
