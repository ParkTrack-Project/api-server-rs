use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use validator::Validate;

use crate::types::{ConfidenceLevel, MAX_OBSERVATION_OFFSET, default_observation_top};

#[derive(Debug, Deserialize, Validate)]
pub struct ListObservations {
    #[validate(range(min = 1))]
    pub camera_id: Option<i32>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[serde(default = "default_observation_top")]
    #[validate(range(min = 1, max = 1000))]
    pub top: u64,
    #[serde(default)]
    #[validate(range(max = "MAX_OBSERVATION_OFFSET"))]
    pub offset: u64,
    #[validate(range(min = 1))]
    pub zone_id: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateObservation {
    #[validate(range(min = 1))]
    pub zone_id: i32,
    #[validate(range(min = 0))]
    pub occupied: i32,
    #[validate(range(min = 0., max = 1.))]
    pub confidence: f64,
    pub confidence_level: Option<ConfidenceLevel>,
    #[validate(length(min = 1, max = 255))]
    pub source_ref: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub metadata: Option<Value>,
}
