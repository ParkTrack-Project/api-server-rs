use crate::{
    error::{ApiError, ApiResult},
    state::ApiState,
};

use super::{repository, requests, responses};

#[tracing::instrument(skip(state), err)]
pub async fn list_weather(
    state: &ApiState,
    query: requests::ListWeather,
) -> ApiResult<responses::ListWeather> {
    repository::list_weather(&state.pool, query)
        .await
        .map(responses::ListWeather)
}

#[tracing::instrument(skip(state), err)]
pub async fn get_weather(
    state: &ApiState,
    key: requests::WeatherKey,
) -> ApiResult<responses::WeatherObservation> {
    repository::find_weather(&state.pool, &key)
        .await?
        .ok_or_else(|| ApiError::NotFound("weather observation".into()))
}

#[tracing::instrument(skip(state), err)]
pub async fn upsert_weather(
    state: &ApiState,
    key: requests::WeatherKey,
    input: requests::UpsertWeather,
) -> ApiResult<(responses::WeatherKey, bool)> {
    let inserted = repository::upsert_weather(&state.pool, &key, input).await?;
    Ok((
        responses::WeatherKey {
            camera_id: key.camera_id,
            observed_at: key.observed_at,
        },
        inserted,
    ))
}
