use sea_query::Iden;

#[derive(Iden)]
pub enum WeatherObservations {
    Table,
    CameraId,
    ObservedAt,
    Temperature,
    Precipitation,
}

pub const WEATHER_COLUMNS: [WeatherObservations; 4] = [
    WeatherObservations::CameraId,
    WeatherObservations::ObservedAt,
    WeatherObservations::Temperature,
    WeatherObservations::Precipitation,
];
