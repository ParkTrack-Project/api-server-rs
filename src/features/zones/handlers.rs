use super::{requests, responses, service};
use crate::{
    error::ApiResult,
    http::{
        authorization::{Authenticated, AuthenticatedOrApiToken},
        permissions::Permission,
    },
    state::ApiState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use axum_valid::Valid;
#[debug_handler]
pub async fn list(
    AuthenticatedOrApiToken(user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListZones>>,
) -> ApiResult<Json<responses::ListZones>> {
    user.require(Permission::ZonesView)?;
    Ok(Json(service::list(&state, &user, input).await?))
}
#[debug_handler]
pub async fn get(
    AuthenticatedOrApiToken(user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
) -> ApiResult<Json<responses::Zone>> {
    user.require(Permission::ZonesView)?;
    Ok(Json(service::get(&state, &user, id.zone_id).await?))
}
#[debug_handler]
pub async fn create(
    AuthenticatedOrApiToken(user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateZone>>,
) -> ApiResult<(StatusCode, Json<responses::Created>)> {
    user.require(Permission::ZonesWrite)?;
    Ok((
        StatusCode::CREATED,
        Json(service::create(&state, &user, input).await?),
    ))
}
#[debug_handler]
pub async fn update(
    AuthenticatedOrApiToken(user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateZone>>,
) -> ApiResult<Json<responses::Zone>> {
    user.require(Permission::ZonesWrite)?;
    Ok(Json(
        service::update(&state, &user, id.zone_id, input).await?,
    ))
}
#[debug_handler]
pub async fn delete(
    AuthenticatedOrApiToken(user): AuthenticatedOrApiToken,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
) -> ApiResult<StatusCode> {
    user.require(Permission::ZonesDelete)?;
    service::delete(&state, &user, id.zone_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
