use super::{repository::ZoneAccess, requests, responses, service};
use crate::{
    error::ApiResult,
    http::{
        authorization::{Authenticated, PartnerContext},
        permissions::{Permission, Scope, ScopeType},
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

fn access(user: &PartnerContext, operation: ScopeType) -> ApiResult<ZoneAccess> {
    Ok(match user.resource_scope(operation)? {
        Scope::Own => ZoneAccess::PartnerOwned {
            partner_id: user.partner_id,
            user_id: user.user_id,
        },
        Scope::PartnerAll => ZoneAccess::Partner {
            partner_id: user.partner_id,
        },
        _ => return Err(crate::error::ApiError::AccessDenied),
    })
}

#[debug_handler]
pub async fn list(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListZones>>,
) -> ApiResult<Json<responses::ListZones>> {
    current_user.require(Permission::ZonesView)?;
    let scope = access(&current_user, ScopeType::View)?;
    Ok(Json(service::list_with_scope(&state, input, scope).await?))
}

#[debug_handler]
pub async fn get_one(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
) -> ApiResult<Json<responses::Zone>> {
    current_user.require(Permission::ZonesView)?;
    let scope = access(&current_user, ScopeType::View)?;
    Ok(Json(
        service::get_with_scope(&state, id.zone_id, scope).await?,
    ))
}

#[debug_handler]
pub async fn create(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateZone>>,
) -> ApiResult<(StatusCode, Json<responses::Created>)> {
    current_user.require(Permission::ZonesWrite)?;
    access(&current_user, ScopeType::Write)?;
    Ok((
        StatusCode::CREATED,
        Json(
            service::create_with_scope(
                &state,
                input,
                Some(current_user.user_id),
                crate::cameras::repository::CameraAccess::Partner {
                    partner_id: current_user.partner_id,
                },
            )
            .await?,
        ),
    ))
}

#[debug_handler]
pub async fn update(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateZone>>,
) -> ApiResult<Json<responses::Zone>> {
    current_user.require(Permission::ZonesWrite)?;
    let scope = access(&current_user, ScopeType::Write)?;
    Ok(Json(
        service::update_with_scope(
            &state,
            id.zone_id,
            input,
            scope,
            crate::cameras::repository::CameraAccess::Partner {
                partner_id: current_user.partner_id,
            },
        )
        .await?,
    ))
}

#[debug_handler]
pub async fn delete(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::ZoneId>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::ZonesDelete)?;
    let scope = access(&current_user, ScopeType::Delete)?;
    service::delete_with_scope(&state, id.zone_id, scope).await?;
    Ok(StatusCode::NO_CONTENT)
}
