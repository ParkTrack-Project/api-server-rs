use axum::{
    Router,
    routing::{get, post},
};

use crate::state::ApiState;

mod handlers;
mod models;
mod repository;
mod requests;
mod responses;
mod service;

pub fn camera_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(handlers::get_camera))
        .route("/next", get(handlers::next_camera))
        .route("/new", post(handlers::create_camera))
        .route(
            "/{camera_id}",
            get(handlers::get_camera)
                .put(handlers::update_camera)
                .delete(handlers::delete_camera),
        )
}
