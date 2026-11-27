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

pub fn occupancy_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(handlers::list_observations))
        .route("/new", post(handlers::create_observation))
        .route(
            "/{observation_id}",
            get(handlers::get_observation)
                .put(handlers::update_observation)
                .delete(handlers::delete_observation),
        )
}
