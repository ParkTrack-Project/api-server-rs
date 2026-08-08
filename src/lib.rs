use axum::Router;

use crate::state::ApiState;

mod error;
pub mod http;
mod state;
mod types;

mod cameras;

pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .nest(
            "/api/v3",
            Router::new()
                .nest("/cameras", cameras::camera_routes())
        )
}