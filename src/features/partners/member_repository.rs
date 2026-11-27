use super::{
    models::{
        MEMBER_COLUMNS, MemberPermissionRow, MemberRow, PartnerMemberPermissions as G,
        PartnerMemberships as M, Partners as P,
    },
    repository::db_error,
    requests,
};
use crate::{error::ApiResult, features::users::models::Users as U};
use sea_query::{Cond, Expr, ExprTrait, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum MemberAccess {
    Partner { partner_id: i32 },
}
impl MemberAccess {
    pub fn partner_id(self) -> i32 {
        match self {
            Self::Partner { partner_id } => partner_id,
        }
    }
}

fn base(scope: MemberAccess) -> sea_query::SelectStatement {
    Query::select()
        .from(M::Table)
        .and_where(Expr::col((M::Table, M::PartnerId)).eq(scope.partner_id()))
        .to_owned()
}

pub async fn find(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    user_id: i32,
) -> ApiResult<Option<MemberRow>> {
    let (sql, values) = base(scope)
        .columns(MEMBER_COLUMNS)
        .and_where(Expr::col(M::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}

// Reads identity and activity only; authorization decisions belong to the extractor/handler.
pub async fn active_identity(
    pool: impl PgExecutor<'_>,
    partner_id: i32,
    user_id: i32,
) -> ApiResult<Option<MemberRow>> {
    let (sql, values) = base(MemberAccess::Partner { partner_id })
        .columns(MEMBER_COLUMNS.map(|col| (M::Table, col)))
        .inner_join(
            P::Table,
            Expr::col((P::Table, P::PartnerId)).equals((M::Table, M::PartnerId)),
        )
        .inner_join(
            U::Table,
            Expr::col((U::Table, U::UserId)).equals((M::Table, M::UserId)),
        )
        .and_where(Expr::col((M::Table, M::UserId)).eq(user_id))
        .and_where(Expr::col((M::Table, M::IsActive)).eq(true))
        .and_where(Expr::col((P::Table, P::IsActive)).eq(true))
        .and_where(Expr::col((U::Table, U::IsActive)).eq(true))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}

fn filtered(scope: MemberAccess, input: &requests::ListMembers) -> sea_query::SelectStatement {
    let mut q = base(scope);
    q.inner_join(
        U::Table,
        Expr::col((U::Table, U::UserId)).equals((M::Table, M::UserId)),
    );
    q.apply_if(input.q.as_deref(), |q, v| {
        q.cond_where(
            Cond::any()
                .add(Expr::col((U::Table, U::Email)).like(format!("%{v}%")))
                .add(Expr::col((U::Table, U::FullName)).like(format!("%{v}%"))),
        );
    });
    q.apply_if(input.is_active, |q, v| {
        q.and_where(Expr::col((M::Table, M::IsActive)).eq(v));
    });
    q.apply_if(input.role.as_deref(), |q, v| {
        q.and_where(Expr::col(M::UserRole).eq(v));
    });
    q
}

pub async fn list_members(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    input: &requests::ListMembers,
) -> ApiResult<Vec<MemberRow>> {
    let (sql, values) = filtered(scope, input)
        .columns(MEMBER_COLUMNS.map(|col| (M::Table, col)))
        .order_by((M::Table, M::UserId), sea_query::Order::Asc)
        .limit(input.top)
        .offset(input.offset)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .map_err(db_error)
}

pub async fn count(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    input: &requests::ListMembers,
) -> ApiResult<i64> {
    let (sql, values) = filtered(scope, input)
        .expr(Expr::col(M::PartnerMembershipId).count())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}
pub async fn permissions(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    users: Vec<i32>,
) -> ApiResult<HashMap<i32, Vec<String>>> {
    if users.is_empty() {
        return Ok(HashMap::new());
    }

    let (sql, values) = Query::select()
        .columns([G::UserId, G::Permission])
        .from(G::Table)
        .and_where(Expr::col(G::PartnerId).eq(scope.partner_id()))
        .and_where(Expr::col(G::UserId).is_in(users))
        .order_by(G::Permission, sea_query::Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let rows = sqlx::query_as_with::<_, MemberPermissionRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

    let mut grouped: HashMap<i32, Vec<String>> = HashMap::new();
    for row in rows {
        grouped.entry(row.user_id).or_default().push(row.permission);
    }
    
    Ok(grouped)
}
pub async fn create_member(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    input: &requests::CreateMember,
) -> ApiResult<MemberRow> {
    let (sql, values) = Query::insert()
        .into_table(M::Table)
        .columns([
            M::PartnerId,
            M::UserId,
            M::UserRole,
            M::ReadScope,
            M::WriteScope,
            M::DeleteScope,
        ])
        .values_panic([
            scope.partner_id().into(),
            input.user_id.into(),
            input.role.clone().into(),
            input.read_scope.clone().into(),
            input.write_scope.clone().into(),
            input.delete_scope.clone().into(),
        ])
        .returning(Query::returning().columns(MEMBER_COLUMNS))
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}
pub async fn update_member(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    user_id: i32,
    input: &requests::UpdateMember,
) -> ApiResult<Option<MemberRow>> {
    let mut q = Query::update();
    q.table(M::Table)
        .and_where(Expr::col(M::PartnerId).eq(scope.partner_id()))
        .and_where(Expr::col(M::UserId).eq(user_id));
    q.values(
        [
            input.role.clone().map(|v| (M::UserRole, v.into())),
            input.read_scope.clone().map(|v| (M::ReadScope, v.into())),
            input.write_scope.clone().map(|v| (M::WriteScope, v.into())),
            input
                .delete_scope
                .clone()
                .map(|v| (M::DeleteScope, v.into())),
        ]
        .into_iter()
        .flatten(),
    );
    // Also takes a row lock for permission-only replacements.
    q.value(
        M::IsActive,
        input
            .is_active
            .map_or_else(|| Expr::col(M::IsActive), Expr::val),
    );
    let (sql, values) = q
        .returning(Query::returning().columns(MEMBER_COLUMNS))
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}
pub async fn clear_permissions(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    user_id: i32,
) -> ApiResult<()> {
    let (sql, values) = Query::delete()
        .from_table(G::Table)
        .and_where(Expr::col(G::PartnerId).eq(scope.partner_id()))
        .and_where(Expr::col(G::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .map_err(db_error)?;
    Ok(())
}
pub async fn insert_permissions(
    pool: impl PgExecutor<'_>,
    scope: MemberAccess,
    user_id: i32,
    granted_by: i32,
    permissions: &[String],
) -> ApiResult<()> {
    if permissions.is_empty() {
        return Ok(());
    }
    let mut q = Query::insert();
    q.into_table(G::Table)
        .columns([G::PartnerId, G::UserId, G::Permission, G::GrantedBy]);
    for permission in permissions {
        q.values_panic([
            scope.partner_id().into(),
            user_id.into(),
            permission.clone().into(),
            granted_by.into(),
        ]);
    }
    let (sql, values) = q.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .map_err(db_error)?;
    Ok(())
}
