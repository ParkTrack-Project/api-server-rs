use sea_query::Iden;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

#[derive(Iden)]
pub enum CameraIden {
    Table,
    CameraId,
    Title,
    Source,
    ImageWidth,
    ImageHeight,
    Calib,
    Latitude,
    Longitude,
    PartnerId,
    CreatedByUserId,
    IsActive,
    LastSnapshotAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, FromRow)]
pub struct CameraRow {
    pub camera_id: i32,
    pub title: String,
    pub source: String,
    pub image_width: i32,
    pub image_height: i32,
    pub calib: Option<serde_json::Value>,
    pub latitude: f32,
    pub longitude: f32,
    pub partner_id: Option<i32>,
    pub created_by_user_id: Option<i32>,
    pub is_active: bool,
    pub last_snapshot_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub const CAMERA_COLUMNS: [CameraIden; 14] = [
    CameraIden::CameraId,
    CameraIden::Title,
    CameraIden::Source,
    CameraIden::ImageWidth,
    CameraIden::ImageHeight,
    CameraIden::Calib,
    CameraIden::Latitude,
    CameraIden::Longitude,
    CameraIden::PartnerId,
    CameraIden::CreatedByUserId,
    CameraIden::IsActive,
    CameraIden::LastSnapshotAt,
    CameraIden::CreatedAt,
    CameraIden::UpdatedAt,
];

#[derive(Debug, FromRow)]
pub struct CameraMapItemRow {
    pub camera_id: i32,
    pub title: String,
    pub latitude: f32,
    pub longitude: f32,
    pub partner_id: Option<i32>,
    pub is_active: bool,
}

pub const CAMERA_MAP_ITEM_COLUMNS: [CameraIden; 6] = [
    CameraIden::CameraId,
    CameraIden::Title,
    CameraIden::Latitude,
    CameraIden::Longitude,
    CameraIden::PartnerId,
    CameraIden::IsActive,
];

#[derive(Debug, FromRow)]
pub struct CameraNextRow {
    pub camera_id: i32,
    pub source: String,
    pub image_width: i32,
    pub image_height: i32,
    pub calib: Option<serde_json::Value>,
    pub partner_id: Option<i32>,
    pub is_active: bool,
}

pub const CAMERA_NEXT_COLUMNS: [CameraIden; 7] = [
    CameraIden::CameraId,
    CameraIden::Source,
    CameraIden::ImageWidth,
    CameraIden::ImageHeight,
    CameraIden::Calib,
    CameraIden::PartnerId,
    CameraIden::IsActive,
];

#[derive(Debug, FromRow)]
pub struct CreateCameraRow {
    pub camera_id: i32,
}
