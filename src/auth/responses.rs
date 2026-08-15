use axum::{Json, response::IntoResponse};
use serde::Serialize;

use crate::{auth::models::{self, ShortUserRow}, http::permissions::GlobalRole};

#[derive(Debug, Serialize)]
pub struct PasswordResetRequest {
    ok: bool,
    reset_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetConfirm {
    ok: bool,
}

#[derive(Debug, Serialize)]
pub struct PartnerMembershipInfo {
    partner_id: u32,
    role: String,
    permissions: Vec<String>,
    read_scope: String,
    write_scope: String,
    delete_scope: String,
    is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthUserInfo {
    user_id: u32,
    email: String,
    full_name: Option<String>,
    global_role: String,
    permissions: Vec<String>,
    partner_memberships: Vec<PartnerMembershipInfo>,
}

#[derive(Debug, Serialize)]
pub struct Token {
    access_token: String,
    token_type: String,
    expires_in: u32,
    user: AuthUserInfo,
}

#[derive(Debug, Serialize)]
pub struct Me {
    user_id: u32,
    email: String,
    full_name: Option<String>,
    global_role: String,
    permissions: Vec<String>,
    partner_memberships: Vec<PartnerMembershipInfo>,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub user_id: u32,
    pub email: String,
    pub full_name: Option<String>,
    pub global_role: GlobalRole
}

impl From<ShortUserRow> for User {
    fn from(row: ShortUserRow) -> Self {
        Self {
            user_id: row.user_id as u32,
            email: row.email,
            full_name: row.full_name,
            global_role: row.global_role.parse().expect("global_role should be valid")
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Access {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: User
}

impl IntoResponse for Access {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}