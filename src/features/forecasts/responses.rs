use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CreatedForecast {
    pub forecast_id: i32,
}
