use anyhow::Context;
use sea_query::{
    Alias, Cond, Expr, ExprTrait, Order, PostgresQueryBuilder, Query, SelectStatement,
    extension::postgres::PgExpr,
};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

use crate::{
    error::{ApiError, ApiResult},
    features::partners::models::PartnerMemberships,
    http::permissions::GlobalRole,
};

use super::{
    models::{MEMBERSHIP_COLUMNS, MembershipRow, USER_COLUMNS, UserIdentityRow, UserRow, Users},
    requests::{CreateUser, ListUsers, UpdateUser},
};

#[derive(Debug, Clone, Copy)]
pub enum UserAccess {
    All,
    Only { user_id: i32 },
}

fn condition(access: UserAccess) -> Cond {
    match access {
        UserAccess::All => Cond::all(),
        UserAccess::Only { user_id } => Cond::all().add(Expr::col(Users::UserId).eq(user_id)),
    }
}

fn user_projection() -> [Expr; USER_COLUMNS.len()] {
    USER_COLUMNS.map(|column| match column {
        Users::GlobalRole => Expr::col(Users::GlobalRole).cast_as(Alias::new("text")),
        column => Expr::col(column),
    })
}

fn pg_role(role: GlobalRole) -> Expr {
    Expr::val(role.as_ref()).cast_as(Alias::new("global_roles"))
}

fn filtered(query: &ListUsers, access: UserAccess) -> SelectStatement {
    Query::select()
        .from(Users::Table)
        .cond_where(condition(access))
        .apply_if(query.q.as_deref(), |stmt, q| {
            let pattern = format!("%{q}%");
            stmt.cond_where(
                Cond::any()
                    .add(Expr::col(Users::Email).ilike(pattern.clone()))
                    .add(Expr::col(Users::FullName).ilike(pattern)),
            );
        })
        .apply_if(query.role, |stmt, role| {
            stmt.and_where(Expr::col(Users::GlobalRole).eq(pg_role(role)));
        })
        .apply_if(query.is_active, |stmt, is_active| {
            stmt.and_where(Expr::col(Users::IsActive).eq(is_active));
        })
        .to_owned()
}

pub async fn list_users(
    pool: impl PgExecutor<'_>,
    query: &ListUsers,
    access: UserAccess,
) -> ApiResult<Vec<UserRow>> {
    let (sql, values) = filtered(query, access)
        .exprs(user_projection())
        .order_by(Users::UserId, Order::Asc)
        .limit(query.top)
        .offset(query.offset)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .context("listing users")
        .map_err(ApiError::from)
}

pub async fn count_users(
    pool: impl PgExecutor<'_>,
    query: &ListUsers,
    access: UserAccess,
) -> ApiResult<i64> {
    let (sql, values) = filtered(query, access)
        .expr(Expr::col(Users::UserId).count())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .context("counting users")
        .map_err(ApiError::from)
}

pub async fn find_user(
    pool: impl PgExecutor<'_>,
    user_id: i32,
    access: UserAccess,
) -> ApiResult<Option<UserRow>> {
    let (sql, values) = Query::select()
        .from(Users::Table)
        .exprs(user_projection())
        .cond_where(condition(access))
        .and_where(Expr::col(Users::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .context("fetching user profile")
        .map_err(ApiError::from)
}

pub(crate) async fn find_identity(
    pool: impl PgExecutor<'_>,
    user_id: i32,
) -> ApiResult<Option<UserIdentityRow>> {
    let (sql, values) = Query::select()
        .from(Users::Table)
        .column(Users::IsActive)
        .expr_as(
            Expr::col(Users::GlobalRole).cast_as(Alias::new("text")),
            Users::GlobalRole,
        )
        .and_where(Expr::col(Users::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .context("fetching user identity")
        .map_err(ApiError::from)
}

pub async fn list_memberships(
    pool: impl PgExecutor<'_>,
    user_id: i32,
    access: UserAccess,
) -> ApiResult<Vec<MembershipRow>> {
    let (sql, values) = Query::select()
        .from(PartnerMemberships::Table)
        .columns(MEMBERSHIP_COLUMNS)
        .cond_where(match access {
            UserAccess::All => Cond::all(),
            UserAccess::Only { user_id } => {
                Cond::all().add(Expr::col(PartnerMemberships::UserId).eq(user_id))
            }
        })
        .and_where(Expr::col(PartnerMemberships::UserId).eq(user_id))
        .and_where(Expr::col(PartnerMemberships::PartnerId).is_not_null())
        .order_by(PartnerMemberships::PartnerId, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .context("fetching user memberships")
        .map_err(ApiError::from)
}

pub async fn create_user(
    pool: impl PgExecutor<'_>,
    input: CreateUser,
    hashed_password: String,
) -> ApiResult<i32> {
    let (sql, values) = Query::insert()
        .into_table(Users::Table)
        .columns([
            Users::Email,
            Users::FullName,
            Users::Phone,
            Users::HashedPassword,
            Users::GlobalRole,
            Users::IsActive,
        ])
        .values_panic([
            input.email.into(),
            input.full_name.unwrap_or_default().into(),
            input.phone.into(),
            hashed_password.into(),
            pg_role(input.global_role),
            input.is_active.into(),
        ])
        .returning_col(Users::UserId)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(write_error)
}

pub async fn update_user(
    pool: impl PgExecutor<'_>,
    user_id: i32,
    input: UpdateUser,
    access: UserAccess,
) -> ApiResult<Option<UserRow>> {
    let (sql, values) = Query::update()
        .table(Users::Table)
        .cond_where(condition(access))
        .and_where(Expr::col(Users::UserId).eq(user_id))
        .values(
            [
                input.full_name.map(|value| (Users::FullName, value.into())),
                input.phone.map(|value| (Users::Phone, value.into())),
                input.email.map(|value| (Users::Email, value.into())),
                input
                    .global_role
                    .map(|value| (Users::GlobalRole, pg_role(value))),
                input.is_active.map(|value| (Users::IsActive, value.into())),
            ]
            .into_iter()
            .flatten(),
        )
        .value(Users::UpdatedAt, Expr::current_timestamp())
        .returning(Query::returning().exprs(user_projection()))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(write_error)
}

pub async fn find_password_hash(
    pool: impl PgExecutor<'_>,
    access: UserAccess,
) -> ApiResult<Option<String>> {
    let (sql, values) = Query::select()
        .from(Users::Table)
        .column(Users::HashedPassword)
        .cond_where(condition(access))
        .and_where(Expr::col(Users::IsActive).eq(true))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .context("fetching password credentials")
        .map_err(ApiError::from)
}

pub async fn update_password_if_current(
    pool: impl PgExecutor<'_>,
    access: UserAccess,
    expected_hash: &str,
    new_hash: String,
) -> ApiResult<bool> {
    let (sql, values) = Query::update()
        .table(Users::Table)
        .cond_where(condition(access))
        .and_where(Expr::col(Users::IsActive).eq(true))
        .and_where(Expr::col(Users::HashedPassword).eq(expected_hash))
        .value(Users::HashedPassword, new_hash)
        .value(Users::UpdatedAt, Expr::current_timestamp())
        .build_sqlx(PostgresQueryBuilder);

    let result = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .context("changing user password")?;
    Ok(result.rows_affected() == 1)
}

fn write_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_unique_violation()
    {
        return ApiError::Conflict("email is already taken".into());
    }
    ApiError::from(anyhow::Error::from(error).context("writing user"))
}
