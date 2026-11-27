use crate::{
    error::{ApiError, ApiResult},
    features::zones::{repository::ZoneAccess, service as zones},
    state::ApiState,
};

use super::{repository, requests};

#[tracing::instrument(skip(state, input), err)]
pub async fn create_forecast(
    state: &ApiState,
    input: requests::CreateForecast,
    scope: ZoneAccess,
) -> ApiResult<i32> {
    if input.predicted_for < input.generated_at {
        return Err(ApiError::BadRequest(
            "predicted_for must be at or after generated_at".into(),
        ));
    }

    let zone_id = input.zone_id;
    match repository::create_forecast(&state.pool, input, scope).await? {
        Some(id) => Ok(id),
        None => Err(zones::observation_insert_error(state, zone_id, scope).await),
    }
}
