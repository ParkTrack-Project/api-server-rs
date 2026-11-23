use crate::{cameras::partner_camera_routes, state::ApiState, zones::partner_zone_routes};
use axum::{Router, routing::get};

pub mod handlers;
pub mod member_repository;
pub mod models;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn partner_routes() -> Router<ApiState> {
    Router::new()
        .nest(
            "/{partner_id}/cameras",
            partner_camera_routes(),
        )
        .nest("/{partner_id}/zones", partner_zone_routes())
        .route(
            "/{partner_id}/members",
            get(handlers::list_members).post(handlers::create_member),
        )
        .route(
            "/{partner_id}/members/{user_id}",
            get(handlers::get_member)
                .put(handlers::update_member)
                .delete(handlers::disable_member),
        )
        .route("/", get(handlers::list).post(handlers::create))
        .route(
            "/{partner_id}",
            get(handlers::get)
                .put(handlers::update)
                .delete(handlers::disable),
        )
}
