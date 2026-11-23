use super::{
    models::{PARTNER_COLUMNS, PartnerMemberships as M, Partners as P},
    requests,
};
use crate::error::{ApiError, ApiResult};
use sea_query::{Cond, Expr, ExprTrait, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

#[derive(Debug, Clone, Copy)]
pub enum PartnerAccess {
    All,
    Member { user_id: i32 },
}
fn condition(scope: PartnerAccess) -> Cond {
    match scope {
        PartnerAccess::All => Cond::all(),
        PartnerAccess::Member { user_id } => Cond::all().add(
            Expr::col(P::PartnerId).in_subquery(
                Query::select()
                    .column(M::PartnerId)
                    .from(M::Table)
                    .and_where(Expr::col(M::UserId).eq(user_id))
                    .and_where(Expr::col(M::IsActive).eq(true))
                    .to_owned(),
            ),
        ),
    }
}
fn base(scope: PartnerAccess) -> sea_query::SelectStatement {
    Query::select()
        .from(P::Table)
        .cond_where(condition(scope))
        .to_owned()
}

fn filtered(input: &requests::ListPartners, scope: PartnerAccess) -> sea_query::SelectStatement {
    let mut q = base(scope);
    q.apply_if(input.q.as_deref(), |q, v| {
        q.cond_where(
            Cond::any()
                .add(Expr::col(P::LegalName).like(format!("%{v}%")))
                .add(Expr::col(P::Slug).like(format!("%{v}%"))),
        );
    });
    q.apply_if(input.is_active, |q, v| {
        q.and_where(Expr::col(P::IsActive).eq(v));
    });
    q
}
pub(super) fn db_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(e) = &error {
        if e.is_unique_violation() {
            return ApiError::Conflict("partner or membership already exists".into());
        }
        if e.is_foreign_key_violation() {
            return ApiError::NotFound("partner or user".into());
        }
    }
    ApiError::from(anyhow::Error::from(error).context("accessing partners"))
}
pub async fn list_partners(
    pool: impl PgExecutor<'_>,
    input: &requests::ListPartners,
    scope: PartnerAccess,
) -> ApiResult<Vec<super::models::PartnerRow>> {
    let (sql, values) = filtered(input, scope)
        .columns(PARTNER_COLUMNS)
        .order_by(P::PartnerId, sea_query::Order::Asc)
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
    input: &requests::ListPartners,
    scope: PartnerAccess,
) -> ApiResult<i64> {
    let (sql, values) = filtered(input, scope)
        .expr(Expr::col(P::PartnerId).count())
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}
pub async fn find(
    pool: impl PgExecutor<'_>,
    id: i32,
    scope: PartnerAccess,
) -> ApiResult<Option<super::models::PartnerRow>> {
    let (sql, values) = base(scope)
        .columns(PARTNER_COLUMNS)
        .and_where(Expr::col(P::PartnerId).eq(id))
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}
pub async fn create_partner(
    pool: impl PgExecutor<'_>,
    input: requests::CreatePartner,
) -> ApiResult<i32> {
    let (sql, values) = Query::insert()
        .into_table(P::Table)
        .columns([P::LegalName, P::Slug, P::ContactEmail, P::ContactPhone])
        .values_panic([
            input.name.into(),
            input.slug.into(),
            input.contact_email.into(),
            input.contact_phone.into(),
        ])
        .returning_col(P::PartnerId)
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}
pub async fn update_partner(
    pool: impl PgExecutor<'_>,
    id: i32,
    input: requests::UpdatePartner,
    scope: PartnerAccess,
) -> ApiResult<Option<super::models::PartnerRow>> {
    let mut q = Query::update();
    q.table(P::Table)
        .cond_where(condition(scope))
        .and_where(Expr::col(P::PartnerId).eq(id));
    q.values(
        [
            input.name.map(|v| (P::LegalName, v.into())),
            input.slug.map(|v| (P::Slug, v.into())),
            input.contact_email.map(|v| (P::ContactEmail, v.into())),
            input.contact_phone.map(|v| (P::ContactPhone, v.into())),
            input.is_active.map(|v| (P::IsActive, v.into())),
        ]
        .into_iter()
        .flatten(),
    );
    q.value(P::UpdatedAt, chrono::Utc::now().naive_utc());
    let (sql, values) = q
        .returning(Query::returning().columns(PARTNER_COLUMNS))
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}
