use chrono::{DateTime, Timelike, Utc};
use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::types::{MAX_OBSERVATION_OFFSET, default_observation_top};

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_interval"))]
pub struct ListWeather {
    #[validate(range(min = 1))]
    pub camera_id: Option<i64>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[serde(default)]
    pub latest_only: bool,
    #[serde(default = "default_observation_top")]
    #[validate(range(min = 1, max = 1000))]
    pub top: u64,
    #[serde(default)]
    #[validate(range(max = "MAX_OBSERVATION_OFFSET"))]
    pub offset: u64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct WeatherKey {
    #[validate(range(min = 1))]
    pub camera_id: i64,
    #[validate(custom(function = "validate_hour"))]
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertWeather {
    #[validate(custom(function = "validate_finite"))]
    pub temperature: f64,
    #[validate(range(min = 0.), custom(function = "validate_finite"))]
    pub precipitation: f64,
}

fn validate_interval(query: &ListWeather) -> Result<(), ValidationError> {
    if let (Some(from), Some(to)) = (query.from, query.to)
        && from >= to
    {
        return Err(ValidationError::new("interval").with_message("from must precede to".into()));
    }
    Ok(())
}

fn validate_hour(time: &DateTime<Utc>) -> Result<(), ValidationError> {
    if time.minute() != 0 || time.second() != 0 || time.nanosecond() != 0 {
        return Err(ValidationError::new("hour")
            .with_message("observed_at must be truncated to hour".into()));
    }
    Ok(())
}

fn validate_finite(value: f64) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError::new("finite").with_message("value must be finite".into()));
    }
    Ok(())
}
