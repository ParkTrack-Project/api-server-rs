
use std::hash::Hash;

use anyhow::Context;
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

use crate::{auth::models::{LOGIN_USER_COLUMNS, LoginUserRow, ShortUserRow}, error::{ApiError, ApiResult}, http::{hasher::hash_password, permissions::GlobalRole}, users::models::Users};

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

pub async fn find_by_email(
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
        .with_context(|| format!("fetching user with email {}", email))
        .map_err(ApiError::from)
}