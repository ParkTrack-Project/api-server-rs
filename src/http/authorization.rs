use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use constant_time_eq::constant_time_eq;

use crate::{
    error::{
        ApiError::{self, Unauthenticated},
        ApiResult,
    },
    http::{
        permissions::{GlobalRole, Permission, Scope},
        token::decode_token,
    },
    state::ApiState,
};

pub trait Authenticated {
    fn require(&self, permission: Permission) -> ApiResult<()>;
}

#[derive(Debug)]
pub enum PublicAuthenticated {
    User {
        user_id: i32,
        global_role: GlobalRole,
    },
    ApiToken,
}

impl Authenticated for PublicAuthenticated {
    fn require(&self, permission: Permission) -> ApiResult<()> {
        match &self {
            Self::User { global_role, .. } => {
                let effective_permissions = global_role.get_permissions();

                return match effective_permissions.contains(&permission) {
                    true => Ok(()),
                    false => Err(ApiError::Forbidden(permission)),
                };
            }
            Self::ApiToken => {
                return Ok(());
            }
        }
    }
}

#[derive(Debug)]
pub enum PartnerAuthenticated {
    Admin {
        user_id: i32,
    },
    Member {
        user_id: i32,
        permissions: Vec<Permission>,
        view_scope: Scope,
        write_scope: Scope,
        delete_scope: Scope,
    },
}

impl Authenticated for PartnerAuthenticated {
    fn require(&self, permission: Permission) -> ApiResult<()> {
        match &self {
            Self::Admin { .. } => {
                let effective_permissions = GlobalRole::Admin.get_permissions();

                return match effective_permissions.contains(&permission) {
                    true => Ok(()),
                    false => Err(ApiError::Forbidden(permission)),
                };
            }
            Self::Member {
                permissions: effective_permissions,
                ..
            } => {
                return match effective_permissions.contains(&permission) {
                    true => Ok(()),
                    false => Err(ApiError::Forbidden(permission)),
                };
            }
        }
    }
}

fn get_token_string(parts: &mut Parts) -> ApiResult<String> {
    let parts = parts
        .headers
        .typed_get::<Authorization<Bearer>>()
        .ok_or(ApiError::Unauthenticated)?
        .token()
        .to_string();

    Ok(parts)
}

pub struct AuthenticatedOrApiToken(pub PublicAuthenticated);

impl FromRequestParts<ApiState> for AuthenticatedOrApiToken {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let token = get_token_string(parts)?;

        if constant_time_eq(state.config.api_token.as_bytes(), token.as_bytes()) {
            return Ok(AuthenticatedOrApiToken(PublicAuthenticated::ApiToken));
        }

        let token_data = decode_token(&token, &state.decoding_key)?;
        if crate::auth::repository::is_token_revoked(&state.pool, &token).await? {
            return Err(ApiError::InvalidToken);
        }

        Ok(AuthenticatedOrApiToken(PublicAuthenticated::User {
            user_id: token_data.claims.sub as i32,
            global_role: token_data.claims.role,
        }))
    }
}

pub struct AuthenticatedAdminOrPartner(pub PartnerAuthenticated);

impl FromRequestParts<ApiState> for AuthenticatedAdminOrPartner {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let token = get_token_string(parts)?;

        let token_data = decode_token(&token, &state.decoding_key)?;
        if crate::auth::repository::is_token_revoked(&state.pool, &token).await? {
            return Err(ApiError::InvalidToken);
        }

        if token_data.claims.role == GlobalRole::Admin {
            return Ok(AuthenticatedAdminOrPartner(PartnerAuthenticated::Admin {
                user_id: token_data.claims.sub as i32,
            }));
        }

        todo!()
    }
}

pub struct ApiToken;

impl FromRequestParts<ApiState> for ApiToken {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let token = get_token_string(parts)?;

        if constant_time_eq(state.config.api_token.as_bytes(), token.as_bytes()) {
            return Ok(ApiToken);
        }

        Err(Unauthenticated)
    }
}

pub struct AuthenticatedUserOrAdmin(pub PublicAuthenticated);

impl FromRequestParts<ApiState> for AuthenticatedUserOrAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let token = get_token_string(parts)?;

        let token_data = decode_token(&token, &state.decoding_key)?;
        if crate::auth::repository::is_token_revoked(&state.pool, &token).await? {
            return Err(ApiError::InvalidToken);
        }

        Ok(AuthenticatedUserOrAdmin(PublicAuthenticated::User {
            user_id: token_data.claims.sub as i32,
            global_role: token_data.claims.role,
        }))
    }
}
