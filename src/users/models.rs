use sea_query::Iden;

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
    IsActive
}