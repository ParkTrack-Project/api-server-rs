use api_server_rust::{create_routes, state::ApiState};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Сетапится через переменную RUST_LOG
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,sqlx=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let state = ApiState::default().await;
    let app = create_routes().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}
