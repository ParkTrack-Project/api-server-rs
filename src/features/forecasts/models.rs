use sea_query::Iden;

#[derive(Iden)]
pub enum Forecasts {
    Table,
    ForecastId,
    ZoneId,
    CameraId,
    PartnerId,
    ModelType,
    ModelVersion,
    GeneratedAt,
    PredictedFor,
    Capacity,
    PredictedOccupied,
    ProbabilityFreeSpace,
    Confidence,
    ConfidenceLevel,
    Metadata,
}
