use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use axum_valid::Valid;

use crate::{
    error::ApiResult,
    features::zones::repository::ZoneAccess,
    http::{authorization::AuthenticatedService, permissions::ServicePermission},
    state::ApiState,
};

use super::{requests, responses, service};

// GET /occupancy
#[debug_handler]
pub async fn list_observations(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    Valid(Query(query)): Valid<Query<requests::ListObservations>>,
) -> ApiResult<Json<Vec<responses::Observation>>> {
    caller.require(ServicePermission::OccupancyView)?;
    service::list_observations(&state, query).await.map(Json)
}

// POST /occupancy/new
#[debug_handler]
pub async fn create_observation(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateObservation>>,
) -> ApiResult<(StatusCode, Json<responses::CreatedObservation>)> {
    caller.require(ServicePermission::OccupancyWrite)?;
    let id = service::create_observation(&state, input, ZoneAccess::All).await?;
    Ok((
        StatusCode::CREATED,
        Json(responses::CreatedObservation { observation_id: id }),
    ))
}
