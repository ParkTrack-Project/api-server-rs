use crate::types::{Bbox, CameraView};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneType {
    Parallel,
    Standard,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationType {
    Street,
    Yard,
    OpenLot,
    Underground,
    Multilevel,
}
impl LocationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Street => "street",
            Self::Yard => "yard",
            Self::OpenLot => "open_lot",
            Self::Underground => "underground",
            Self::Multilevel => "multilevel",
        }
    }
}
impl ZoneType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parallel => "parallel",
            Self::Standard => "standard",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ZoneId {
    #[validate(range(min = 1))]
    pub zone_id: i32,
}
#[derive(Debug, Deserialize, Validate)]
pub struct ListZones {
    #[validate(range(min = 1))]
    pub camera_id: Option<i32>,
    #[validate(range(min = 1))]
    pub partner_id: Option<i32>,
    pub is_active: Option<bool>,
    #[validate(range(min = 0))]
    pub min_free_count: Option<i32>,
    #[validate(range(min = 0., max = 1.))]
    pub min_confidence: Option<f64>,
    #[validate(range(min = 0))]
    pub max_pay: Option<i32>,
    pub include_private: Option<bool>,
    pub include_accessible: Option<bool>,
    #[validate(custom(function = "validate_locations"))]
    pub hide_location_types: Option<String>,
    #[validate(nested)]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    pub view: CameraView,
}
fn validate_locations(value: &str) -> Result<(), ValidationError> {
    if value.split(',').all(|s| {
        matches!(
            s,
            "street" | "yard" | "open_lot" | "underground" | "multilevel"
        )
    }) {
        Ok(())
    } else {
        Err(ValidationError::new("location_type"))
    }
}
#[derive(Debug, Deserialize, Validate)]
pub struct CreateZone {
    #[validate(range(min = 1))]
    pub camera_id: i32,
    pub zone_type: ZoneType,
    #[validate(range(min = 0))]
    pub capacity: i32,
    #[validate(range(min = 0))]
    pub pay: i32,
    #[validate(custom(function = "validate_geometry"))]
    pub geometry: serde_json::Value,
    #[validate(custom(function = "validate_image"))]
    pub image_polygon: serde_json::Value,
    pub location_type: Option<LocationType>,
    pub is_private: Option<bool>,
    pub is_accessible: Option<bool>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateZone {
    #[validate(range(min = 1))]
    pub camera_id: Option<i32>,
    pub zone_type: Option<ZoneType>,
    #[validate(range(min = 0))]
    pub capacity: Option<i32>,
    #[validate(range(min = 0))]
    pub pay: Option<i32>,
    #[validate(range(min = 0))]
    pub occupied: Option<i32>,
    #[validate(range(min = 0., max = 1.))]
    pub confidence: Option<f64>,
    #[validate(custom(function = "validate_geometry"))]
    pub geometry: Option<serde_json::Value>,
    #[validate(custom(function = "validate_image"))]
    pub image_polygon: Option<serde_json::Value>,
    pub location_type: Option<LocationType>,
    pub is_private: Option<bool>,
    pub is_accessible: Option<bool>,
    pub is_active: Option<bool>,
}
pub fn centroid(geometry: &serde_json::Value) -> (f64, f64) {
    let points = geometry["coordinates"][0].as_array();
    let (mut longitude, mut latitude, mut count) = (0., 0., 0.);
    if let Some(points) = points {
        for point in points {
            if let (Some(x), Some(y)) = (point[0].as_f64(), point[1].as_f64()) {
                longitude += x;
                latitude += y;
                count += 1.;
            }
        }
    }
    if count == 0. {
        (0., 0.)
    } else {
        (longitude / count, latitude / count)
    }
}
fn validate_geometry(value: &serde_json::Value) -> Result<(), ValidationError> {
    let valid = value["type"] == "Polygon"
        && value["coordinates"].as_array().is_some_and(|rings| {
            !rings.is_empty()
                && rings.iter().all(|ring| {
                    ring.as_array().is_some_and(|points| {
                        points.len() >= 4
                            && points.first() == points.last()
                            && points.iter().all(|point| {
                                point.as_array().is_some_and(|p| {
                                    p.len() == 2
                                        && p[0]
                                            .as_f64()
                                            .is_some_and(|x| (-180. ..=180.).contains(&x))
                                        && p[1].as_f64().is_some_and(|y| (-90. ..=90.).contains(&y))
                                })
                            })
                    })
                })
        });
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new("polygon"))
    }
}
fn validate_image(value: &serde_json::Value) -> Result<(), ValidationError> {
    let Some(points) = value.as_array().filter(|p| p.len() == 4) else {
        return Err(ValidationError::new("image_polygon"));
    };
    if !points.iter().all(|point| {
        point
            .as_array()
            .is_some_and(|p| p.len() == 2 && p.iter().all(|v| v.as_u64().is_some()))
    }) {
        return Err(ValidationError::new("image_polygon"));
    }
    let area: f64 = (0..4)
        .map(|i| {
            let a = &points[i];
            let b = &points[(i + 1) % 4];
            a[0].as_f64().unwrap_or_default() * b[1].as_f64().unwrap_or_default()
                - b[0].as_f64().unwrap_or_default() * a[1].as_f64().unwrap_or_default()
        })
        .sum();
    if area > 0. {
        Ok(())
    } else {
        Err(ValidationError::new("clockwise_image_polygon"))
    }
}
