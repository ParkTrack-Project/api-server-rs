use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use constant_time_eq::constant_time_eq;

use crate::{
    error::{ApiError, ApiResult},
    features::{auth::repository::is_token_revoked, users::repository::find_identity},
    http::{
        permissions::{GlobalRole, Permission, Scope, ServicePermission},
        token::decode_token,
    },
    state::{ApiConfig, ApiState},
};

pub trait Authenticated {
    fn require(&self, permission: Permission) -> ApiResult<()>;
}

#[derive(Debug)]
pub struct UserIdentity {
    pub user_id: i32,
    pub global_role: GlobalRole,
}

#[derive(Debug)]
pub struct ServiceIdentity {
    pub service_id: ServiceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceId {
    CarDetector,
    MlPrediction,
}

#[derive(Debug)]
pub enum Principal {
    User(UserIdentity),
    Service(ServiceIdentity),
}

impl Authenticated for UserIdentity {
    fn require(&self, permission: Permission) -> ApiResult<()> {
        if self.global_role.get_permissions().contains(&permission) {
            Ok(())
        } else {
            Err(ApiError::Forbidden(permission))
        }
    }
}

impl ServiceIdentity {
    pub fn require(&self, permission: ServicePermission) -> ApiResult<()> {
        use ServicePermission::*;

        let allowed = match self.service_id {
            ServiceId::CarDetector => matches!(
                permission,
                DetectionClaim | ZonesView | ZoneOccupancyWrite | OccupancyWrite
            ),
            ServiceId::MlPrediction => matches!(
                permission,
                CamerasView
                    | ZonesView
                    | OccupancyView
                    | WeatherView
                    | WeatherWrite
                    | ForecastsWrite
            ),
        };

        if allowed {
            Ok(())
        } else {
            Err(ApiError::ForbiddenService(permission))
        }
    }
}

impl Authenticated for Principal {
    fn require(&self, permission: Permission) -> ApiResult<()> {
        match (self, permission) {
            (Self::User(user), permission) => user.require(permission),
            (Self::Service(service), Permission::CamerasView) => {
                service.require(ServicePermission::CamerasView)
            }
            (Self::Service(service), Permission::ZonesView) => {
                service.require(ServicePermission::ZonesView)
            }
            (Self::Service(_), permission) => Err(ApiError::Forbidden(permission)),
        }
    }
}

fn bearer(parts: &Parts) -> ApiResult<Authorization<Bearer>> {
    parts
        .headers
        .typed_get::<Authorization<Bearer>>()
        .ok_or(ApiError::Unauthenticated)
}

fn authenticate_service(token: &str, config: &ApiConfig) -> ApiResult<Option<ServiceIdentity>> {
    let credentials = [
        (ServiceId::CarDetector, &config.car_detector_token),
        (ServiceId::MlPrediction, &config.ml_prediction_token),
    ];
    let mut matching_services = credentials
        .into_iter()
        .filter_map(|(service_id, configured)| {
            (!configured.is_empty() && constant_time_eq(configured.as_bytes(), token.as_bytes()))
                .then_some(ServiceIdentity { service_id })
        });

    let identity = matching_services.next();
    if matching_services.next().is_some() {
        return Err(ApiError::InvalidToken);
    }

    Ok(identity)
}

async fn authenticate_user(token: &str, state: &ApiState) -> ApiResult<UserIdentity> {
    let claims = decode_token(token, &state.decoding_key)?.claims;

    if is_token_revoked(&state.pool, token).await? {
        return Err(ApiError::InvalidToken);
    }

    let user_id = i32::try_from(claims.sub).map_err(|_| ApiError::InvalidToken)?;
    let identity = find_identity(&state.pool, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user".into()))?;

    if !identity.is_active {
        return Err(ApiError::AccessDenied);
    }

    if identity.global_role != claims.role.as_ref() {
        return Err(ApiError::InvalidToken);
    }

    Ok(UserIdentity {
        user_id,
        global_role: claims.role,
    })
}

pub struct AuthenticatedUser(pub UserIdentity);

impl FromRequestParts<ApiState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &ApiState) -> ApiResult<Self> {
        let bearer = bearer(parts)?;
        let token = bearer.token();
        if authenticate_service(token, &state.config)?.is_some() {
            return Err(ApiError::InvalidToken);
        }
        authenticate_user(token, state).await.map(Self)
    }
}

pub struct AuthenticatedService(pub ServiceIdentity);

impl FromRequestParts<ApiState> for AuthenticatedService {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &ApiState) -> ApiResult<Self> {
        let bearer = bearer(parts)?;
        authenticate_service(bearer.token(), &state.config)?
            .map(Self)
            .ok_or(ApiError::InvalidToken)
    }
}

pub struct AuthenticatedPrincipal(pub Principal);

impl FromRequestParts<ApiState> for AuthenticatedPrincipal {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &ApiState) -> ApiResult<Self> {
        let bearer = bearer(parts)?;
        let token = bearer.token();
        let principal = match authenticate_service(token, &state.config)? {
            Some(service) => Principal::Service(service),
            None => Principal::User(authenticate_user(token, state).await?),
        };
        Ok(Self(principal))
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
        let AuthenticatedUser(UserIdentity {
            user_id,
            global_role,
        }) = AuthenticatedUser::from_request_parts(parts, state).await?;
        let axum::extract::Path(path) =
            axum::extract::Path::<PartnerId>::from_request_parts(parts, state)
                .await
                .map_err(|_| ApiError::BadRequest("invalid partner id".into()))?;

        if path.partner_id <= 0 {
            return Err(ApiError::BadRequest("invalid partner id".into()));
        }

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
