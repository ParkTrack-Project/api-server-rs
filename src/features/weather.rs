use axum::{Router, routing::get};

use crate::state::ApiState;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn weather_routes() -> Router<ApiState> {
    Router::new().route("/", get(handlers::list_weather)).route(
        "/{camera_id}/{observed_at}",
        get(handlers::get_weather).put(handlers::upsert_weather),
    )
}
