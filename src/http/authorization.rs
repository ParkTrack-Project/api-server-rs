use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use constant_time_eq::constant_time_eq;
use serde::{Deserialize, Serialize};

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
    fn require(&self, permissions: &[Permission]) -> ApiResult<()>;
}

pub enum PublicAuthenticated {
    User {
        user_id: i32,
        global_role: GlobalRole,
    },
    ApiToken,
}

impl Authenticated for PublicAuthenticated {
    fn require(&self, permissions: &[Permission]) -> ApiResult<()> {
        match &self {
            Self::User { global_role, .. } => {
                let effective_permissions = global_role.get_permissions();
                let permissions: Vec<Permission> = permissions
                    .iter()
                    .filter(|p| !effective_permissions.contains(p))
                    .copied()
                    .collect();

                if permissions.is_empty() {
                    return Ok(());
                }

                return Err(ApiError::Forbidden(permissions));
            }
            Self::ApiToken => {
                return Ok(());
            }
        }
    }
}

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
    fn require(&self, permissions: &[Permission]) -> ApiResult<()> {
        match &self {
            Self::Admin { .. } => {
                let effective_permissions = GlobalRole::Admin.get_permissions();
                let permissions: Vec<Permission> = permissions
                    .iter()
                    .filter(|p| !effective_permissions.contains(p))
                    .copied()
                    .collect();

                if permissions.is_empty() {
                    return Ok(());
                }

                return Err(ApiError::Forbidden(permissions));
            }
            Self::Member {
                permissions: effective_permissions,
                ..
            } => {
                let permissions: Vec<Permission> = permissions
                    .iter()
                    .filter(|p| !effective_permissions.contains(p))
                    .copied()
                    .collect();

                if permissions.is_empty() {
                    return Ok(());
                }

                return Err(ApiError::Forbidden(permissions));
            }
        }
    }
}

impl PartnerAuthenticated {
    fn view_scope(&self) -> Scope {
        match &self {
            PartnerAuthenticated::Admin { .. } => Scope::PartnerAll,
            PartnerAuthenticated::Member { view_scope, .. } => *view_scope,
        }
    }

    fn write_scope(&self) -> Scope {
        match &self {
            PartnerAuthenticated::Admin { .. } => Scope::PartnerAll,
            PartnerAuthenticated::Member { write_scope, .. } => *write_scope,
        }
    }

    fn delete_scope(&self) -> Scope {
        match &self {
            PartnerAuthenticated::Admin { .. } => Scope::PartnerAll,
            PartnerAuthenticated::Member { delete_scope, .. } => *delete_scope,
        }
    }
}

pub struct AuthenticatedOrApiToken(pub PublicAuthenticated);

impl FromRequestParts<ApiState> for AuthenticatedOrApiToken {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .typed_get::<Authorization<Bearer>>()
            .ok_or(ApiError::Unauthenticated)?
            .token()
            .to_string();

        if constant_time_eq(state.config.api_token.as_bytes(), token.as_bytes()) {
            return Ok(AuthenticatedOrApiToken(PublicAuthenticated::ApiToken));
        }

        let token_data = decode_token(&token, &state.decoding_key)?;

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
        let token = parts
            .headers
            .typed_get::<Authorization<Bearer>>()
            .ok_or(ApiError::Unauthenticated)?
            .token()
            .to_string();

        let token_data = decode_token(&token, &state.decoding_key)?;

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
        let token = parts
            .headers
            .typed_get::<Authorization<Bearer>>()
            .ok_or(ApiError::Unauthenticated)?
            .token()
            .to_string();

        if constant_time_eq(state.config.api_token.as_bytes(), token.as_bytes()) {
            return Ok(ApiToken);
        }

        Err(Unauthenticated)
    }
}
