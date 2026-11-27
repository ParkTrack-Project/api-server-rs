use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct WeatherObservation {
    pub camera_id: i64,
    pub observed_at: DateTime<Utc>,
    pub temperature: f64,
    pub precipitation: f64,
}
