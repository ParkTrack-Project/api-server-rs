use axum::{
    Json,
    response::{IntoResponse, Response},
};
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

impl IntoResponse for WeatherObservation {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ListWeather(pub Vec<WeatherObservation>);

impl IntoResponse for ListWeather {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct WeatherKey {
    pub camera_id: i64,
    pub observed_at: DateTime<Utc>,
}

impl IntoResponse for WeatherKey {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
