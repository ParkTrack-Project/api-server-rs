use axum::{
    Router,
    routing::{get, post},
};

use crate::state::ApiState;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn weather_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(handlers::list_weather))
        .route("/new", post(handlers::save_weather))
}
