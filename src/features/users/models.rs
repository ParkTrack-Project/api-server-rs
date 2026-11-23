use chrono::{DateTime, Utc};
use sea_query::Iden;
use sqlx::prelude::FromRow;

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

#[derive(FromRow)]
pub struct UserRow {
    pub user_id: i32,
    pub full_name: String,
    pub email: String,
    pub hashed_password: String,
    pub phone: String,
    pub global_role: String,
    pub crated_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}
