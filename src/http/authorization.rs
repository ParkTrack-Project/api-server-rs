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
        if crate::features::auth::repository::is_token_revoked(&state.pool, &token).await? {
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
        if crate::features::auth::repository::is_token_revoked(&state.pool, &token).await? {
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
        if crate::features::auth::repository::is_token_revoked(&state.pool, &token).await? {
            return Err(ApiError::InvalidToken);
        }

        Ok(AuthenticatedUserOrAdmin(PublicAuthenticated::User {
            user_id: token_data.claims.sub as i32,
            global_role: token_data.claims.role,
        }))
    }
}

/// Authenticated identity in one partner. Membership permissions never become global permissions.
#[derive(Debug)]
pub struct PartnerContext {
    pub user_id: i32,
    pub partner_id: i32,
    pub membership: Option<crate::features::partners::models::MemberRow>,
    permissions: Vec<Permission>,
    is_admin: bool,
}
impl PartnerContext {
    pub fn resource_scope(
        &self,
        operation: crate::http::permissions::ScopeType,
    ) -> ApiResult<Scope> {
        if self.is_admin {
            return Ok(Scope::PartnerAll);
        }
        let membership = self.membership.as_ref().ok_or(ApiError::AccessDenied)?;
        let value = match operation {
            crate::http::permissions::ScopeType::View => &membership.read_scope,
            crate::http::permissions::ScopeType::Write => &membership.write_scope,
            crate::http::permissions::ScopeType::Delete => &membership.delete_scope,
        };
        match value.parse::<Scope>() {
            Ok(Scope::Own) => Ok(Scope::Own),
            Ok(Scope::PartnerAll) => Ok(Scope::PartnerAll),
            _ => Err(ApiError::AccessDenied),
        }
    }
    pub fn scope(&self) -> crate::features::partners::member_repository::MemberAccess {
        crate::features::partners::member_repository::MemberAccess::Partner {
            partner_id: self.partner_id,
        }
    }
}
impl Authenticated for PartnerContext {
    fn require(&self, permission: Permission) -> ApiResult<()> {
        let allowed = if self.is_admin {
            GlobalRole::Admin.get_permissions().contains(&permission)
        } else {
            self.permissions.contains(&permission)
        };
        if allowed {
            Ok(())
        } else {
            Err(ApiError::Forbidden(permission))
        }
    }
}
impl FromRequestParts<ApiState> for PartnerContext {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &ApiState) -> ApiResult<Self> {
        use crate::features::partners::{member_repository as repository, requests::PartnerId};
        let AuthenticatedUserOrAdmin(identity) =
            AuthenticatedUserOrAdmin::from_request_parts(parts, state).await?;
        let axum::extract::Path(path) =
            axum::extract::Path::<PartnerId>::from_request_parts(parts, state)
                .await
                .map_err(|_| ApiError::BadRequest("invalid partner id".into()))?;

        if path.partner_id <= 0 {
            return Err(ApiError::BadRequest("invalid partner id".into()));
        }

        let PublicAuthenticated::User {
            user_id,
            global_role,
        } = identity
        else {
            return Err(ApiError::Unauthenticated);
        };

        if global_role == GlobalRole::Admin {
            crate::features::partners::repository::find(
                &state.pool,
                path.partner_id,
                crate::features::partners::repository::PartnerAccess::All,
            )
            .await?
            .ok_or_else(|| ApiError::NotFound("partner".into()))?;
            return Ok(Self {
                user_id,
                partner_id: path.partner_id,
                membership: None,
                permissions: Vec::new(),
                is_admin: true,
            });
        }
        let membership = repository::active_identity(&state.pool, path.partner_id, user_id)
            .await?
            .ok_or(ApiError::AccessDenied)?;
        let scope = repository::MemberAccess::Partner {
            partner_id: path.partner_id,
        };
        let mut granted = repository::permissions(&state.pool, scope, vec![user_id]).await?;
        let permissions = granted
            .remove(&user_id)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| !p.starts_with("admin."))
            .filter_map(|p| p.parse::<Permission>().ok())
            .collect();
        Ok(Self {
            user_id,
            partner_id: path.partner_id,
            membership: Some(membership),
            permissions,
            is_admin: false,
        })
    }
}
