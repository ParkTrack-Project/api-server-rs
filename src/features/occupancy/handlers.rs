use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
};
use axum_macros::debug_handler;
use axum_valid::{Valid, ValidRejection, ValidationRejection};

use crate::{
    error::{ApiError, ApiResult},
    features::zones::repository::ZoneAccess,
    http::{
        authorization::{Authenticated, AuthenticatedPrincipal, Principal},
        permissions::{GlobalRole, Permission, ServicePermission},
    },
    state::ApiState,
};

use super::{repository::ObservationAccess, requests, responses, service};

type Validated<T, E> = Result<Valid<T>, ValidRejection<E>>;

fn invalid_parameters<E: std::fmt::Display>(error: ValidRejection<E>) -> ApiError {
    ApiError::UnprocessableEntity(error.to_string())
}

fn invalid_body(error: ValidRejection<JsonRejection>) -> ApiError {
    match &error {
        ValidationRejection::Valid(_) => ApiError::UnprocessableEntity(error.to_string()),
        ValidationRejection::Inner(JsonRejection::MissingJsonContentType(_)) => {
            ApiError::UnsupportedMediaType
        }
        ValidationRejection::Inner(inner) if inner.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            ApiError::PayloadTooLarge
        }
        _ => ApiError::BadRequest(error.to_string()),
    }
}

// GET /occupancy
#[debug_handler]
pub async fn list_observations(
    AuthenticatedPrincipal(caller): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    query: Validated<Query<requests::ListObservations>, QueryRejection>,
) -> ApiResult<responses::ListObservations> {
    let access = match caller {
        Principal::User(user) => {
            user.require(Permission::OccupancyView)?;
            if user.global_role == GlobalRole::Admin {
                ObservationAccess::All
            } else {
                ObservationAccess::Public
            }
        }
        Principal::Service(service) => {
            service.require(ServicePermission::OccupancyView)?;
            ObservationAccess::All
        }
    };
    let Valid(Query(query)) = query.map_err(invalid_parameters)?;
    service::list_observations(&state, query, access).await
}

// GET /occupancy/{observation_id}
#[debug_handler]
pub async fn get_observation(
    AuthenticatedPrincipal(caller): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    path: Validated<Path<requests::ObservationId>, PathRejection>,
) -> ApiResult<responses::Observation> {
    let access = match caller {
        Principal::User(user) => {
            user.require(Permission::OccupancyView)?;
            if user.global_role == GlobalRole::Admin {
                ObservationAccess::All
            } else {
                ObservationAccess::Public
            }
        }
        Principal::Service(service) => {
            service.require(ServicePermission::OccupancyView)?;
            ObservationAccess::All
        }
    };
    let Valid(Path(path)) = path.map_err(invalid_parameters)?;
    service::get_observation(&state, path.observation_id, access).await
}

// POST /occupancy/new
#[debug_handler]
pub async fn create_observation(
    AuthenticatedPrincipal(caller): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    input: Validated<Json<requests::CreateObservation>, JsonRejection>,
) -> ApiResult<(StatusCode, responses::CreatedObservation)> {
    let (scope, created_by) = match caller {
        Principal::User(user) => {
            user.require(Permission::OccupancyWrite)?;
            (ZoneAccess::for_user_write(&user), Some(user.user_id))
        }
        Principal::Service(service) => {
            service.require(ServicePermission::OccupancyWrite)?;
            (ZoneAccess::All, None)
        }
    };
    let Valid(Json(input)) = input.map_err(invalid_body)?;
    let created = service::create_observation(&state, input, scope, created_by).await?;
    Ok((StatusCode::CREATED, created))
}

// PUT /occupancy/{observation_id}
#[debug_handler]
pub async fn update_observation(
    AuthenticatedPrincipal(caller): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    path: Validated<Path<requests::ObservationId>, PathRejection>,
    input: Validated<Json<requests::UpdateObservation>, JsonRejection>,
) -> ApiResult<responses::Observation> {
    let access = match caller {
        Principal::User(user) => {
            user.require(Permission::OccupancyWrite)?;
            if user.global_role != GlobalRole::Admin {
                return Err(ApiError::AccessDenied);
            }
            ObservationAccess::All
        }
        Principal::Service(service) => {
            service.require(ServicePermission::OccupancyWrite)?;
            ObservationAccess::Detector
        }
    };
    let Valid(Path(path)) = path.map_err(invalid_parameters)?;
    let Valid(Json(input)) = input.map_err(invalid_body)?;
    service::update_observation(&state, path.observation_id, input, access).await
}

// DELETE /occupancy/{observation_id}
#[debug_handler]
pub async fn delete_observation(
    AuthenticatedPrincipal(caller): AuthenticatedPrincipal,
    State(state): State<ApiState>,
    path: Validated<Path<requests::ObservationId>, PathRejection>,
) -> ApiResult<StatusCode> {
    let Principal::User(user) = caller else {
        return Err(ApiError::Forbidden(Permission::OccupancyDelete));
    };
    user.require(Permission::OccupancyDelete)?;
    if user.global_role != GlobalRole::Admin {
        return Err(ApiError::AccessDenied);
    }
    let Valid(Path(path)) = path.map_err(invalid_parameters)?;
    service::delete_observation(&state, path.observation_id, ObservationAccess::All).await?;
    Ok(StatusCode::NO_CONTENT)
}
