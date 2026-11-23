use axum::Router;

use crate::state::ApiState;

pub mod error;
pub mod http;
pub mod state;
pub mod types;

pub mod auth;
pub mod cameras;
pub mod partners;
pub mod users;
pub mod utils;
pub mod zones;

pub fn create_routes() -> Router<ApiState> {
    Router::new().nest(
        "/api/v3",
        Router::new()
            .nest("/auth", auth::auth_routes())
            .nest("/cameras", cameras::camera_routes())
            .nest("/zones", zones::zone_routes())
            .nest("/partners", partners::partner_routes())
            .merge(http::system_routes()),
    )
}
