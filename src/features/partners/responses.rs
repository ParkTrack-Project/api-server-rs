use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Partner {
    pub partner_id: i32,
    pub name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub is_active: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
#[derive(Debug, Serialize)]
pub struct Created {
    pub partner_id: i32,
}
#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub top: u64,
    pub offset: u64,
}

impl From<super::models::PartnerRow> for Partner {
    fn from(row: super::models::PartnerRow) -> Self {
        Self {
            partner_id: row.partner_id,
            name: row.legal_name,
            slug: row.slug,
            contact_email: row.contact_email,
            contact_phone: row.contact_phone,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Member {
    pub partner_id: i32,
    pub user_id: i32,
    pub role: String,
    pub permissions: Vec<String>,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String,
    pub is_active: bool,
    pub created_at: Option<NaiveDateTime>,
}
impl Member {
    pub fn from_row(row: super::models::MemberRow, permissions: Vec<String>) -> Self {
        Self {
            partner_id: row.partner_id,
            user_id: row.user_id,
            role: row.user_role,
            permissions,
            read_scope: row.read_scope,
            write_scope: row.write_scope,
            delete_scope: row.delete_scope,
            is_active: row.is_active,
            created_at: row.created_at,
        }
    }
}
