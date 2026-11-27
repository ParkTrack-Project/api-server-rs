use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use axum_valid::{Valid, ValidRejection, ValidationRejection};
use serde_json::json;

use crate::{
    error::ApiError,
    http::{authorization::AuthenticatedService, permissions::ServicePermission},
    state::ApiState,
};

use super::{requests, responses, service};

type Validated<T, E> = Result<Valid<T>, ValidRejection<E>>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum WeatherError {
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error("{1}")]
    Request(StatusCode, String),
}

impl IntoResponse for WeatherError {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Request(status, message) => {
                (status, Json(json!({"error_description": message}))).into_response()
            }
        }
    }
}

fn invalid_parameters<E: std::fmt::Display>(error: ValidRejection<E>) -> WeatherError {
    WeatherError::Request(StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
}

fn invalid_body(error: ValidRejection<JsonRejection>) -> WeatherError {
    let status = match &error {
        ValidationRejection::Valid(_) => StatusCode::UNPROCESSABLE_ENTITY,
        ValidationRejection::Inner(
            JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_),
        ) => StatusCode::BAD_REQUEST,
        ValidationRejection::Inner(error) => error.status(),
    };
    WeatherError::Request(status, error.to_string())
}

// GET /weather
#[debug_handler]
pub(crate) async fn list_weather(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    query: Validated<Query<requests::ListWeather>, QueryRejection>,
) -> Result<responses::ListWeather, WeatherError> {
    caller.require(ServicePermission::WeatherView)?;
    let Valid(Query(query)) = query.map_err(invalid_parameters)?;
    service::list_weather(&state, query)
        .await
        .map_err(WeatherError::from)
}

// GET /weather/{camera_id}/{observed_at}
#[debug_handler]
pub(crate) async fn get_weather(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    key: Validated<Path<requests::WeatherKey>, PathRejection>,
) -> Result<responses::WeatherObservation, WeatherError> {
    caller.require(ServicePermission::WeatherView)?;
    let Valid(Path(key)) = key.map_err(invalid_parameters)?;
    service::get_weather(&state, key)
        .await
        .map_err(WeatherError::from)
}

// PUT /weather/{camera_id}/{observed_at}
#[debug_handler]
pub(crate) async fn upsert_weather(
    AuthenticatedService(caller): AuthenticatedService,
    State(state): State<ApiState>,
    key: Validated<Path<requests::WeatherKey>, PathRejection>,
    input: Validated<Json<requests::UpsertWeather>, JsonRejection>,
) -> Result<(StatusCode, responses::WeatherKey), WeatherError> {
    caller.require(ServicePermission::WeatherWrite)?;
    let Valid(Path(key)) = key.map_err(invalid_parameters)?;
    let Valid(Json(input)) = input.map_err(invalid_body)?;
    let (key, inserted) = service::upsert_weather(&state, key, input).await?;
    let status = if inserted {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, key))
}
