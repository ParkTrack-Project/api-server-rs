use anyhow::Context;
use sea_query::{Alias, Asterisk, Cond, Expr, ExprTrait, Func, Order, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::{PgExecutor, PgPool};

use crate::{
    error::{ApiError, ApiResult},
    features::zones::{
        models::ParkingZones as Z,
        repository::{self as zones, ZoneAccess},
    },
};

use super::{
    models::{OBSERVATION_COLUMNS, OccupancyObservations as O, SERIES_COLUMNS},
    requests::{self, CAMERA_CV, DETECTOR_SOURCE, View},
    responses,
};

#[derive(Clone, Copy, Debug)]
pub enum ObservationAccess {
    All,
    Public,
    Detector,
}

fn condition(access: ObservationAccess) -> Cond {
    let cond = Cond::all();
    match access {
        ObservationAccess::All => cond,
        ObservationAccess::Public => cond.add(Expr::col((O::Table, O::PartnerId)).is_null()).add(
            Expr::col((O::Table, O::ZoneId)).in_subquery(
                Query::select()
                    .column(Z::ParkingZoneId)
                    .from(Z::Table)
                    .and_where(Expr::col(Z::PartnerId).is_null())
                    .take(),
            ),
        ),
        ObservationAccess::Detector => {
            cond.add(Expr::col((O::Table, O::SourceType)).eq(DETECTOR_SOURCE))
        }
    }
}

pub async fn list_observations(
    pool: &PgPool,
    query: requests::ListObservations,
    access: ObservationAccess,
) -> ApiResult<responses::ListObservations> {
    let mut selected = Query::select();
    selected
        .from(O::Table)
        .cond_where(condition(access))
        .apply_if(query.zone_id, |q, id| {
            q.and_where(Expr::col((O::Table, O::ZoneId)).eq(id));
        })
        .apply_if(query.camera_id, |q, id| {
            q.and_where(Expr::col((O::Table, O::CameraId)).eq(id));
        })
        .apply_if(query.partner_id, |q, id| {
            q.and_where(Expr::col((O::Table, O::PartnerId)).eq(id));
        })
        .apply_if(query.source_type.as_deref(), |q, source| {
            let column = Expr::col((O::Table, O::SourceType));
            q.and_where(if source == CAMERA_CV {
                column.is_in([CAMERA_CV, DETECTOR_SOURCE])
            } else {
                column.eq(source)
            });
        })
        .apply_if(query.from, |q, from| {
            q.and_where(Expr::col((O::Table, O::ObservedAt)).gte(from));
        })
        .apply_if(query.to, |q, to| {
            q.and_where(Expr::col((O::Table, O::ObservedAt)).lt(to));
        })
        .apply_if(query.at, |q, at| {
            q.and_where(Expr::col((O::Table, O::ObservedAt)).lte(at));
        });

    if query.bbox.is_some() || query.view == View::Map {
        selected.inner_join(
            Z::Table,
            Expr::col((O::Table, O::ZoneId)).equals((Z::Table, Z::ParkingZoneId)),
        );
    }
    selected.apply_if(query.bbox, |q, bbox| {
        q.and_where(
            Expr::col(Z::CentroidLongitude).between(bbox.min_longitude, bbox.max_longitude),
        )
        .and_where(Expr::col(Z::CentroidLatitude).between(bbox.min_latitude, bbox.max_latitude));
    });

    match query.view {
        View::Observations => {
            selected.columns(OBSERVATION_COLUMNS.map(|column| (O::Table, column)));
        }
        View::Series => {
            selected.columns(SERIES_COLUMNS.map(|column| (O::Table, column)));
        }
        View::Map => {
            selected
                .columns(
                    [
                        O::ZoneId,
                        O::CameraId,
                        O::Capacity,
                        O::Occupied,
                        O::FreeCount,
                        O::Confidence,
                        O::ConfidenceLevel,
                        O::ObservedAt,
                    ]
                    .map(|column| (O::Table, column)),
                )
                .columns([Z::Geometry, Z::Pay, Z::IsAccessible].map(|column| (Z::Table, column)))
                .expr_as(
                    Expr::col((Z::Table, Z::ZoneType)).cast_as(Alias::new("text")),
                    Z::ZoneType,
                )
                .expr_as(
                    Expr::col((Z::Table, Z::LocationType)).cast_as(Alias::new("text")),
                    Z::LocationType,
                )
                .expr_as(
                    Func::coalesce([Expr::col((Z::Table, Z::IsActive)), Expr::val(false)]),
                    Z::IsActive,
                );
        }
    }
    if query.view != View::Observations {
        selected.column((O::Table, O::ObservationId));
    }

    let (sql, values) = if query.latest_only || query.view == View::Map {
        selected
            .distinct_on([(O::Table, O::ZoneId)])
            .order_by((O::Table, O::ZoneId), Order::Asc)
            .order_by((O::Table, O::ObservedAt), Order::Desc)
            .order_by((O::Table, O::ObservationId), Order::Desc);
        Query::select()
            .column(Asterisk)
            .from_subquery(selected, Alias::new("latest"))
            .order_by(O::ObservedAt, Order::Asc)
            .order_by(O::ObservationId, Order::Asc)
            .limit(query.top)
            .offset(query.offset)
            .build_sqlx(PostgresQueryBuilder)
    } else {
        selected
            .order_by((O::Table, O::ObservedAt), Order::Asc)
            .order_by((O::Table, O::ObservationId), Order::Asc)
            .limit(query.top)
            .offset(query.offset)
            .build_sqlx(PostgresQueryBuilder)
    };

    let sql = sqlx::AssertSqlSafe(sql);
    match query.view {
        View::Observations => sqlx::query_as_with(sql, values)
            .fetch_all(pool)
            .await
            .map(responses::ListObservations::Observations),
        View::Series => sqlx::query_as_with(sql, values)
            .fetch_all(pool)
            .await
            .map(responses::ListObservations::Series),
        View::Map => sqlx::query_as_with(sql, values)
            .fetch_all(pool)
            .await
            .map(responses::ListObservations::Map),
    }
    .context("listing occupancy observations")
    .map_err(ApiError::from)
}

pub async fn find_observation(
    pool: impl PgExecutor<'_>,
    id: i32,
    access: ObservationAccess,
) -> ApiResult<Option<responses::Observation>> {
    let (sql, values) = Query::select()
        .from(O::Table)
        .columns(OBSERVATION_COLUMNS)
        .cond_where(condition(access))
        .and_where(Expr::col(O::ObservationId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .context("fetching occupancy observation")
        .map_err(ApiError::from)
}

pub async fn create_observation(
    pool: impl PgExecutor<'_>,
    input: requests::CreateObservation,
    scope: ZoneAccess,
    source: &str,
    created_by: Option<i32>,
) -> ApiResult<Option<i32>> {
    let capacity = input
        .capacity
        .map_or_else(|| Expr::col(Z::Capacity), Expr::val);
    let sample = zones::select_zone_for_observation(
        input.zone_id,
        if input.capacity.is_some() {
            0
        } else {
            input.occupied
        },
        scope,
    )
    .clear_selects()
    .columns([Z::ParkingZoneId, Z::CameraId, Z::PartnerId])
    .exprs([
        capacity,
        Expr::val(source),
        Expr::val(input.source_ref),
        Expr::val(input.occupied),
        Expr::val(input.confidence),
        Expr::val(input.confidence_level.as_ref().map(AsRef::as_ref)),
        Expr::val(input.observed_at),
        Expr::val(input.metadata),
        Expr::val(created_by),
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
            O::CreatedByUserId,
        ])
        .select_from(sample)
        .context("building occupancy insert")?
        .returning_col(O::ObservationId)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(write_error)
}

pub async fn update_observation(
    pool: impl PgExecutor<'_>,
    id: i32,
    input: requests::UpdateObservation,
    access: ObservationAccess,
) -> ApiResult<Option<responses::Observation>> {
    let capacity = input
        .capacity
        .map_or_else(|| Expr::col(O::Capacity), Expr::val);
    let occupied = input
        .occupied
        .map_or_else(|| Expr::col(O::Occupied), Expr::val);
    let (sql, values) = Query::update()
        .table(O::Table)
        .values(
            [
                input.observed_at.map(|value| (O::ObservedAt, value.into())),
                input.capacity.map(|value| (O::Capacity, value.into())),
                input.occupied.map(|value| (O::Occupied, value.into())),
                input.confidence.map(|value| (O::Confidence, value.into())),
                input.source_ref.map(|value| (O::SourceRef, value.into())),
                input.metadata.map(|value| (O::Metadata, value.into())),
            ]
            .into_iter()
            .flatten(),
        )
        .cond_where(condition(access))
        .and_where(Expr::col(O::ObservationId).eq(id))
        .and_where(occupied.lte(capacity))
        .returning(Query::returning().columns(OBSERVATION_COLUMNS))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(write_error)
}

pub async fn delete_observation(
    pool: impl PgExecutor<'_>,
    id: i32,
    access: ObservationAccess,
) -> ApiResult<bool> {
    let (sql, values) = Query::delete()
        .from_table(O::Table)
        .cond_where(condition(access))
        .and_where(Expr::col(O::ObservationId).eq(id))
        .build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .context("deleting occupancy observation")?;
    Ok(result.rows_affected() != 0)
}

fn write_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(cause) = &error
        && cause.is_unique_violation()
    {
        return ApiError::Conflict("occupancy observation".into());
    }
    anyhow::Error::from(error)
        .context("writing occupancy observation")
        .into()
}
