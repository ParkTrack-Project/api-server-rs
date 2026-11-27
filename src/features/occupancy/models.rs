use sea_query::Iden;

#[derive(Iden)]
pub enum OccupancyObservations {
    Table,
    ObservationId,
    ZoneId,
    CameraId,
    PartnerId,
    SourceType,
    SourceRef,
    Capacity,
    Occupied,
    FreeCount,
    Confidence,
    ConfidenceLevel,
    ObservedAt,
    IngestedAt,
    Metadata,
    CreatedByUserId,
}

pub const OBSERVATION_COLUMNS: [OccupancyObservations; 15] = [
    OccupancyObservations::ObservationId,
    OccupancyObservations::ZoneId,
    OccupancyObservations::CameraId,
    OccupancyObservations::PartnerId,
    OccupancyObservations::SourceType,
    OccupancyObservations::SourceRef,
    OccupancyObservations::Capacity,
    OccupancyObservations::Occupied,
    OccupancyObservations::FreeCount,
    OccupancyObservations::Confidence,
    OccupancyObservations::ConfidenceLevel,
    OccupancyObservations::ObservedAt,
    OccupancyObservations::IngestedAt,
    OccupancyObservations::Metadata,
    OccupancyObservations::CreatedByUserId,
];

pub const SERIES_COLUMNS: [OccupancyObservations; 7] = [
    OccupancyObservations::ObservedAt,
    OccupancyObservations::Occupied,
    OccupancyObservations::FreeCount,
    OccupancyObservations::Capacity,
    OccupancyObservations::Confidence,
    OccupancyObservations::ConfidenceLevel,
    OccupancyObservations::SourceType,
];
