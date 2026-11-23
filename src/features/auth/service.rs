use crate::error::ApiError;
use crate::error::ApiResult;
use crate::features::auth::responses::Access;
use crate::features::auth::responses::User;
use crate::http::permissions::ADMIN_PERMISSIONS;
use crate::http::permissions::GlobalRole;
use crate::http::permissions::USER_PERMISSIONS;
use crate::http::token::create_access_token;
use crate::state::ApiState;
use crate::utils::hasher;
use crate::utils::hasher::verify_password;
use crate::utils::hasher::verify_password_reset_token;

use super::repository;
use super::requests;
use super::responses;

#[tracing::instrument(skip(state, payload), err)]
pub async fn register(
    state: &ApiState,
    payload: requests::Register,
) -> ApiResult<responses::Access> {
    let user = repository::create_user(&state.pool, payload)
        .await
        .map(responses::User::from)?;

    let token = create_access_token(
        user.user_id,
        state.config.jwt_expire_seconds,
        user.global_role,
        &state.config.jwt_secret,
    );

    Ok(Access {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expire_seconds,
        user,
    })
}

#[tracing::instrument(skip(state, payload), err)]
pub async fn login(state: &ApiState, payload: requests::Login) -> ApiResult<responses::Access> {
    let creds = repository::find_creds_by_email(&state.pool, &payload.login).await?;

    let Some(creds) = creds else {
        return Err(ApiError::InvalidCredentials);
    };

    if !verify_password(&payload.password, &creds.hashed_password)? {
        return Err(ApiError::InvalidCredentials);
    }

    if !creds.is_active {
        return Err(ApiError::AccountDisabled);
    }

    let user = User {
        user_id: creds.user_id as u32,
        email: payload.login,
        full_name: creds.full_name,
        global_role: creds
            .global_role
            .parse()
            .expect("global_role should be valid"),
    };

    let token = create_access_token(
        user.user_id,
        state.config.jwt_expire_seconds,
        user.global_role,
        &state.config.jwt_secret,
    );

    Ok(Access {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expire_seconds,
        user,
    })
}

#[tracing::instrument(skip(state, payload), err)]
pub async fn request_password_reset(
    state: &ApiState,
    payload: requests::PasswordReset,
) -> ApiResult<()> {
    let Some(user) = repository::find_creds_by_email(&state.pool, &payload.email).await? else {
        return Ok(());
    };

    let token = hasher::generate_password_reset_token();

    let expires_at = chrono::Local::now().fixed_offset()
        + chrono::Duration::minutes(state.config.password_reset_ttl_minutes.into());

    repository::create_password_reset_token(&state.pool, user.user_id, &token, expires_at).await?;

    state
        .email_service
        .send_password_reset_email(&payload.email, &token, &state.config)
        .await?;

    Ok(())
}

#[tracing::instrument(skip(state, payload), err)]
pub async fn confirm_password_reset(
    state: &ApiState,
    payload: requests::PasswordResetConfirm,
) -> ApiResult<()> {
    let token = repository::find_password_reset_token_by_email(&state.pool, &payload.email)
        .await?
        .ok_or(ApiError::NotFound("password reset token".into()))?;

    let is_used = token.used_at.is_some();
    let is_expired = token.expires_at <= chrono::Utc::now().fixed_offset();

    if is_used || is_expired {
        return Err(ApiError::TokenExpired);
    }

    if !verify_password_reset_token(&payload.token, &token.token_hash) {
        return Err(ApiError::InvalidToken);
    }

    let mut transaction = state.pool.begin().await.map_err(anyhow::Error::from)?;
    let consumed = repository::consume_password_reset_token(
        &mut *transaction,
        token.user_id,
        &token.token_hash,
    )
    .await?;
    if !consumed {
        return Err(ApiError::TokenExpired);
    }
    repository::update_user_password(&mut *transaction, token.user_id, &payload.new_password)
        .await?;
    transaction.commit().await.map_err(anyhow::Error::from)?;
    Ok(())
}

#[tracing::instrument(skip(state), err)]
pub async fn me(state: &ApiState, user_id: i32) -> ApiResult<responses::Me> {
    let user = repository::find_by_user_id(&state.pool, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("user of id {} not found", user_id)))
        .map(responses::User::from)?;

    let partner_memberships =
        repository::find_partner_memberships_by_user_id(&state.pool, user_id).await?;

    let mut permissions = repository::find_permissions_by_user_id_group_by_partner(
        &state.pool,
        user_id,
        partner_memberships.iter().map(|pm| pm.partner_id),
    )
    .await?;

    let partner_memberships: Vec<responses::PartnerMembership> = partner_memberships
        .into_iter()
        .map(|pm| responses::PartnerMembership {
            partner_id: pm.partner_id as u32,
            role: pm.user_role,
            permissions: permissions.remove(&pm.partner_id).unwrap_or_default(),
            read_scope: pm.read_scope,
            write_scope: pm.write_scope,
            delete_scope: pm.delete_scope,
        })
        .collect();

    let permissions = if user.global_role == GlobalRole::User {
        USER_PERMISSIONS.iter().map(|p| p.to_string()).collect()
    } else {
        ADMIN_PERMISSIONS.iter().map(|p| p.to_string()).collect()
    };

    Ok(responses::Me {
        user,
        permissions,
        partner_memberships,
    })
}

#[tracing::instrument(skip(state, token), err)]
pub async fn logout(state: &ApiState, token: &str) -> ApiResult<()> {
    let claims = crate::http::token::decode_token(token, &state.decoding_key)?.claims;

    let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
        .ok_or(ApiError::InvalidToken)?
        + chrono::Duration::seconds(20);

    repository::revoke_token(
        &state.pool,
        claims.sub as i32,
        token,
        expires_at.naive_utc(),
    )
    .await
}
