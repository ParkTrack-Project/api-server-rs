use serde::Deserialize;
use validator::Validate;

use crate::http::validation::validate_phone;

#[derive(Debug, Validate, Deserialize)]
pub struct Register {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, message = "Password should be at least 6 characters long"))]
    pub password: String,
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
    #[validate(custom(function = "validate_phone"))]
    pub phone: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Login {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct PasswordReset {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct PasswordResetConfirm {
    #[validate(email)]
    pub email: String,
    pub token: String,
    #[validate(length(min = 6, message = "Password should be at least 6 characters long"))]
    pub new_password: String,
}
