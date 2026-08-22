
use std::{collections::HashMap, hash::Hash};

use anyhow::Context;
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

use crate::{auth::models::{LOGIN_USER_COLUMNS, LoginUserRow, PARTNER_MEMBERSHIP_COLUMNS, PartnerMembershipRow, PartnerPermissionRow, SHORT_USER_COLUMNS, ShortUserRow}, error::{ApiError, ApiResult}, http::{hasher::hash_password, permissions::GlobalRole}, partners::models::{PartnerMemberPermissions, PartnerMemberships}, users::models::Users};

use super::requests;

pub async fn create_user(
    pool: impl PgExecutor<'_>,
    query: requests::Register
) -> ApiResult<ShortUserRow> {
    let (sql, values) = sea_query::Query::insert()
        .into_table(Users::Table)
        .columns([
            Users::Email,
            Users::HashedPassword,
            Users::Phone,
            Users::GlobalRole,
            Users::IsActive
        ])
        .values_panic([
            query.email.clone().into(),
            hash_password(&query.password)?.into(),
            query.phone.into(),
            GlobalRole::User.as_ref().into(),
            true.into()
        ])
        .returning_col(Users::UserId)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, ShortUserRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return ApiError::Conflict("user already exists".to_string());
                }
            }

            return ApiError::InternalError(
                anyhow::Error::from(e)
                    .context(format!("creating new user with email {}", query.email)),
            );
        })
}

pub async fn find_creds_by_email(
    pool: impl PgExecutor<'_>,
    email: &str
) -> ApiResult<Option<LoginUserRow>> {
    let (sql, values) = sea_query::Query::select()
        .from(Users::Table)
        .columns(LOGIN_USER_COLUMNS)
        .and_where(Expr::col(Users::Email).eq(email))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, LoginUserRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("fetching user creds with email {}", email))
        .map_err(ApiError::from)
}

pub async fn find_by_user_id(
    pool: impl PgExecutor<'_>,
    user_id: i32
) -> ApiResult<Option<ShortUserRow>> {
    let (sql, values) = sea_query::Query::select()
        .from(Users::Table)
        .columns(SHORT_USER_COLUMNS)
        .and_where(Expr::col(Users::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, ShortUserRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("fetching user with id {user_id}"))
        .map_err(ApiError::from)
}

pub async fn find_partner_memberships_by_user_id(
    pool: impl PgExecutor<'_>,
    user_id: i32
) -> ApiResult<Vec<PartnerMembershipRow>> {
    let (sql, values) = sea_query::Query::select()
        .from(PartnerMemberships::Table)
        .columns(PARTNER_MEMBERSHIP_COLUMNS)
        .and_where(Expr::col(PartnerMemberships::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, PartnerMembershipRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .with_context(|| format!("fetching partner memberships of user with id {user_id}"))
        .map_err(ApiError::from)
}

pub async fn find_permissions_by_user_id_group_by_partner(
    pool: impl PgExecutor<'_>,
    user_id: i32,
    partner_ids: impl Iterator<Item = i32>
) -> ApiResult<HashMap<i32, Vec<String>>> {
    let (sql, values) = sea_query::Query::select()
        .from(PartnerMemberPermissions::Table)
        .columns([
            PartnerMemberPermissions::PartnerId,
            PartnerMemberPermissions::Permission
        ])
        .and_where(Expr::col(PartnerMemberPermissions::UserId).eq(user_id))
        .and_where(Expr::col(PartnerMemberPermissions::PartnerId).is_in(partner_ids))
        .build_sqlx(PostgresQueryBuilder);

    let rows = sqlx::query_as_with::<_, PartnerPermissionRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .with_context(|| format!("fetching partner permissions of user with id {user_id}"))
        .map_err(ApiError::from)?;

    let rows_arranged: HashMap<i32, Vec<String>> = rows.into_iter()
        .fold(HashMap::new(), |mut acc, item| {
        acc.entry(item.partner_id).or_default().push(item.permission);
        acc
    });

    Ok(rows_arranged)
}