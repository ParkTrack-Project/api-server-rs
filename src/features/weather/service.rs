use crate::{error::ApiResult, state::ApiState};

use crate::types::validate_observation_interval;

use super::{repository, requests, responses};

#[tracing::instrument(skip(state), err)]
pub async fn list_weather(
    state: &ApiState,
    query: requests::ListWeather,
) -> ApiResult<Vec<responses::WeatherObservation>> {
    validate_observation_interval(query.from, query.to)?;
    repository::list_weather(&state.pool, query).await
}

#[tracing::instrument(skip(state), err)]
pub async fn save_weather(
    state: &ApiState,
    input: requests::SaveWeather,
) -> ApiResult<responses::WeatherObservation> {
    repository::save_weather(&state.pool, input).await
}
