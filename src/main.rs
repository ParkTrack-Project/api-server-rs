use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(test_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}

async fn test_handler() -> (StatusCode, impl IntoResponse) {
    (StatusCode::NOT_FOUND, "Help")
}
