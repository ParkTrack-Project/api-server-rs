use chrono::{DateTime, FixedOffset};
use sea_query::Iden;
use sqlx::prelude::FromRow;

use crate::{
    partners::models::{PartnerMemberPermissions, PartnerMemberships},
    users::models::Users::{self},
};

#[derive(Debug, FromRow)]
pub struct ShortUserRow {
    pub user_id: i32,
    pub email: String,
    pub full_name: Option<String>,
    pub global_role: String,
}

pub const SHORT_USER_COLUMNS: [Users; 4] = [
    Users::UserId,
    Users::Email,
    Users::FullName,
    Users::GlobalRole,
];

#[derive(Debug, FromRow)]
pub struct LoginUserRow {
    pub user_id: i32,
    pub full_name: Option<String>,
    pub global_role: String,
    pub hashed_password: String,
    pub is_active: bool,
}

pub const LOGIN_USER_COLUMNS: [Users; 5] = [
    Users::UserId,
    Users::FullName,
    Users::GlobalRole,
    Users::HashedPassword,
    Users::IsActive,
];

#[derive(Debug, FromRow)]
pub struct PartnerMembershipRow {
    pub partner_id: i32,
    pub user_role: String,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String,
}

pub const PARTNER_MEMBERSHIP_COLUMNS: [PartnerMemberships; 5] = [
    PartnerMemberships::PartnerId,
    PartnerMemberships::UserRole,
    PartnerMemberships::ReadScope,
    PartnerMemberships::WriteScope,
    PartnerMemberships::DeleteScope,
];

#[derive(Debug, FromRow)]
pub struct PartnerPermissionRow {
    pub partner_id: i32,
    pub permission: String,
}

pub const PARTNER_PERMISSION_COLUMNS: [PartnerMemberPermissions; 2] = [
    PartnerMemberPermissions::PartnerId,
    PartnerMemberPermissions::UserId,
];

#[derive(Iden)]
pub enum PasswordResetTokens {
    Table,
    TokenId,
    UserId,
    TokenHash,
    UsedAt,
    ExpiresAt,
    CreatedAt,
}

#[derive(Debug, FromRow)]
pub struct PasswordResetTokenRow {
    pub token_id: i32,
    pub user_id: i32,
    pub token_hash: String,
    pub used_at: DateTime<FixedOffset>,
    pub expires_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
#[derive(Debug, FromRow)]
pub struct ShortPasswordResetTokenRow {
    pub user_id: i32,
    pub token_hash: String,
    pub used_at: Option<DateTime<FixedOffset>>,
    pub expires_at: DateTime<FixedOffset>,
}

pub const SHORT_PASSWORD_RESET_TOKEN_COLUMNS: [PasswordResetTokens; 4] = [
    PasswordResetTokens::UserId,
    PasswordResetTokens::TokenHash,
    PasswordResetTokens::UsedAt,
    PasswordResetTokens::ExpiresAt,
];

#[derive(Iden)]
pub enum Sessions {
    Table,
    UserId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
}
