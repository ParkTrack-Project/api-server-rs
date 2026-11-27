use anyhow::Context;
use sea_query::{Alias, Expr, ExprTrait, OnConflict, Order, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

use crate::error::{ApiError, ApiResult};

use super::{
    models::{WEATHER_COLUMNS, WeatherObservations as W},
    requests, responses,
};

pub async fn list_weather(
    pool: impl PgExecutor<'_>,
    query: requests::ListWeather,
) -> ApiResult<Vec<responses::WeatherObservation>> {
    let mut select = Query::select();
    select
        .from(W::Table)
        .columns(WEATHER_COLUMNS)
        .apply_if(query.camera_id, |q, id| {
            q.and_where(Expr::col(W::CameraId).eq(id));
        })
        .apply_if(query.from, |q, from| {
            q.and_where(Expr::col(W::ObservedAt).gte(from));
        })
        .apply_if(query.to, |q, to| {
            q.and_where(Expr::col(W::ObservedAt).lt(to));
        });
    if query.latest_only {
        select
            .distinct_on([W::CameraId])
            .order_by(W::CameraId, Order::Asc)
            .order_by(W::ObservedAt, Order::Desc);
        select = Query::select()
            .from_subquery(select, Alias::new("latest"))
            .columns(WEATHER_COLUMNS)
            .to_owned();
    }

    let (sql, values) = select
        .order_by(W::ObservedAt, Order::Asc)
        .order_by(W::CameraId, Order::Asc)
        .limit(query.top)
        .offset(query.offset)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .context("listing weather observations")
        .map_err(ApiError::from)
}

pub async fn find_weather(
    pool: impl PgExecutor<'_>,
    key: &requests::WeatherKey,
) -> ApiResult<Option<responses::WeatherObservation>> {
    let (sql, values) = Query::select()
        .from(W::Table)
        .columns(WEATHER_COLUMNS)
        .and_where(Expr::col(W::CameraId).eq(key.camera_id))
        .and_where(Expr::col(W::ObservedAt).eq(key.observed_at))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .context("fetching weather observation")
        .map_err(ApiError::from)
}

pub async fn upsert_weather(
    pool: impl PgExecutor<'_>,
    key: &requests::WeatherKey,
    input: requests::UpsertWeather,
) -> ApiResult<bool> {
    let mut on_conflict = OnConflict::columns([W::CameraId, W::ObservedAt]);
    on_conflict.update_columns([W::Temperature, W::Precipitation]);

    let (sql, values) = Query::insert()
        .into_table(W::Table)
        .columns(WEATHER_COLUMNS)
        .values([
            key.camera_id.into(),
            key.observed_at.into(),
            input.temperature.into(),
            input.precipitation.into(),
        ])
        .context("building weather insert")?
        .on_conflict(on_conflict)
        // PostgreSQL 18 exposes the previous row on the conflict-update path.
        .returning(Query::returning().expr(Expr::col((Alias::new("old"), W::CameraId)).is_null()))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(|error| match &error {
            sqlx::Error::Database(cause) if cause.is_foreign_key_violation() => {
                ApiError::NotFound("camera".into())
            }
            _ => anyhow::Error::from(error)
                .context("saving weather observation")
                .into(),
        })
}
