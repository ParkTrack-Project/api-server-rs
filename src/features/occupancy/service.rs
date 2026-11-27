use crate::{
    error::ApiResult,
    features::zones::{repository::ZoneAccess, service as zones},
    state::ApiState,
};

use crate::types::validate_observation_interval;

use super::{repository, requests, responses};

#[tracing::instrument(skip(state), err)]
pub async fn list_observations(
    state: &ApiState,
    query: requests::ListObservations,
) -> ApiResult<Vec<responses::Observation>> {
    validate_observation_interval(query.from, query.to)?;
    repository::list_observations(&state.pool, query).await
}

#[tracing::instrument(skip(state, input), err)]
pub async fn create_observation(
    state: &ApiState,
    input: requests::CreateObservation,
    scope: ZoneAccess,
) -> ApiResult<i32> {
    let zone_id = input.zone_id;
    match repository::create_observation(&state.pool, input, scope).await? {
        Some(id) => Ok(id),
        None => Err(zones::observation_insert_error(state, zone_id, scope).await),
    }
}
