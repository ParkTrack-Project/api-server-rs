use crate::http::validation::validate_phone;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn default_top() -> u64 {
    20
}
#[derive(Debug, Deserialize, Validate)]
pub struct ListPartners {
    pub q: Option<String>,
    pub is_active: Option<bool>,
    #[serde(default = "default_top")]
    #[validate(range(min = 1, max = 100))]
    pub top: u64,
    #[serde(default)]
    pub offset: u64,
}
#[derive(Debug, Deserialize, Validate)]
pub struct PartnerId {
    #[validate(range(min = 1))]
    pub partner_id: i32,
}
#[derive(Debug, Deserialize, Validate)]
pub struct MemberId {
    #[validate(range(min = 1))]
    pub partner_id: i32,
    #[validate(range(min = 1))]
    pub user_id: i32,
}
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePartner {
    #[serde(alias = "legal_name")]
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    #[validate(length(min = 2, max = 255), custom(function = "validate_slug"))]
    pub slug: String,
    #[validate(email, length(max = 255))]
    pub contact_email: String,
    #[validate(custom(function = "validate_phone"), length(max = 255))]
    pub contact_phone: String,
}
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePartner {
    #[serde(alias = "legal_name")]
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 2, max = 255), custom(function = "validate_slug"))]
    pub slug: Option<String>,
    #[validate(email, length(max = 255))]
    pub contact_email: Option<String>,
    #[validate(custom(function = "validate_phone"), length(max = 255))]
    pub contact_phone: Option<String>,
    pub is_active: Option<bool>,
}
fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if value
        .bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        Ok(())
    } else {
        Err(ValidationError::new("slug"))
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListMembers {
    pub q: Option<String>,
    pub is_active: Option<bool>,
    #[validate(length(min = 1, max = 50))]
    pub role: Option<String>,
    #[serde(default = "default_top")]
    #[validate(range(min = 1, max = 100))]
    pub top: u64,
    #[serde(default)]
    pub offset: u64,
}
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMember {
    #[validate(range(min = 1))]
    pub user_id: i32,
    #[validate(length(min = 1, max = 50))]
    pub role: String,
    #[validate(custom(function = "validate_permissions"))]
    pub permissions: Vec<String>,
    #[validate(custom(function = "validate_member_scope"))]
    pub read_scope: String,
    #[validate(custom(function = "validate_member_scope"))]
    pub write_scope: String,
    #[validate(custom(function = "validate_member_scope"))]
    pub delete_scope: String,
}
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMember {
    #[validate(length(min = 1, max = 50))]
    pub role: Option<String>,
    #[validate(custom(function = "validate_permissions"))]
    pub permissions: Option<Vec<String>>,
    #[validate(custom(function = "validate_member_scope"))]
    #[serde(default, deserialize_with = "deserialize_scope_update")]
    pub read_scope: Option<String>,
    #[validate(custom(function = "validate_member_scope"))]
    #[serde(default, deserialize_with = "deserialize_scope_update")]
    pub write_scope: Option<String>,
    #[validate(custom(function = "validate_member_scope"))]
    #[serde(default, deserialize_with = "deserialize_scope_update")]
    pub delete_scope: Option<String>,
    pub is_active: Option<bool>,
}
fn validate_permissions(values: &[String]) -> Result<(), ValidationError> {
    use crate::http::permissions::Permission;
    // Global administrative permissions must never be granted through a partner membership.
    if values
        .iter()
        .all(|p| !p.starts_with("admin.") && p.parse::<Permission>().is_ok())
        && values
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            == values.len()
    {
        Ok(())
    } else {
        Err(ValidationError::new("invalid permissions"))
    }
}
fn validate_member_scope(value: &str) -> Result<(), ValidationError> {
    if value.parse::<crate::http::permissions::Scope>().is_ok() {
        Ok(())
    } else {
        Err(ValidationError::new("scope"))
    }
}

// None means the field was omitted; an explicit null is not a scope.
fn deserialize_scope_update<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(Some)
}
