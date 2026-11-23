use sea_query::Expr;
use sqlx::PgExecutor;

use crate::{
    error::ApiResult,
    features::users::models::{UserRow, Users},
};

use super::requests;

// pub async fn update_user(
//     pool: impl PgExecutor<'_>,
//     user_id: i32,
//     query: requests::UpdateUser,
// ) -> ApiResult<UserRow> {
//     let (sql, values) = sea_query::Query::update()
//         .table(Users::Table)
//         .and_where(Expr::col(Users::).eq(camera_id))
//         .values(
//             [
//                 query.title.map(|v| (Cameras::Title, v.into())),
//                 query.source.map(|v| (Cameras::Source, v.into())),
//                 query
//                     .image_width
//                     .map(|v| (Cameras::ImageWidth, v.into())),
//                 query
//                     .image_height
//                     .map(|v| (Cameras::ImageHeight, v.into())),
//                 query.calib.map(|v| (Cameras::Calib, v.into())),
//                 query.latitude.map(|v| (Cameras::Latitude, v.into())),
//                 query.longitude.map(|v| (Cameras::Longitude, v.into())),
//             ]
//             .into_iter()
//             .flatten(),
//         )
//         .returning(sea_query::Query::returning().columns(CAMERA_COLUMNS))
//         .to_owned()
//         .build_sqlx(PostgresQueryBuilder);

//     sqlx::query_as_with::<_, CameraRow, _>(sqlx::AssertSqlSafe(sql), values)
//         .fetch_optional(pool)
//         .await
//         .with_context(|| format!("updating camera with id {}", camera_id))
//         .map_err(ApiError::from)
// }
