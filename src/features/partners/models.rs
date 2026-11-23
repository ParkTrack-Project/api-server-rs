use sea_query::Iden;

#[derive(Iden)]
pub enum Partners {
    Table,
    PartnerId,
    LegalName,
    Slug,
    ContactEmail,
    ContactPhone,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum PartnerMemberships {
    Table,
    PartnerMembershipId,
    UserId,
    PartnerId,
    UserRole,
    ReadScope,
    WriteScope,
    DeleteScope,
    IsActive,
    CreatedAt,
}

#[derive(Iden)]
pub enum PartnerMemberPermissions {
    Table,
    UserId,
    PartnerId,
    Permission,
    GrantedAt,
    GrantedBy,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PartnerRow {
    pub partner_id: i32,
    pub legal_name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub is_active: Option<bool>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MemberRow {
    pub partner_id: i32,
    pub user_id: i32,
    pub user_role: String,
    pub read_scope: String,
    pub write_scope: String,
    pub delete_scope: String,
    pub is_active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MemberPermissionRow {
    pub user_id: i32,
    pub permission: String,
}

pub const PARTNER_COLUMNS: [Partners; 8] = [
    Partners::PartnerId,
    Partners::LegalName,
    Partners::Slug,
    Partners::ContactEmail,
    Partners::ContactPhone,
    Partners::IsActive,
    Partners::CreatedAt,
    Partners::UpdatedAt,
];

pub const MEMBER_COLUMNS: [PartnerMemberships; 9] = [
    PartnerMemberships::PartnerId,
    PartnerMemberships::UserId,
    PartnerMemberships::UserRole,
    PartnerMemberships::ReadScope,
    PartnerMemberships::WriteScope,
    PartnerMemberships::DeleteScope,
    PartnerMemberships::IsActive,
    PartnerMemberships::CreatedAt,
    PartnerMemberships::PartnerMembershipId,
];
