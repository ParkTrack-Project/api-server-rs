pub mod handlers;
pub mod models;
pub mod repository;
pub mod requests;
pub mod responses;
pub mod service;

pub fn auth_routes() -> axum::Router<crate::state::ApiState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route(
            "/password-reset/request",
            post(handlers::reset_password_request),
        )
        .route(
            "/password-reset/confirm",
            post(handlers::reset_password_confirm),
        )
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::get_me))
}
