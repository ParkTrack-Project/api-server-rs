use axum::{Json, response::IntoResponse};
use serde::Serialize;

use crate::{features::auth::models::ShortUserRow, http::permissions::GlobalRole};

#[derive(Debug, Serialize)]
pub struct PasswordResetRequest {
    reset_token: Option<String>,
}

impl IntoResponse for PasswordResetRequest {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct PasswordResetConfirm {
    ok: bool,
}

impl IntoResponse for PasswordResetConfirm {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct PartnerMembership {
    pub partner_id: u32,
    pub role: String,
    pub permissions: Vec<String>,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub user_id: u32,
    pub email: String,
    pub full_name: Option<String>,
    pub global_role: GlobalRole,
}

impl From<ShortUserRow> for User {
    fn from(row: ShortUserRow) -> Self {
        Self {
            user_id: row.user_id as u32,
            email: row.email,
            full_name: row.full_name,
            global_role: row
                .global_role
                .parse()
                .expect("global_role should be valid"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Me {
    #[serde(flatten)]
    pub user: User,
    pub permissions: Vec<String>,
    pub partner_memberships: Vec<PartnerMembership>,
}

impl IntoResponse for Me {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

// register + login
#[derive(Debug, Serialize)]
pub struct Access {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: User,
}

impl IntoResponse for Access {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
