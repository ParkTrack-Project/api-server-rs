use axum::{Json, extract::State, http::StatusCode};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::ApiResult,
    features::zones::repository::ZoneAccess,
    http::{authorization::AuthenticatedService, permissions::ServicePermission},
    state::ApiState,
};

use super::{requests, responses, service};

// POST /forecasts/new
#[debug_handler]
pub async fn create_forecast(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateForecast>>,
) -> ApiResult<(StatusCode, Json<responses::CreatedForecast>)> {
    caller.require(ServicePermission::ForecastsWrite)?;
    let forecast_id = service::create_forecast(&state, input, ZoneAccess::All).await?;
    Ok((
        StatusCode::CREATED,
        Json(responses::CreatedForecast { forecast_id }),
    ))
}
