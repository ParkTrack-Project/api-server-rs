use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use validator::Validate;

use crate::types::ConfidenceLevel;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateForecast {
    #[validate(range(min = 1))]
    pub zone_id: i32,
    #[validate(length(min = 1, max = 50))]
    pub model_type: String,
    #[validate(length(min = 1, max = 100))]
    pub model_version: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub predicted_for: DateTime<Utc>,
    #[validate(range(min = 0))]
    pub predicted_occupied: i32,
    #[validate(range(min = 0., max = 1.))]
    pub probability_free_space: f64,
    #[validate(range(min = 0., max = 1.))]
    pub confidence: f64,
    pub confidence_level: Option<ConfidenceLevel>,
    pub metadata: Option<Value>,
}
