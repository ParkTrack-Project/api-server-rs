use axum::{Router, routing::post};

use crate::state::ApiState;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn forecast_routes() -> Router<ApiState> {
    Router::new().route("/new", post(handlers::create_forecast))
}
