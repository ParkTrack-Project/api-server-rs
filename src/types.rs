use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate)]
pub struct Bbox {
    #[validate(range(min = -180.0, max = 180.0))]
    pub min_longitude: f32,
    #[validate(range(min = -90.0, max = 90.0))]
    pub min_latitude: f32,
    #[validate(range(min = -180.0, max = 180.0))]
    pub max_longitude: f32,
    #[validate(range(min = -90.0, max = 90.0))]
    pub max_latitude: f32,
}

impl Bbox {
    pub fn new(
        min_longitude: f32,
        min_latitude: f32,
        max_longitude: f32,
        max_latitude: f32,
    ) -> Self {
        Self {
            min_longitude,
            min_latitude,
            max_longitude,
            max_latitude,
        }
    }
}

impl<'de> Deserialize<'de> for Bbox {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() != 4 {
            return Err(serde::de::Error::custom(
                "expected exactly 4 comma-separated values",
            ));
        }

        let min_longitude: f32 = parts[0].parse().map_err(serde::de::Error::custom)?;
        let min_latitude: f32 = parts[1].parse().map_err(serde::de::Error::custom)?;
        let max_longitude: f32 = parts[2].parse().map_err(serde::de::Error::custom)?;
        let max_latitude: f32 = parts[3].parse().map_err(serde::de::Error::custom)?;

        Ok(Bbox {
            min_longitude,
            min_latitude,
            max_longitude,
            max_latitude,
        })
    }
}

#[derive(Default, Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CameraView {
    #[default]
    Full,
    Map,
}

#[derive(Debug, Clone, Copy, Deserialize, strum::AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Medium,
    High,
}

pub(crate) const MAX_OBSERVATION_OFFSET: u64 = i64::MAX as u64;

pub(crate) fn default_observation_top() -> u64 {
    1000
}
