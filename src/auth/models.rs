use sqlx::prelude::FromRow;

use crate::users::models::Users::{self};

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