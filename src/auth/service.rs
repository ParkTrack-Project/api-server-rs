use crate::auth::responses::Access;
use crate::auth::responses::User;
use crate::error::ApiError;
use crate::error::ApiResult;
use crate::http::hasher::hash_password;
use crate::http::hasher::verify_password;
use crate::http::token::create_access_token;
use crate::state::ApiState;

use super::repository;
use super::requests;
use super::responses;

pub async fn register(
    state: &ApiState,
    payload: requests::Register
) -> ApiResult<responses::Access> {
    let user = repository::create_user(&state.pool, payload)
        .await
        .map(responses::User::from)?;

    let token = create_access_token(
        user.user_id,
        state.config.jwt_expire_seconds,
        user.global_role,
        &state.config.jwt_secret
    );

    Ok(Access {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expire_seconds,
        user
    })
}

pub async fn login(
    state: &ApiState,
    payload: requests::Login
) -> ApiResult<responses::Access> {
    let creds = repository::find_by_email(&state.pool, &payload.login).await?;

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
        global_role: creds.global_role.parse().expect("global_role should be valid")
    };

    let token = create_access_token(
        user.user_id,
        state.config.jwt_expire_seconds,
        user.global_role,
        &state.config.jwt_secret
    );

    Ok(Access {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expire_seconds,
        user
    })
}