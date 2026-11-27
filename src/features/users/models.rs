use chrono::NaiveDateTime;
use sea_query::Iden;
use sqlx::FromRow;

use crate::features::partners::models::PartnerMemberships;

#[derive(Iden)]
pub enum Users {
    Table,
    UserId,
    FullName,
    Email,
    HashedPassword,
    Phone,
    GlobalRole,
    CreatedAt,
    UpdatedAt,
    IsActive,
}

pub const USER_COLUMNS: [Users; 8] = [
    Users::UserId,
    Users::Email,
    Users::FullName,
    Users::Phone,
    Users::GlobalRole,
    Users::IsActive,
    Users::CreatedAt,
    Users::UpdatedAt,
];

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub user_id: i32,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub global_role: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub(crate) struct UserIdentityRow {
    pub global_role: String,
    pub is_active: bool,
}

pub const MEMBERSHIP_COLUMNS: [PartnerMemberships; 6] = [
    PartnerMemberships::PartnerId,
    PartnerMemberships::UserRole,
    PartnerMemberships::IsActive,
    PartnerMemberships::ReadScope,
    PartnerMemberships::WriteScope,
    PartnerMemberships::DeleteScope,
];

#[derive(Debug, FromRow)]
pub struct MembershipRow {
    pub partner_id: i32,
    pub user_role: String,
    pub is_active: bool,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String,
}
