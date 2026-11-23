use serde::Deserialize;
use validator::Validate;

use crate::http::{permissions::GlobalRole, validation::validate_phone};

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUser {
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
    #[validate(custom(function = "validate_phone"))]
    pub phone: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub global_role: Option<GlobalRole>,
    pub is_active: Option<bool>,
}
