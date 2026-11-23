use super::{requests, responses, service};
use crate::{
    error::ApiResult,
    http::{
        authorization::{Authenticated, AuthenticatedUserOrAdmin, PublicAuthenticated},
        permissions::{GlobalRole, Permission},
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
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListPartners>>,
) -> ApiResult<Json<responses::Page<responses::Partner>>> {
    if matches!(
        current_user,
        PublicAuthenticated::User {
            global_role: GlobalRole::Admin,
            ..
        }
    ) {
        current_user.require(Permission::AdminPartnersView)?;
    }
    Ok(Json(service::list(&state, &current_user, input).await?))
}
#[debug_handler]
pub async fn get(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::PartnerId>>,
) -> ApiResult<Json<responses::Partner>> {
    if matches!(
        current_user,
        PublicAuthenticated::User {
            global_role: GlobalRole::Admin,
            ..
        }
    ) {
        current_user.require(Permission::AdminPartnersView)?;
    }
    Ok(Json(
        service::get(&state, &current_user, id.partner_id).await?,
    ))
}
#[debug_handler]
pub async fn create(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreatePartner>>,
) -> ApiResult<(StatusCode, Json<responses::Created>)> {
    current_user.require(Permission::AdminPartnersManage)?;
    Ok((
        StatusCode::CREATED,
        Json(service::create(&state, input).await?),
    ))
}
#[debug_handler]
pub async fn update(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::PartnerId>>,
    Valid(Json(input)): Valid<Json<requests::UpdatePartner>>,
) -> ApiResult<Json<responses::Partner>> {
    current_user.require(Permission::AdminPartnersManage)?;
    Ok(Json(
        service::update(&state, &current_user, id.partner_id, input).await?,
    ))
}
#[debug_handler]
pub async fn disable(
    AuthenticatedUserOrAdmin(current_user): AuthenticatedUserOrAdmin,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::PartnerId>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::AdminPartnersManage)?;
    service::disable(&state, &current_user, id.partner_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

use crate::http::authorization::PartnerContext;
#[debug_handler]
pub async fn list_members(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Query(input)): Valid<Query<requests::ListMembers>>,
) -> ApiResult<Json<responses::Page<responses::Member>>> {
    current_user.require(Permission::PartnerMembersView)?;
    Ok(Json(
        service::list_members(&state, &current_user, input).await?,
    ))
}
#[debug_handler]
pub async fn get_member(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::MemberId>>,
) -> ApiResult<Json<responses::Member>> {
    current_user.require(Permission::PartnerMembersView)?;
    Ok(Json(
        service::get_member(&state, &current_user, id.user_id).await?,
    ))
}
#[debug_handler]
pub async fn create_member(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Json(input)): Valid<Json<requests::CreateMember>>,
) -> ApiResult<(StatusCode, Json<responses::Member>)> {
    current_user.require(Permission::PartnerMembersWrite)?;
    Ok((
        StatusCode::CREATED,
        Json(service::create_member(&state, &current_user, input).await?),
    ))
}
#[debug_handler]
pub async fn update_member(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::MemberId>>,
    Valid(Json(input)): Valid<Json<requests::UpdateMember>>,
) -> ApiResult<Json<responses::Member>> {
    current_user.require(Permission::PartnerMembersWrite)?;
    Ok(Json(
        service::update_member(&state, &current_user, id.user_id, input).await?,
    ))
}
#[debug_handler]
pub async fn disable_member(
    current_user: PartnerContext,
    State(state): State<ApiState>,
    Valid(Path(id)): Valid<Path<requests::MemberId>>,
) -> ApiResult<StatusCode> {
    current_user.require(Permission::PartnerMembersDisable)?;
    service::disable_member(&state, &current_user, id.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
