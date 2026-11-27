use anyhow::Context;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

use crate::{
    error::{ApiError, ApiResult},
    features::zones::repository::{self as zones, ZoneAccess},
};

use super::{models::Forecasts as F, requests};

pub async fn create_forecast(
    pool: impl PgExecutor<'_>,
    input: requests::CreateForecast,
    scope: ZoneAccess,
) -> ApiResult<Option<i32>> {
    let sample = zones::select_zone_for_observation(input.zone_id, input.predicted_occupied, scope)
        .exprs([
            Expr::val(input.model_type),
            Expr::val(input.model_version),
            Expr::val(input.generated_at),
            Expr::val(input.predicted_for),
            Expr::val(input.predicted_occupied),
            Expr::val(input.probability_free_space),
            Expr::val(input.confidence),
            Expr::val(input.confidence_level.as_ref().map(AsRef::as_ref)),
            Expr::val(input.metadata),
        ])
        .take();

    let (sql, values) = Query::insert()
        .into_table(F::Table)
        .columns([
            F::ZoneId,
            F::CameraId,
            F::PartnerId,
            F::Capacity,
            F::ModelType,
            F::ModelVersion,
            F::GeneratedAt,
            F::PredictedFor,
            F::PredictedOccupied,
            F::ProbabilityFreeSpace,
            F::Confidence,
            F::ConfidenceLevel,
            F::Metadata,
        ])
        .select_from(sample)
        .context("building forecast insert")?
        .returning_col(F::ForecastId)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(|error| match &error {
            sqlx::Error::Database(cause) if cause.is_unique_violation() => {
                ApiError::Conflict("forecast".into())
            }
            _ => anyhow::Error::from(error)
                .context("creating forecast")
                .into(),
        })
}
