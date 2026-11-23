use axum::{
    Router,
    routing::{get, post},
};

use crate::state::ApiState;

pub mod handlers;
pub mod models;
pub mod partners;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn camera_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(handlers::list_cameras))
        .route("/next", get(handlers::next_camera))
        .route("/new", post(handlers::create_camera))
        .route(
            "/{camera_id}",
            get(handlers::get_camera)
                .put(handlers::update_camera)
                .delete(handlers::delete_camera),
        )
}

pub fn partner_camera_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(partners::list))
        .route("/new", post(partners::create))
        .route(
            "/{camera_id}",
            get(partners::get_one)
                .put(partners::update)
                .delete(partners::delete),
        )
}
