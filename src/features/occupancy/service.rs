use crate::{
    error::{ApiError, ApiResult},
    features::zones::{repository::ZoneAccess, service as zones},
    state::ApiState,
};

use super::{
    repository::{self, ObservationAccess},
    requests::{self, CAMERA_CV, DETECTOR_SOURCE},
    responses,
};

fn missing() -> ApiError {
    ApiError::NotFound("occupancy observation".into())
}

fn invalid_capacity() -> ApiError {
    ApiError::UnprocessableEntity("occupied must not exceed capacity".into())
}

#[tracing::instrument(skip(state), err)]
pub async fn list_observations(
    state: &ApiState,
    query: requests::ListObservations,
    access: ObservationAccess,
) -> ApiResult<responses::ListObservations> {
    repository::list_observations(&state.pool, query, access).await
}

#[tracing::instrument(skip(state), err)]
pub async fn get_observation(
    state: &ApiState,
    id: i32,
    access: ObservationAccess,
) -> ApiResult<responses::Observation> {
    repository::find_observation(&state.pool, id, access)
        .await?
        .ok_or_else(missing)
}

#[tracing::instrument(skip(state, input), err)]
pub async fn create_observation(
    state: &ApiState,
    input: requests::CreateObservation,
    scope: ZoneAccess,
    created_by: Option<i32>,
) -> ApiResult<responses::CreatedObservation> {
    let source = match (created_by, input.source_type.as_deref()) {
        (None, None | Some(CAMERA_CV | DETECTOR_SOURCE)) => DETECTOR_SOURCE.to_owned(),
        (None, Some(_)) => {
            return Err(ApiError::UnprocessableEntity(
                "invalid source_type for the detector".into(),
            ));
        }
        (Some(_), Some(DETECTOR_SOURCE)) => {
            return Err(ApiError::UnprocessableEntity(
                "source_type car-detector is reserved for the detector".into(),
            ));
        }
        (Some(_), Some(source)) => source.to_owned(),
        (Some(_), None) => {
            return Err(ApiError::UnprocessableEntity(
                "source_type is required".into(),
            ));
        }
    };
    if input
        .capacity
        .is_some_and(|capacity| input.occupied > capacity)
    {
        return Err(invalid_capacity());
    }
    let zone_id = input.zone_id;
    match repository::create_observation(&state.pool, input, scope, &source, created_by).await? {
        Some(observation_id) => Ok(responses::CreatedObservation { observation_id }),
        None => match zones::observation_insert_error(state, zone_id, scope).await {
            ApiError::BadRequest(_) => Err(invalid_capacity()),
            error => Err(error),
        },
    }
}

#[tracing::instrument(skip(state, input), err)]
pub async fn update_observation(
    state: &ApiState,
    id: i32,
    input: requests::UpdateObservation,
    access: ObservationAccess,
) -> ApiResult<responses::Observation> {
    if input.observed_at.is_none()
        && input.capacity.is_none()
        && input.occupied.is_none()
        && input.confidence.is_none()
        && input.source_ref.is_none()
        && input.metadata.is_none()
    {
        return get_observation(state, id, access).await;
    }
    if let Some(updated) = repository::update_observation(&state.pool, id, input, access).await? {
        return Ok(updated);
    }
    get_observation(state, id, access).await?;
    Err(invalid_capacity())
}

#[tracing::instrument(skip(state), err)]
pub async fn delete_observation(
    state: &ApiState,
    id: i32,
    access: ObservationAccess,
) -> ApiResult<()> {
    if !repository::delete_observation(&state.pool, id, access).await? {
        return Err(missing());
    }
    Ok(())
}
