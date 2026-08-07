use std::sync::LazyLock;

use axum::{
    RequestExt, Router,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, Error, authorization::Bearer},
};
use regex::{Regex, RegexBuilder};
use tower::{Layer, ServiceBuilder};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    trace::TraceLayer,
};

pub fn apply_middleware(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(configured_cors_layer()),
    )
}

fn configured_cors_layer() -> CorsLayer {
    static CORS_ORIGINS: [&str; 7] = [
        "https://swagger.parktrack.live",
        "https://swagger.dev.parktrack.live",
        "https://labeler.parktrack.live",
        "https://admin.parktrack.live",
        "https://parktrack.live",
        "https://dev.parktrack.live",
        "https://admin.dev.parktrack.live",
    ];

    static CORS_DEV_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        RegexBuilder::new(r"^https?://(localhost|127\.0\.0\.1)(:\d+)?$")
            .build()
            .unwrap()
    });

    CorsLayer::new()
        .allow_origin(
            CORS_ORIGINS
                .iter()
                .map(|x| x.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_origin(AllowOrigin::predicate(move |origin, _| {
            if let Ok(origin_str) = origin.to_str() {
                return CORS_DEV_REGEX.is_match(origin_str);
            }

            false
        }))
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true)
}
