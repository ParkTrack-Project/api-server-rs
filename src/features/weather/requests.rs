use chrono::{DateTime, Utc};
use serde::Deserialize;
use validator::Validate;

use crate::types::{MAX_OBSERVATION_OFFSET, default_observation_top};

#[derive(Debug, Deserialize, Validate)]
pub struct ListWeather {
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
}

#[derive(Debug, Deserialize, Validate)]
pub struct SaveWeather {
    #[validate(range(min = 1))]
    pub camera_id: i32,
    pub observed_at: DateTime<Utc>,
    pub temperature: f64,
    #[validate(range(min = 0.))]
    pub precipitation: f64,
}
