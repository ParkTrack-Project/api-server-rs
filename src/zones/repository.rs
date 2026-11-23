use super::{
    models::{ParkingZones as Z, ZoneRow},
    requests,
};
use crate::{
    cameras::models::Cameras,
    error::{ApiError, ApiResult},
};
use sea_query::{Alias, Cond, Expr, ExprTrait, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::PgExecutor;

#[derive(Debug, Clone, Copy)]
pub enum ZoneAccess {
    All,
    Public,
    PublicOwned { user_id: i32 },
    Partner { partner_id: i32 },
    PartnerOwned { partner_id: i32, user_id: i32 },
}

fn condition(scope: ZoneAccess) -> Cond {
    let c = Cond::all();
    match scope {
        ZoneAccess::All => c,
        ZoneAccess::Public => c.add(Expr::col(Z::PartnerId).is_null()),
        ZoneAccess::PublicOwned { user_id } => c
            .add(Expr::col(Z::PartnerId).is_null())
            .add(Expr::col(Z::CreatedByUserId).eq(user_id)),
        ZoneAccess::Partner { partner_id } => c.add(Expr::col(Z::PartnerId).eq(partner_id)),
        ZoneAccess::PartnerOwned {
            partner_id,
            user_id,
        } => c
            .add(Expr::col(Z::PartnerId).eq(partner_id))
            .add(Expr::col(Z::CreatedByUserId).eq(user_id)),
    }
}

fn select() -> sea_query::SelectStatement {
    let mut q = Query::select();
    q.from(Z::Table)
        .columns([
            Z::ParkingZoneId,
            Z::CameraId,
            Z::Capacity,
            Z::Occupied,
            Z::FreeCount,
            Z::Confidence,
            Z::Pay,
            Z::Geometry,
            Z::ImagePolygon,
            Z::PartnerId,
            Z::CreatedByUserId,
            Z::IsActive,
            Z::IsPrivate,
            Z::IsAccessible,
            Z::OccupancyUpdatedAt,
            Z::CreatedAt,
            Z::UpdatedAt,
            Z::CentroidLatitude,
            Z::CentroidLongitude,
        ])
        .expr_as(
            Expr::col(Z::ZoneType).cast_as(Alias::new("text")),
            Z::ZoneType,
        )
        .expr_as(
            Expr::col(Z::LocationType).cast_as(Alias::new("text")),
            Z::LocationType,
        )
        .expr_as(
            Expr::col(Z::ConfidenceLevel).cast_as(Alias::new("text")),
            Z::ConfidenceLevel,
        );

    q
}
fn db_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(e) = &error {
        if e.is_foreign_key_violation() {
            return ApiError::NotFound("related camera or zone".into());
        }
        if e.is_check_violation() {
            return ApiError::BadRequest("invalid zone values".into());
        }
        if e.is_unique_violation() {
            return ApiError::Conflict("zone".into());
        }
    }
    ApiError::from(anyhow::Error::from(error).context("accessing parking zones"))
}
pub async fn list_zones(
    pool: impl PgExecutor<'_>,
    input: requests::ListZones,
    scope: ZoneAccess,
) -> ApiResult<Vec<ZoneRow>> {
    let mut q = select();
    q.cond_where(condition(scope))
        .order_by(Z::ParkingZoneId, sea_query::Order::Asc)
        .apply_if(input.camera_id, |q, v| {
            q.and_where(Expr::col(Z::CameraId).eq(v));
        })
        .apply_if(input.partner_id, |q, v| {
            q.and_where(Expr::col(Z::PartnerId).eq(v));
        })
        .apply_if(input.is_active, |q, v| {
            q.and_where(Expr::col(Z::IsActive).eq(v));
        })
        .apply_if(input.min_free_count, |q, v| {
            q.and_where(Expr::col(Z::FreeCount).gte(v));
        })
        .apply_if(input.min_confidence, |q, v| {
            q.and_where(Expr::col(Z::Confidence).gte(v));
        })
        .apply_if(input.max_pay, |q, v| {
            q.and_where(Expr::col(Z::Pay).lte(v));
        })
        .apply_if(input.include_private, |q, include| {
            if include {
                q.and_where(Expr::col(Z::IsPrivate).is_not_null());
            } else {
                q.and_where(Expr::col(Z::IsPrivate).eq(false));
            }
        })
        .apply_if(input.include_accessible, |q, include| {
            if include {
                q.and_where(Expr::col(Z::IsAccessible).is_not_null());
            } else {
                q.and_where(Expr::col(Z::IsAccessible).eq(false));
            }
        })
        .apply_if(input.hide_location_types, |q, v| {
            q.and_where(
                Expr::col(Z::LocationType)
                    .cast_as(Alias::new("text"))
                    .is_not_in(v.split(',')),
            );
        })
        .apply_if(input.bbox, |q, b| {
            q.and_where(Expr::col(Z::CentroidLongitude).between(b.min_longitude, b.max_longitude))
                .and_where(Expr::col(Z::CentroidLatitude).between(b.min_latitude, b.max_latitude));
        });
    let (sql, values) = q.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .map_err(db_error)
}
pub async fn find(
    pool: impl PgExecutor<'_>,
    id: i32,
    scope: ZoneAccess,
) -> ApiResult<Option<ZoneRow>> {
    let (sql, values) = select()
        .cond_where(condition(scope))
        .and_where(Expr::col(Z::ParkingZoneId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}
pub async fn camera_partner(
    pool: impl PgExecutor<'_>,
    id: i32,
    public_only: bool,
) -> ApiResult<Option<Option<i32>>> {
    let mut q = Query::select();
    q.column(Cameras::PartnerId)
        .from(Cameras::Table)
        .and_where(Expr::col(Cameras::CameraId).eq(id));
    if public_only {
        q.and_where(Expr::col(Cameras::PartnerId).is_null());
    }
    let (sql, values) = q.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .map_err(db_error)
}
pub async fn create_zone(
    pool: impl PgExecutor<'_>,
    input: requests::CreateZone,
    owner: Option<i32>,
    partner: Option<i32>,
) -> ApiResult<i32> {
    let (x, y) = requests::centroid(&input.geometry);
    let location = input.location_type.map(|v| v.as_str());
    let (sql, values) = Query::insert()
        .into_table(Z::Table)
        .columns([
            Z::CameraId,
            Z::ZoneType,
            Z::Capacity,
            Z::Pay,
            Z::Geometry,
            Z::ImagePolygon,
            Z::PartnerId,
            Z::CreatedByUserId,
            Z::LocationType,
            Z::IsPrivate,
            Z::IsAccessible,
            Z::CentroidLongitude,
            Z::CentroidLatitude,
        ])
        .values_panic([
            input.camera_id.into(),
            Expr::val(input.zone_type.as_str()).cast_as(Alias::new("zone_types")),
            input.capacity.into(),
            input.pay.into(),
            input.geometry.into(),
            input.image_polygon.into(),
            partner.into(),
            owner.into(),
            Expr::val(location).cast_as(Alias::new("location_types")),
            input.is_private.into(),
            input.is_accessible.into(),
            x.into(),
            y.into(),
        ])
        .returning_col(Z::ParkingZoneId)
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}
pub async fn update_zone(
    pool: impl PgExecutor<'_>,
    id: i32,
    input: requests::UpdateZone,
    scope: ZoneAccess,
    camera: Option<Option<i32>>,
) -> ApiResult<u64> {
    let mut q = Query::update();
    q.table(Z::Table)
        .cond_where(condition(scope))
        .and_where(Expr::col(Z::ParkingZoneId).eq(id));
    // Validate against the current database values atomically, including partial updates.
    q.and_where(
        Expr::val(input.occupied)
            .if_null(Expr::col(Z::Occupied))
            .lte(Expr::val(input.capacity).if_null(Expr::col(Z::Capacity))),
    );
    let centroid = input.geometry.as_ref().map(requests::centroid);
    q.values(
        [
            input.camera_id.map(|v| (Z::CameraId, v.into())),
            camera.map(|v| (Z::PartnerId, v.into())),
            input.capacity.map(|v| (Z::Capacity, v.into())),
            input.occupied.map(|v| (Z::Occupied, v.into())),
            input.pay.map(|v| (Z::Pay, v.into())),
            input.confidence.map(|v| (Z::Confidence, v.into())),
            input.is_private.map(|v| (Z::IsPrivate, v.into())),
            input.is_accessible.map(|v| (Z::IsAccessible, v.into())),
            input.is_active.map(|v| (Z::IsActive, v.into())),
            input.geometry.map(|v| (Z::Geometry, v.into())),
            input.image_polygon.map(|v| (Z::ImagePolygon, v.into())),
            input.zone_type.map(|v| {
                (
                    Z::ZoneType,
                    Expr::val(v.as_str()).cast_as(Alias::new("zone_types")),
                )
            }),
            input.location_type.map(|v| {
                (
                    Z::LocationType,
                    Expr::val(v.as_str()).cast_as(Alias::new("location_types")),
                )
            }),
            centroid.map(|(x, _)| (Z::CentroidLongitude, x.into())),
            centroid.map(|(_, y)| (Z::CentroidLatitude, y.into())),
            (input.occupied.is_some() || input.confidence.is_some())
                .then(|| (Z::OccupancyUpdatedAt, chrono::Utc::now().into())),
        ]
        .into_iter()
        .flatten(),
    );
    q.value(Z::UpdatedAt, chrono::Utc::now());
    let (sql, values) = q.build_sqlx(PostgresQueryBuilder);
    Ok(sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .map_err(db_error)?
        .rows_affected())
}
pub async fn delete_zone(pool: impl PgExecutor<'_>, id: i32, scope: ZoneAccess) -> ApiResult<u64> {
    let (sql, values) = Query::delete()
        .from_table(Z::Table)
        .cond_where(condition(scope))
        .and_where(Expr::col(Z::ParkingZoneId).eq(id))
        .build_sqlx(PostgresQueryBuilder);
    Ok(sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .map_err(db_error)?
        .rows_affected())
}
