use api_server_rust::{create_routes, state::ApiState};
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

#[tokio::main]
async fn main() {
    let state = ApiState::default().await;
    let app = create_routes().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}
