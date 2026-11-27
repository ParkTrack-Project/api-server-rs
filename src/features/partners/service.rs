use super::{
    repository::{self, PartnerAccess},
    requests, responses,
};
use crate::{
    error::{ApiError, ApiResult},
    http::{authorization::PublicAuthenticated, permissions::GlobalRole},
    state::ApiState,
};

pub fn access(user: &PublicAuthenticated) -> PartnerAccess {
    match user {
        PublicAuthenticated::User {
            global_role: GlobalRole::User,
            user_id,
        } => PartnerAccess::Member { user_id: *user_id },
        _ => PartnerAccess::All,
    }
}

#[tracing::instrument(skip(state), err)]
pub async fn list(
    state: &ApiState,
    user: &PublicAuthenticated,
    input: requests::ListPartners,
) -> ApiResult<responses::Page<responses::Partner>> {
    let scope = access(user);
    let total = repository::count(&state.pool, &input, scope).await?;
    let items = repository::list_partners(&state.pool, &input, scope).await?;
    Ok(responses::Page {
        items: items.into_iter().map(Into::into).collect(),
        total,
        top: input.top,
        offset: input.offset,
    })
}

#[tracing::instrument(skip(state), err)]
pub async fn get(
    state: &ApiState,
    user: &PublicAuthenticated,
    id: i32,
) -> ApiResult<responses::Partner> {
    repository::find(&state.pool, id, access(user))
        .await?
        .ok_or_else(|| ApiError::NotFound("partner".into()))
        .map(Into::into)
}

#[tracing::instrument(skip(state), err)]
pub async fn create(
    state: &ApiState,
    input: requests::CreatePartner,
) -> ApiResult<responses::Created> {
    Ok(responses::Created {
        partner_id: repository::create_partner(&state.pool, input).await?,
    })
}

#[tracing::instrument(skip(state), err)]
pub async fn update(
    state: &ApiState,
    user: &PublicAuthenticated,
    id: i32,
    input: requests::UpdatePartner,
) -> ApiResult<responses::Partner> {
    if input.name.is_none()
        && input.slug.is_none()
        && input.contact_email.is_none()
        && input.contact_phone.is_none()
        && input.is_active.is_none()
    {
        return Err(ApiError::BadRequest("missing body".into()));
    }
    repository::update_partner(&state.pool, id, input, access(user))
        .await?
        .ok_or_else(|| ApiError::NotFound("partner".into()))
        .map(Into::into)
}

#[tracing::instrument(skip(state), err)]
pub async fn disable(state: &ApiState, user: &PublicAuthenticated, id: i32) -> ApiResult<()> {
    update(
        state,
        user,
        id,
        requests::UpdatePartner {
            name: None,
            slug: None,
            contact_email: None,
            contact_phone: None,
            is_active: Some(false),
        },
    )
    .await?;
    Ok(())
}

use super::member_repository as members;
use crate::http::authorization::PartnerContext;

fn member_missing() -> ApiError {
    ApiError::NotFound("partner membership".into())
}

async fn ensure_partner(state: &ApiState, partner_id: i32) -> ApiResult<()> {
    repository::find(&state.pool, partner_id, PartnerAccess::All)
        .await?
        .ok_or_else(|| ApiError::NotFound("partner".into()))?;
    Ok(())
}

#[tracing::instrument(skip(state), err)]
pub async fn list_members(
    state: &ApiState,
    current_user: &PartnerContext,
    input: requests::ListMembers,
) -> ApiResult<responses::Page<responses::Member>> {
    ensure_partner(state, current_user.partner_id).await?;
    let scope = current_user.scope();
    let total = members::count(&state.pool, scope, &input).await?;
    let rows = members::list_members(&state.pool, scope, &input).await?;
    let mut permissions =
        members::permissions(&state.pool, scope, rows.iter().map(|r| r.user_id).collect()).await?;
    let items = rows
        .into_iter()
        .map(|row| {
            let granted = permissions.remove(&row.user_id).unwrap_or_default();
            responses::Member::from_row(row, granted)
        })
        .collect();
    Ok(responses::Page {
        items,
        total,
        top: input.top,
        offset: input.offset,
    })
}

#[tracing::instrument(skip(state), err)]
pub async fn get_member(
    state: &ApiState,
    current_user: &PartnerContext,
    user_id: i32,
) -> ApiResult<responses::Member> {
    ensure_partner(state, current_user.partner_id).await?;
    let row = members::find(&state.pool, current_user.scope(), user_id)
        .await?
        .ok_or_else(member_missing)?;
    let mut permissions =
        members::permissions(&state.pool, current_user.scope(), vec![user_id]).await?;
    Ok(responses::Member::from_row(
        row,
        permissions.remove(&user_id).unwrap_or_default(),
    ))
}

#[tracing::instrument(skip(state), err)]
pub async fn create_member(
    state: &ApiState,
    current_user: &PartnerContext,
    input: requests::CreateMember,
) -> ApiResult<responses::Member> {
    ensure_partner(state, current_user.partner_id).await?;
    
    let scope = current_user.scope();
    let mut transaction = state.pool.begin().await.map_err(anyhow::Error::from)?;
    let row = members::create_member(&mut *transaction, scope, &input).await?;
    members::insert_permissions(
        &mut *transaction,
        scope,
        input.user_id,
        current_user.user_id,
        &input.permissions,
    )
    .await?;
    transaction.commit().await.map_err(anyhow::Error::from)?;
    let mut permissions = input.permissions;
    permissions.sort();
    Ok(responses::Member::from_row(row, permissions))
}

#[tracing::instrument(skip(state), err)]
pub async fn update_member(
    state: &ApiState,
    current_user: &PartnerContext,
    user_id: i32,
    input: requests::UpdateMember,
) -> ApiResult<responses::Member> {
    if input.role.is_none()
        && input.permissions.is_none()
        && input.read_scope.is_none()
        && input.write_scope.is_none()
        && input.delete_scope.is_none()
        && input.is_active.is_none()
    {
        return Err(ApiError::BadRequest("missing body".into()));
    }
    let scope = current_user.scope();
    let mut transaction = state.pool.begin().await.map_err(anyhow::Error::from)?;
    let row = members::update_member(&mut *transaction, scope, user_id, &input)
        .await?
        .ok_or_else(member_missing)?;
    if let Some(granted) = &input.permissions {
        members::clear_permissions(&mut *transaction, scope, user_id).await?;
        members::insert_permissions(
            &mut *transaction,
            scope,
            user_id,
            current_user.user_id,
            granted,
        )
        .await?;
    }
    let mut permissions = members::permissions(&mut *transaction, scope, vec![user_id]).await?;
    transaction.commit().await.map_err(anyhow::Error::from)?;
    Ok(responses::Member::from_row(
        row,
        permissions.remove(&user_id).unwrap_or_default(),
    ))
}

#[tracing::instrument(skip(state), err)]
pub async fn disable_member(
    state: &ApiState,
    current_user: &PartnerContext,
    user_id: i32,
) -> ApiResult<()> {
    let input = requests::UpdateMember {
        role: None,
        permissions: None,
        read_scope: None,
        write_scope: None,
        delete_scope: None,
        is_active: Some(false),
    };
    members::update_member(&state.pool, current_user.scope(), user_id, &input)
        .await?
        .ok_or_else(member_missing)?;
    Ok(())
}
