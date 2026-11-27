use axum::{Json, response::IntoResponse};
use chrono::NaiveDateTime;
use serde::Serialize;

use crate::{error::ApiError, http::permissions::GlobalRole};

use super::models::{MembershipRow, UserRow};

#[derive(Debug, Serialize)]
pub struct User {
    pub user_id: i32,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub global_role: Option<GlobalRole>,
    pub is_active: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl IntoResponse for User {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl TryFrom<UserRow> for User {
    type Error = ApiError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let global_role = row
            .global_role
            .map(|role| role.parse::<GlobalRole>())
            .transpose()
            .map_err(anyhow::Error::from)?;

        Ok(Self {
            user_id: row.user_id,
            email: row.email,
            full_name: row.full_name,
            phone: row.phone,
            global_role,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct PartnerMembership {
    pub partner_id: i32,
    pub role: String,
    pub is_active: bool,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String,
}

impl From<MembershipRow> for PartnerMembership {
    fn from(row: MembershipRow) -> Self {
        Self {
            partner_id: row.partner_id,
            role: row.user_role,
            is_active: row.is_active,
            read_scope: row.read_scope,
            write_scope: row.write_scope,
            delete_scope: row.delete_scope,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub user: User,
    pub partner_memberships: Vec<PartnerMembership>,
}

impl IntoResponse for UserProfile {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ListUsers {
    pub items: Vec<User>,
    pub total: i64,
    pub top: u64,
    pub offset: u64,
}

impl IntoResponse for ListUsers {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct CreatedUser {
    pub user_id: i32,
}

impl IntoResponse for CreatedUser {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
