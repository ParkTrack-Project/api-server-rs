use crate::{
    error::{ApiError, ApiResult},
    state::ApiState,
    utils::hasher::{hash_password, verify_password},
};

use super::{
    repository::{self, UserAccess},
    requests, responses,
};

fn user_not_found() -> ApiError {
    ApiError::NotFound("user".into())
}

#[tracing::instrument(skip(state), err)]
pub async fn list_users(
    state: &ApiState,
    input: requests::ListUsers,
    access: UserAccess,
) -> ApiResult<responses::ListUsers> {
    let rows = repository::list_users(&state.pool, &input, access).await?;
    let total = repository::count_users(&state.pool, &input, access).await?;

    Ok(responses::ListUsers {
        items: rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<ApiResult<_>>()?,
        total,
        top: input.top,
        offset: input.offset,
    })
}

#[tracing::instrument(skip(state), err)]
pub async fn get_profile(
    state: &ApiState,
    user_id: i32,
    access: UserAccess,
) -> ApiResult<responses::UserProfile> {
    let user = repository::find_user(&state.pool, user_id, access)
        .await?
        .ok_or_else(user_not_found)?
        .try_into()?;
    let partner_memberships = repository::list_memberships(&state.pool, user_id, access)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(responses::UserProfile {
        user,
        partner_memberships,
    })
}

#[tracing::instrument(skip(state, input), err)]
pub async fn create_user(
    state: &ApiState,
    mut input: requests::CreateUser,
) -> ApiResult<responses::CreatedUser> {
    let password = std::mem::take(&mut input.password);
    let hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(anyhow::Error::from)??;
    let user_id = repository::create_user(&state.pool, input, hash).await?;
    Ok(responses::CreatedUser { user_id })
}

#[tracing::instrument(skip(state, input), err)]
pub async fn update_user(
    state: &ApiState,
    user_id: i32,
    input: requests::UpdateUser,
    access: UserAccess,
) -> ApiResult<responses::User> {
    if input.is_empty() {
        return Err(ApiError::BadRequest("no profile fields to update".into()));
    }

    repository::update_user(&state.pool, user_id, input, access)
        .await?
        .ok_or_else(user_not_found)?
        .try_into()
}

#[tracing::instrument(skip(state), err)]
pub async fn deactivate_user(state: &ApiState, user_id: i32, access: UserAccess) -> ApiResult<()> {
    update_user(
        state,
        user_id,
        requests::UpdateUser {
            is_active: Some(false),
            ..Default::default()
        },
        access,
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip(state, input), err)]
pub async fn change_password(
    state: &ApiState,
    user_id: i32,
    input: requests::ChangePassword,
) -> ApiResult<()> {
    let access = UserAccess::Only { user_id };
    let expected_hash = repository::find_password_hash(&state.pool, access)
        .await?
        .ok_or_else(user_not_found)?;

    let (expected_hash, new_hash) = tokio::task::spawn_blocking(move || {
        if !verify_password(&input.current_password, &expected_hash)? {
            return Err(ApiError::AccessDenied);
        }
        Ok::<_, ApiError>((expected_hash, hash_password(&input.new_password)?))
    })
    .await
    .map_err(anyhow::Error::from)??;

    if !repository::update_password_if_current(&state.pool, access, &expected_hash, new_hash)
        .await?
    {
        return Err(ApiError::AccessDenied);
    }
    Ok(())
}
