use std::sync::LazyLock;

use axum::{
    Router, extract::{MatchedPath, Request}, http::HeaderName,
};
use regex::{Regex, RegexBuilder};
use tower::{ServiceBuilder};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer}, request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer}, trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{info_span, Level};

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

static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub fn apply_middleware(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
        // ID SETTER
            .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER.clone(), MakeRequestUuid))
        // TRACING
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request| {
                        let request_id = request
                            .headers()
                            .get(&REQUEST_ID_HEADER)
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("-");

                        let matched_path = request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str);

                        info_span!(
                            "http_request",
                            method = %request.method(),
                            path = matched_path,
                            request_id = %request_id,
                        )
                    })
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
        // ID PROPAGATOR
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER.clone()))
        // CORS
            .layer(
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
                    .allow_credentials(true),
            ),
    )
}
