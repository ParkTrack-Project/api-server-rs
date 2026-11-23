use crate::state::ApiState;
use axum::{
    Router,
    routing::{get, post},
};
pub mod handlers;
pub mod models;
pub mod partners;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;
pub fn zone_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(handlers::list))
        .route("/new", post(handlers::create))
        .route(
            "/{zone_id}",
            get(handlers::get)
                .put(handlers::update)
                .delete(handlers::delete),
        )
}

pub fn partner_zone_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(partners::list))
        .route("/new", post(partners::create))
        .route(
            "/{zone_id}",
            get(partners::get_one)
                .put(partners::update)
                .delete(partners::delete),
        )
}
