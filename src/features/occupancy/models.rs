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
}
