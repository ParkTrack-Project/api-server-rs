use axum::{Router, routing::get};

use crate::state::ApiState;

pub mod error;
pub mod http;
pub mod state;
pub mod types;

pub mod cameras;

pub fn create_routes() -> Router<ApiState> {
    Router::new().nest(
        "/api/v3",
        Router::new()
            .nest("/cameras", cameras::camera_routes())
            .merge(http::system_routes()),
    )
}
