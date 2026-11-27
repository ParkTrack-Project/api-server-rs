use axum::{
    Router,
    routing::{get, put},
};

use crate::state::ApiState;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn user_routes() -> Router<ApiState> {
    Router::new()
        .route("/", get(handlers::list_users).post(handlers::create_user))
        .route("/me", get(handlers::get_me).put(handlers::update_me))
        .route("/me/password", put(handlers::change_my_password))
        .route(
            "/{user_id}",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::deactivate_user),
        )
}
