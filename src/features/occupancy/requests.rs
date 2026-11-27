use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use validator::{Validate, ValidationError};

use crate::types::{Bbox, ConfidenceLevel, MAX_OBSERVATION_OFFSET, default_observation_top};

pub(super) const DETECTOR_SOURCE: &str = "car-detector";
pub(super) const CAMERA_CV: &str = "camera_cv";

#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Observations,
    Series,
    Map,
}

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_filters"))]
pub struct ListObservations {
    #[validate(range(min = 1))]
    pub camera_id: Option<i32>,
    #[validate(range(min = 1))]
    pub zone_id: Option<i32>,
    #[validate(range(min = 1))]
    pub partner_id: Option<i32>,
    #[validate(length(min = 1, max = 50))]
    pub source_type: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub latest_only: bool,
    #[validate(nested)]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    pub view: View,
    #[serde(default = "default_observation_top")]
    #[validate(range(min = 1, max = 1000))]
    pub top: u64,
    #[serde(default)]
    #[validate(range(max = "MAX_OBSERVATION_OFFSET"))]
    pub offset: u64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ObservationId {
    #[validate(range(min = 1))]
    pub observation_id: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateObservation {
    #[validate(range(min = 1))]
    pub zone_id: i32,
    #[validate(length(min = 1, max = 50))]
    pub source_type: Option<String>,
    #[validate(range(min = 0))]
    pub capacity: Option<i32>,
    #[validate(range(min = 0))]
    pub occupied: i32,
    #[validate(range(min = 0., max = 1.))]
    pub confidence: f64,
    pub confidence_level: Option<ConfidenceLevel>,
    #[validate(length(min = 1, max = 255))]
    pub source_ref: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update"))]
pub struct UpdateObservation {
    pub observed_at: Option<DateTime<Utc>>,
    #[validate(range(min = 0))]
    pub capacity: Option<i32>,
    #[validate(range(min = 0))]
    pub occupied: Option<i32>,
    #[validate(range(min = 0., max = 1.))]
    pub confidence: Option<f64>,
    #[serde(default, deserialize_with = "nullable")]
    pub source_ref: Option<Option<String>>,
    #[serde(default, deserialize_with = "nullable")]
    pub metadata: Option<Option<Value>>,
}

fn nullable<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(deserializer).map(Some)
}

fn validate_filters(query: &ListObservations) -> Result<(), ValidationError> {
    if let (Some(from), Some(to)) = (query.from, query.to)
        && from >= to
    {
        return Err(ValidationError::new("interval").with_message("from must precede to".into()));
    }
    if let Some(bbox) = &query.bbox
        && (bbox.min_longitude > bbox.max_longitude || bbox.min_latitude > bbox.max_latitude)
    {
        return Err(ValidationError::new("bbox").with_message("invalid bbox bounds".into()));
    }
    Ok(())
}

fn validate_update(input: &UpdateObservation) -> Result<(), ValidationError> {
    if let Some(Some(reference)) = &input.source_ref
        && !(1..=255).contains(&reference.chars().count())
    {
        return Err(ValidationError::new("source_ref")
            .with_message("source_ref must contain between 1 and 255 characters".into()));
    }
    Ok(())
}
