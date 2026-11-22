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
