use anyhow::Context;
use sea_query::{Expr, ExprTrait, Order, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

use crate::{
    error::{ApiError, ApiResult},
    features::zones::repository::{self as zones, ZoneAccess},
};

use super::{models::OccupancyObservations as O, requests, responses};

pub async fn list_observations(
    pool: impl PgExecutor<'_>,
    query: requests::ListObservations,
) -> ApiResult<Vec<responses::Observation>> {
    let (sql, values) = Query::select()
        .from(O::Table)
        .columns([
            O::ObservationId,
            O::ZoneId,
            O::CameraId,
            O::PartnerId,
            O::SourceType,
            O::SourceRef,
            O::Capacity,
            O::Occupied,
            O::FreeCount,
            O::Confidence,
            O::ConfidenceLevel,
            O::ObservedAt,
            O::IngestedAt,
            O::Metadata,
        ])
        .apply_if(query.zone_id, |q, id| {
            q.and_where(Expr::col(O::ZoneId).eq(id));
        })
        .apply_if(query.camera_id, |q, id| {
            q.and_where(Expr::col(O::CameraId).eq(id));
        })
        .apply_if(query.from, |q, from| {
            q.and_where(Expr::col(O::ObservedAt).gte(from));
        })
        .apply_if(query.to, |q, to| {
            q.and_where(Expr::col(O::ObservedAt).lt(to));
        })
        .order_by(O::ObservedAt, Order::Asc)
        .order_by(O::ObservationId, Order::Asc)
        .limit(query.top)
        .offset(query.offset)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .context("listing occupancy observations")
        .map_err(ApiError::from)
}

pub async fn create_observation(
    pool: impl PgExecutor<'_>,
    input: requests::CreateObservation,
    scope: ZoneAccess,
) -> ApiResult<Option<i32>> {
    let sample = zones::select_zone_for_observation(input.zone_id, input.occupied, scope)
        .exprs([
            Expr::val("car-detector"),
            Expr::val(input.source_ref),
            Expr::val(input.occupied),
            Expr::val(input.confidence),
            Expr::val(input.confidence_level.as_ref().map(AsRef::as_ref)),
            Expr::val(input.observed_at),
            Expr::val(input.metadata),
        ])
        .take();

    let (sql, values) = Query::insert()
        .into_table(O::Table)
        .columns([
            O::ZoneId,
            O::CameraId,
            O::PartnerId,
            O::Capacity,
            O::SourceType,
            O::SourceRef,
            O::Occupied,
            O::Confidence,
            O::ConfidenceLevel,
            O::ObservedAt,
            O::Metadata,
        ])
        .select_from(sample)
        .context("building occupancy insert")?
        .returning_col(O::ObservationId)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(|error| match &error {
            sqlx::Error::Database(cause) if cause.is_unique_violation() => {
                ApiError::Conflict("occupancy observation".into())
            }
            _ => anyhow::Error::from(error)
                .context("creating occupancy observation")
                .into(),
        })
}
