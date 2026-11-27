use serde::Deserialize;
use validator::Validate;

use crate::http::{permissions::GlobalRole, validation::validate_phone};

const MAX_OFFSET: u64 = i64::MAX as u64;

#[derive(Debug, Validate, Deserialize)]
pub struct UserId {
    #[validate(range(min = 1))]
    pub user_id: i32,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ListUsers {
    pub q: Option<String>,
    pub role: Option<GlobalRole>,
    pub is_active: Option<bool>,
    #[serde(default = "default_top")]
    #[validate(range(min = 1, max = 100))]
    pub top: u64,
    #[serde(default)]
    #[validate(range(max = "MAX_OFFSET"))]
    pub offset: u64,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateProfile {
    #[validate(length(min = 1, max = 200))]
    pub full_name: Option<String>,
    #[validate(length(max = 50), custom(function = "validate_phone"))]
    pub phone: Option<String>,
}

#[derive(Validate, Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    #[validate(length(min = 6, message = "Password should be at least 6 characters long"))]
    pub new_password: String,
}

#[derive(Validate, Deserialize)]
pub struct CreateUser {
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 6, message = "Password should be at least 6 characters long"))]
    pub password: String,
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
    #[validate(length(max = 50), custom(function = "validate_phone"))]
    pub phone: Option<String>,
    #[serde(default = "default_role")]
    pub global_role: GlobalRole,
    #[serde(default = "default_active")]
    pub is_active: bool,
}

#[derive(Debug, Default, Validate, Deserialize)]
pub struct UpdateUser {
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
    #[validate(length(max = 50), custom(function = "validate_phone"))]
    pub phone: Option<String>,
    #[validate(email, length(max = 255))]
    pub email: Option<String>,
    pub global_role: Option<GlobalRole>,
    pub is_active: Option<bool>,
}

impl UpdateUser {
    pub fn is_empty(&self) -> bool {
        self.full_name.is_none()
            && self.phone.is_none()
            && self.email.is_none()
            && self.global_role.is_none()
            && self.is_active.is_none()
    }
}

fn default_top() -> u64 {
    20
}

fn default_role() -> GlobalRole {
    GlobalRole::User
}

fn default_active() -> bool {
    true
}
