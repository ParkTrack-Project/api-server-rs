use serde::Deserialize;
use validator::Validate;

use super::super::validation::validate_phone;

#[derive(Debug, Validate, Deserialize)]
pub struct Register {
    #[validate(email)]
    email: String,
    #[validate(length(min = 6, message = "Password should be at least 6 characters long"))]
    password: Option<String>,
    #[validate(length(max = 255))]
    full_name: Option<String>,
    #[validate(custom(function = "validate_phone"))]
    phone: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Login {
    login: String,
    password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct PasswordReset {
    #[validate(email)]
    email: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct PasswordResetConfirm {
    token: String,
    #[validate(length(min = 6, message = "Password should be at least 6 characters long"))]
    new_password: String,
}
