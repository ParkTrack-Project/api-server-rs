use anyhow::Context;
use phonenumber::format;
use sea_query::{Alias, Cond, Expr, ExprTrait, Func, Iden, OnConflict, PostgresQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{PgPool, prelude::FromRow};
use time::OffsetDateTime;

use crate::{
    cameras::{
        models::{
            CAMERA_COLUMNS, CAMERA_MAP_ITEM_COLUMNS, CAMERA_NEXT_COLUMNS, Cameras,
            CameraMapItemRow, CameraNextRow, CameraRow, CreateCameraRow,
        },
        requests, responses,
    },
    error::{ApiError, ApiResult},
};

#[derive(Debug, Clone, Copy)]
pub enum CameraAccess {
    All,
    Public,
    PublicOwned { user_id: i32 },
    Partner { partner_id: i32 },
    PartnerOwned { partner_id: i32, user_id: i32 },
}

pub async fn list_cameras(
    pool: &PgPool,
    query: requests::ListCamerasQuery,
    scope: CameraAccess,
) -> ApiResult<Vec<CameraRow>> {
    let mut select = base_list_query(query, scope);
    let (sql, values) = select
        .columns(CAMERA_COLUMNS)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, CameraRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .context("listing full cameras")
        .map_err(ApiError::from)
}

pub async fn list_camera_map_items(
    pool: &PgPool,
    query: requests::ListCamerasQuery,
    scope: CameraAccess,
) -> ApiResult<Vec<CameraMapItemRow>> {
    let mut select = base_list_query(query, scope);
    let (sql, values) = select
        .columns(CAMERA_MAP_ITEM_COLUMNS)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, CameraMapItemRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await
        .context("listing map cameras")
        .map_err(ApiError::from)
}

pub async fn get_next_camera(pool: &PgPool, idx: i32) -> ApiResult<CameraNextRow> {
    let (sql, values) = sea_query::Query::select()
        .columns(CAMERA_NEXT_COLUMNS)
        .column(Alias::new("next_id"))
        .offset((idx - 1).try_into().unwrap_or(0))
        .limit(1)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .context("fetching next camera")
        .map_err(ApiError::from)
}

pub async fn count_cameras(pool: &PgPool) -> ApiResult<i32> {
    let (sql, values) = sea_query::Query::select()
        .from(Cameras::Table)
        .expr(Func::count(Expr::col(Cameras::CameraId)))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_scalar_with::<_, i32, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .context("counting cameras")
        .map_err(ApiError::from)
}

pub async fn create_camera(
    pool: &PgPool,
    query: requests::CreateCamera,
    user_id: Option<i32>,
) -> ApiResult<CreateCameraRow> {
    let (sql, values) = sea_query::Query::insert()
        .into_table(Cameras::Table)
        .columns([
            Cameras::Title,
            Cameras::Source,
            Cameras::ImageWidth,
            Cameras::ImageHeight,
            Cameras::Calib,
            Cameras::Latitude,
            Cameras::Longitude,
            Cameras::CreatedByUserId,
        ])
        .values_panic([
            query.title.clone().into(),
            query.source.into(),
            query.image_width.into(),
            query.image_height.into(),
            query.calib.into(),
            query.latitude.into(),
            query.longitude.into(),
            user_id.into(),
        ])
        .returning_col(Cameras::CameraId)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, CreateCameraRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return ApiError::Conflict("camera already exists".to_string());
                }
            }

            return ApiError::InternalError(
                anyhow::Error::from(e)
                    .context(format!("creating new camera with title {}", query.title)),
            );
        })
}

pub async fn get_camera(
    pool: &PgPool,
    camera_id: i32,
    scope: CameraAccess,
) -> ApiResult<Option<CameraRow>> {
    let (sql, values) = sea_query::Query::select()
        .from(Cameras::Table)
        .cond_where(scope_condition(scope))
        .and_where(Expr::col(Cameras::CameraId).eq(camera_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, CameraRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("fetching camera with id {}", camera_id))
        .map_err(ApiError::from)
}

pub async fn update_camera(
    pool: &PgPool,
    camera_id: i32,
    query: requests::UpdateCamera,
    scope: CameraAccess,
) -> ApiResult<Option<CameraRow>> {
    let (sql, values) = sea_query::Query::update()
        .table(Cameras::Table)
        .cond_where(scope_condition(scope))
        .and_where(Expr::col(Cameras::CameraId).eq(camera_id))
        .values(
            [
                query.title.map(|v| (Cameras::Title, v.into())),
                query.source.map(|v| (Cameras::Source, v.into())),
                query
                    .image_width
                    .map(|v| (Cameras::ImageWidth, v.into())),
                query
                    .image_height
                    .map(|v| (Cameras::ImageHeight, v.into())),
                query.calib.map(|v| (Cameras::Calib, v.into())),
                query.latitude.map(|v| (Cameras::Latitude, v.into())),
                query.longitude.map(|v| (Cameras::Longitude, v.into())),
            ]
            .into_iter()
            .flatten(),
        )
        .returning(sea_query::Query::returning().columns(CAMERA_COLUMNS))
        .to_owned()
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, CameraRow, _>(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("updating camera with id {}", camera_id))
        .map_err(ApiError::from)
}

pub async fn delete_camera(pool: &PgPool, camera_id: i32, scope: CameraAccess) -> ApiResult<()> {
    let (sql, values) = sea_query::Query::delete()
        .from_table(Cameras::Table)
        .cond_where(scope_condition(scope))
        .and_where(Expr::col(Cameras::CameraId).eq(camera_id))
        .build_sqlx(PostgresQueryBuilder);

    let result = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await
        .with_context(|| format!("deleting camera with id {}", camera_id))
        .map_err(ApiError::from)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("camera with id {camera_id}")));
    }

    Ok(())
}

fn base_list_query(
    query: requests::ListCamerasQuery,
    scope: CameraAccess,
) -> sea_query::SelectStatement {
    sea_query::Query::select()
        .from(Cameras::Table)
        .cond_where(scope_condition(scope))
        .order_by(Cameras::CameraId, sea_query::Order::Asc)
        .apply_if(query.q.as_deref(), |query, v| {
            query.and_where(Expr::col(Cameras::Title).like(format!("%{v}%")));
        })
        .apply_if(query.is_active, |query, v| {
            query.and_where(Expr::col(Cameras::IsActive).eq(v));
        })
        .apply_if(query.bbox.as_ref(), |query, v| {
            query.cond_where(
                Cond::all()
                    .add(Expr::col(Cameras::Latitude).between(v.min_latitude, v.max_latitude))
                    .add(
                        Expr::col(Cameras::Longitude).between(v.min_longitude, v.max_longitude),
                    ),
            );
        })
        .to_owned()
}

fn scope_condition(scope: CameraAccess) -> sea_query::Condition {
    let condition = sea_query::Condition::all();

    match scope {
        CameraAccess::All => condition,
        CameraAccess::Public => condition.add(Expr::col(Cameras::PartnerId).is_null()),
        CameraAccess::PublicOwned { user_id } => {
            condition.add(Expr::col(Cameras::CreatedByUserId).eq(user_id))
        }
        CameraAccess::Partner { partner_id } => {
            condition.add(Expr::col(Cameras::PartnerId).eq(partner_id))
        }
        CameraAccess::PartnerOwned {
            partner_id,
            user_id,
        } => condition
            .add(Expr::col(Cameras::PartnerId).eq(partner_id))
            .add(Expr::col(Cameras::CreatedByUserId).eq(user_id)),
    }
}
