use axum::{
    Json,
    extract::{Query, State},
};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::ApiResult,
    http::{authorization::AuthenticatedService, permissions::ServicePermission},
    state::ApiState,
};

use super::{requests, responses, service};

// GET /weather
#[debug_handler]
pub async fn list_weather(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    Valid(Query(query)): Valid<Query<requests::ListWeather>>,
) -> ApiResult<Json<Vec<responses::WeatherObservation>>> {
    caller.require(ServicePermission::WeatherView)?;
    service::list_weather(&state, query).await.map(Json)
}

// POST /weather/new
#[debug_handler]
pub async fn save_weather(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::SaveWeather>>,
) -> ApiResult<Json<responses::WeatherObservation>> {
    caller.require(ServicePermission::WeatherWrite)?;
    service::save_weather(&state, input).await.map(Json)
}
