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
        .route("/", get(handlers::list_zones))
        .route("/new", post(handlers::create_zone))
        .route(
            "/{zone_id}",
            get(handlers::get_zone)
                .put(handlers::update_zone)
                .delete(handlers::delete_zone),
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
