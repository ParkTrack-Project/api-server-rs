mod support;

use api_server_rust::{
    create_routes,
    state::{ApiConfig, ApiState},
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use jsonwebtoken::DecodingKey;
use serde_json::{Value, json};
use sqlx::{ConnectOptions, PgPool};
use tower::ServiceExt;

const JWT_SECRET: &str = "weather-integration-test-signing-key";
const DETECTOR: &str = "weather-test-detector";
const ML: &str = "weather-test-ml";

async fn app(pool: PgPool) -> Router {
    let config = ApiConfig {
        car_detector_token: DETECTOR.into(),
        ml_prediction_token: ML.into(),
        database_url: pool.connect_options().to_url_lossy().to_string(),
        jwt_secret: JWT_SECRET.into(),
        jwt_expire_seconds: 3600,
        smtp_host: String::new(),
        smtp_port: 25,
        smtp_username: String::new(),
        smtp_password: String::new(),
        smtp_from_email: "sender@example.com".into(),
        smtp_from_name: "Weather Tests".into(),
        smtp_use_tls: false,
        smtp_use_ssl: false,
        password_reset_return_token: false,
        password_reset_ttl_minutes: 15,
        password_reset_login_url: String::new(),
    };
    let state = ApiState::new(
        config,
        DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        pool,
    )
    .await;
    create_routes().with_state(state)
}

const HOUR_10: &str = "2026-04-06T10:00:00Z";
const HOUR_11: &str = "2026-04-06T11:00:00Z";
const HOUR_12: &str = "2026-04-06T12:00:00Z";
const HOUR_13: &str = "2026-04-06T13:00:00Z";
const HOUR_14: &str = "2026-04-06T14:00:00Z";
const OBSERVATION: &str = r#"{"temperature":7.4,"precipitation":0.8}"#;

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: &str,
    bearer: Option<&str>,
    content_type: &str,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(format!("/api/v3/weather{path}"))
        .header("content-type", content_type);
    if let Some(bearer) = bearer {
        builder = builder.header("authorization", format!("Bearer {bearer}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    (status, json)
}

async fn get(app: &Router, path: &str) -> (StatusCode, Value) {
    request(app, "GET", path, "", Some(ML), "application/json").await
}

async fn put(app: &Router, path: &str, body: &str) -> (StatusCode, Value) {
    request(app, "PUT", path, body, Some(ML), "application/json").await
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("cameras", "weather"))
)]
async fn list_filters_before_selecting_latest_and_paginates_after(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let interval = format!("from={HOUR_10}&to={HOUR_13}");

    //Act
    let (status, filtered) = get(&app, &format!("?camera_id=1&{interval}")).await;
    let (latest_status, latest) = get(&app, &format!("?{interval}&latest_only=true")).await;
    let (page_status, page) = get(
        &app,
        &format!("?{interval}&latest_only=true&top=1&offset=1"),
    )
    .await;

    //Assert
    assert_eq!(status, StatusCode::OK, "{filtered}");
    let filtered = filtered.as_array().unwrap();
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0]["observed_at"], HOUR_10);
    assert_eq!(filtered[1]["observed_at"], HOUR_11);
    assert_eq!(latest_status, StatusCode::OK, "{latest}");
    assert_eq!(latest.as_array().unwrap().len(), 2);
    assert_eq!(latest[0]["camera_id"], 1);
    assert_eq!(latest[0]["observed_at"], HOUR_11);
    assert_eq!(latest[1]["camera_id"], 2);
    assert_eq!(latest[1]["observed_at"], HOUR_12);
    assert_eq!(page_status, StatusCode::OK, "{page}");
    assert_eq!(page, json!([latest[1]]));
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("cameras", "weather"))
)]
async fn get_observation_by_composite_key(pool: PgPool) {
    //Arrange
    let app = app(pool).await;

    //Act
    let (status, observation) = get(&app, &format!("/1/{HOUR_10}")).await;
    let (missing, _) = get(&app, &format!("/2/{HOUR_11}")).await;

    //Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        observation,
        json!({"camera_id": 1, "observed_at": HOUR_10, "temperature": 7.4, "precipitation": 0.8})
    );
    assert_eq!(missing, StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("cameras"))
)]
async fn put_creates_and_updates_the_same_observation(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let path = format!("/1/{HOUR_14}");

    //Act
    let created = put(&app, &path, OBSERVATION).await;
    let updated = put(&app, &path, r#"{"temperature":-2.5,"precipitation":0}"#).await;
    let (status, observation) = get(&app, &path).await;
    let (missing_camera, _) = put(&app, &format!("/9999/{HOUR_14}"), OBSERVATION).await;

    //Assert
    let key = json!({"camera_id": 1, "observed_at": HOUR_14});
    assert_eq!(created, (StatusCode::CREATED, key.clone()));
    assert_eq!(updated, (StatusCode::OK, key));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(observation["temperature"], -2.5);
    assert_eq!(observation["precipitation"], 0.0);
    assert_eq!(missing_camera, StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("cameras"))
)]
async fn concurrent_put_creates_exactly_one_observation(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let path = format!("/1/{HOUR_14}");

    //Act
    let (first, second) = tokio::join!(
        put(&app, &path, OBSERVATION),
        put(&app, &path, r#"{"temperature":10,"precipitation":2}"#)
    );
    let (list_status, observations) = get(&app, "?camera_id=1").await;

    //Assert
    let key = json!({"camera_id": 1, "observed_at": HOUR_14});
    assert_eq!(first.1, key);
    assert_eq!(second.1, key);
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(observations.as_array().unwrap().len(), 1);
    let winner = if second.0 == StatusCode::OK {
        assert_eq!(first.0, StatusCode::CREATED);
        (10.0, 2.0)
    } else {
        assert_eq!(second.0, StatusCode::CREATED);
        assert_eq!(first.0, StatusCode::OK);
        (7.4, 0.8)
    };
    assert_eq!(observations[0]["temperature"], winner.0);
    assert_eq!(observations[0]["precipitation"], winner.1);
}

#[sqlx::test(migrator = "crate::support::MIGRATOR")]
async fn rejects_bad_requests_and_requires_weather_permissions(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let path = format!("/1/{HOUR_14}");
    let routes = [("GET", ""), ("GET", path.as_str()), ("PUT", path.as_str())];

    //Act
    let mut access_statuses = Vec::new();
    for (method, path) in routes {
        for token in [None, Some("invalid"), Some(DETECTOR)] {
            access_statuses.push(
                request(&app, method, path, OBSERVATION, token, "application/json")
                    .await
                    .0,
            );
        }
    }
    let (old_route, _) = request(
        &app,
        "POST",
        "/new",
        OBSERVATION,
        Some(ML),
        "application/json",
    )
    .await;
    let (media_type, _) = request(&app, "PUT", &path, OBSERVATION, Some(ML), "text/plain").await;
    let (invalid_json, _) = put(&app, &path, "{").await;
    let (wrong_type, _) = put(&app, &path, r#"{"temperature":"cold","precipitation":0}"#).await;
    let (negative, _) = put(&app, &path, r#"{"temperature":7.4,"precipitation":-0.1}"#).await;
    let (off_hour, _) = put(&app, "/1/2026-04-06T10:00:00.001Z", OBSERVATION).await;
    let (leap_second, _) = get(&app, "/1/2026-04-06T09:59:60Z").await;
    let (bad_timestamp, _) = get(&app, "/1/yesterday").await;
    let (bad_camera_id, _) = get(&app, &format!("/0/{HOUR_14}")).await;
    let (bad_query, _) = get(&app, &format!("?from={HOUR_11}&to={HOUR_10}")).await;

    //Assert
    assert_eq!(
        access_statuses,
        [
            StatusCode::UNAUTHORIZED,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
        ]
        .repeat(3)
    );
    assert_eq!(old_route, StatusCode::NOT_FOUND);
    assert_eq!(media_type, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(invalid_json, StatusCode::BAD_REQUEST);
    assert_eq!(wrong_type, StatusCode::BAD_REQUEST);
    for status in [
        negative,
        off_hour,
        leap_second,
        bad_timestamp,
        bad_camera_id,
        bad_query,
    ] {
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
