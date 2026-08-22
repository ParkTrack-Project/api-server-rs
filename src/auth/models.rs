use sqlx::prelude::FromRow;

use crate::{partners::models::{PartnerMemberPermissions, PartnerMemberships}, users::models::Users::{self}};

#[derive(Debug, FromRow)]
pub struct ShortUserRow {
    pub user_id: i32,
    pub email: String,
    pub full_name: Option<String>,
    pub global_role: String
}

pub const SHORT_USER_COLUMNS: [Users; 4] = [
    Users::UserId,
    Users::Email,
    Users::FullName,
    Users::GlobalRole
];

#[derive(Debug, FromRow)]
pub struct LoginUserRow {
    pub user_id: i32,
    pub full_name: Option<String>,
    pub global_role: String,
    pub hashed_password: String,
    pub is_active: bool
}

pub const LOGIN_USER_COLUMNS: [Users; 5] = [
    Users::UserId,
    Users::FullName,
    Users::GlobalRole,
    Users::HashedPassword,
    Users::IsActive
];

#[derive(Debug, FromRow)]
pub struct PartnerMembershipRow {
    pub partner_id: i32,
    pub user_role: String,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String
}

pub const PARTNER_MEMBERSHIP_COLUMNS: [PartnerMemberships; 5] = [
    PartnerMemberships::PartnerId,
    PartnerMemberships::UserRole,
    PartnerMemberships::ReadScope,
    PartnerMemberships::WriteScope,
    PartnerMemberships::DeleteScope
];

#[derive(Debug, FromRow)]
pub struct PartnerPermissionRow {
    pub partner_id: i32,
    pub permission: String
}

pub const PARTNER_PERMISSION_COLUMNS: [PartnerMemberPermissions; 2] = [
    PartnerMemberPermissions::PartnerId,
    PartnerMemberPermissions::UserId
];