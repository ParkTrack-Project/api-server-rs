mod support;

use api_server_rust::{
    create_routes,
    http::{permissions::GlobalRole, token::create_access_token},
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
const SECRET: &str = "occupancy-test-key";
const DETECTOR: &str = "test-car-detector-token";
const PREDICTION: &str = "test-ml-prediction-token";

async fn app(pool: PgPool) -> Router {
    let config = ApiConfig {
        car_detector_token: DETECTOR.into(),
        ml_prediction_token: PREDICTION.into(),
        database_url: pool.connect_options().to_url_lossy().to_string(),
        jwt_secret: SECRET.into(),
        jwt_expire_seconds: 3600,
        smtp_host: String::new(),
        smtp_port: 25,
        smtp_username: String::new(),
        smtp_password: String::new(),
        smtp_from_email: "sender@example.com".into(),
        smtp_from_name: "Auth".into(),
        smtp_use_tls: false,
        smtp_use_ssl: false,
        password_reset_return_token: false,
        password_reset_ttl_minutes: 15,
        password_reset_login_url: "https://example.com/login".into(),
    };
    let state = ApiState::new(config, DecodingKey::from_secret(SECRET.as_bytes()), pool).await;
    create_routes().with_state(state)
}

const START: &str = "2026-04-06T10:00:00Z";
const MIDDLE: &str = "2026-04-06T11:00:00Z";
const END: &str = "2026-04-06T12:00:00Z";

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    payload: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(format!("/api/v3/occupancy{path}"))
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(payload.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn observation(zone_id: i32, time: &str, occupied: i32) -> Value {
    json!({
        "zone_id": zone_id,
        "observed_at": time,
        "occupied": occupied,
        "confidence": 0.76,
        "confidence_level": "medium",
    })
}

fn admin() -> String {
    create_access_token(102, 3600, GlobalRole::Admin, SECRET)
}

async fn create(app: &Router, payload: Value, token: &str) -> String {
    let (status, created) = request(app, "POST", "/new", payload, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    format!("/{}", created["observation_id"])
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("zones", "occupancy"))
)]
async fn admin_manages_observations_and_server_owns_derived_fields(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let admin = admin();
    let mut payload = observation(1, START, 11);
    payload["source_type"] = json!("manual");
    payload["source_ref"] = json!("manual-1");
    payload["capacity"] = json!(12);
    payload["metadata"] = json!({"note": "counted manually"});
    payload["created_by_user_id"] = json!(101);
    let path = create(&app, payload.clone(), &admin).await;

    //Act
    let (status, original) = request(&app, "GET", &path, Value::Null, Some(&admin)).await;
    let (duplicate, _) = request(&app, "POST", "/new", payload, Some(&admin)).await;
    let (invalid, _) = request(&app, "PUT", &path, json!({"capacity": 10}), Some(&admin)).await;
    let (update_status, updated) = request(
        &app,
        "PUT",
        &path,
        json!({
            "capacity": 8, "occupied": 5, "metadata": null, "source_ref": null,
            "zone_id": 2, "source_type": "other", "free_count": 99, "created_by_user_id": 101,
        }),
        Some(&admin),
    )
    .await;
    let (detector_update, _) =
        request(&app, "PUT", &path, json!({"occupied": 1}), Some(DETECTOR)).await;
    let (deleted, body) = request(&app, "DELETE", &path, Value::Null, Some(&admin)).await;
    let (missing, _) = request(&app, "GET", &path, Value::Null, Some(&admin)).await;
    let (missing_delete, _) = request(&app, "DELETE", &path, Value::Null, Some(&admin)).await;

    //Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(original["capacity"], 12);
    assert_eq!(original["free_count"], 1);
    assert_eq!(original["created_by_user_id"], 102);
    assert_eq!(duplicate, StatusCode::CONFLICT);
    assert_eq!(invalid, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(update_status, StatusCode::OK, "{updated}");
    assert_eq!(updated["capacity"], 8);
    assert_eq!(updated["zone_id"], 1);
    assert_eq!(updated["free_count"], 3);
    assert_eq!(updated["source_type"], "manual");
    assert_eq!(updated["created_by_user_id"], 102);
    assert!(updated["metadata"].is_null());
    assert!(updated["source_ref"].is_null());
    assert_eq!(detector_update, StatusCode::NOT_FOUND);
    assert_eq!((deleted, body), (StatusCode::NO_CONTENT, Value::Null));
    assert_eq!(missing, StatusCode::NOT_FOUND);
    assert_eq!(missing_delete, StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("zones", "occupancy"))
)]
async fn lists_history_series_and_map_snapshots_with_filters(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    create(&app, observation(1, START, 3), DETECTOR).await;
    create(&app, observation(1, MIDDLE, 4), DETECTOR).await;
    create(&app, observation(1, MIDDLE, 5), DETECTOR).await;
    create(&app, observation(1, END, 6), DETECTOR).await;
    create(&app, observation(2, START, 1), DETECTOR).await;

    //Act
    let (history_status, history) = request(
        &app,
        "GET",
        &format!("?zone_id=1&camera_id=1&source_type=camera_cv&from={START}&to={END}"),
        Value::Null,
        Some(PREDICTION),
    )
    .await;
    let (series_status, series) = request(
        &app,
        "GET",
        &format!("?zone_id=1&view=series&at={MIDDLE}&latest_only=true"),
        Value::Null,
        Some(PREDICTION),
    )
    .await;
    let (map_status, map) = request(
        &app,
        "GET",
        &format!("?view=map&bbox=29,59,31,61&at={MIDDLE}"),
        Value::Null,
        Some(PREDICTION),
    )
    .await;
    let (latest_status, latest) = request(
        &app,
        "GET",
        "?view=map&top=1&offset=1",
        Value::Null,
        Some(PREDICTION),
    )
    .await;
    let (partner_status, partner) =
        request(&app, "GET", "?partner_id=1", Value::Null, Some(PREDICTION)).await;

    //Assert
    assert_eq!(history_status, StatusCode::OK, "{history}");
    assert_eq!(history.as_array().unwrap().len(), 3);
    assert_eq!(history[0]["observed_at"], START);
    assert_eq!(series_status, StatusCode::OK, "{series}");
    assert_eq!(
        series,
        json!([{
            "observed_at": MIDDLE, "occupied": 5, "free_count": 5, "capacity": 10,
            "confidence": 0.76, "confidence_level": "medium", "source_type": "car-detector"
        }])
    );
    assert_eq!(map_status, StatusCode::OK, "{map}");
    assert_eq!(map.as_array().unwrap().len(), 1);
    assert_eq!(map[0]["zone_id"], 1);
    assert_eq!(map[0]["observed_at"], MIDDLE);
    assert_eq!(map[0]["geometry"]["type"], "Polygon");
    assert_eq!(map[0]["location_type"], "street");
    assert_eq!(map[0]["pay"], 20);
    assert_eq!(map[0]["is_accessible"], true);
    assert_eq!(map[0]["is_active"], true);
    assert!(map[0].get("observation_id").is_none());
    assert_eq!(latest_status, StatusCode::OK, "{latest}");
    assert_eq!(latest.as_array().unwrap().len(), 1);
    assert_eq!(latest[0]["observed_at"], END);
    assert_eq!(partner_status, StatusCode::OK, "{partner}");
    assert_eq!(partner.as_array().unwrap().len(), 1);
    assert_eq!(partner[0]["zone_id"], 2);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("zones", "occupancy"))
)]
async fn enforces_access_and_validates_queries_and_capacity(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let user = create_access_token(101, 3600, GlobalRole::User, SECRET);
    let public = create(&app, observation(1, START, 1), DETECTOR).await;
    let partner = create(&app, observation(2, START, 2), DETECTOR).await;
    let mut invalid_source = observation(1, START, 1);
    invalid_source["source_type"] = json!("manual");

    //Act
    let (list_status, visible) = request(&app, "GET", "", Value::Null, Some(&user)).await;
    let cases = [
        (
            "GET",
            public.as_str(),
            Value::Null,
            Some(user.as_str()),
            StatusCode::OK,
        ),
        (
            "GET",
            partner.as_str(),
            Value::Null,
            Some(user.as_str()),
            StatusCode::NOT_FOUND,
        ),
        ("GET", "", Value::Null, None, StatusCode::UNAUTHORIZED),
        (
            "GET",
            "",
            Value::Null,
            Some(DETECTOR),
            StatusCode::FORBIDDEN,
        ),
        (
            "POST",
            "/new",
            observation(1, START, 2),
            Some(PREDICTION),
            StatusCode::FORBIDDEN,
        ),
        (
            "PUT",
            public.as_str(),
            json!({"occupied": 2}),
            Some(user.as_str()),
            StatusCode::FORBIDDEN,
        ),
        (
            "PUT",
            public.as_str(),
            json!({"occupied": 2}),
            Some(DETECTOR),
            StatusCode::OK,
        ),
        (
            "DELETE",
            public.as_str(),
            Value::Null,
            Some(DETECTOR),
            StatusCode::FORBIDDEN,
        ),
        (
            "GET",
            "?bbox=31,61,29,59",
            Value::Null,
            Some(&user),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "GET",
            "?view=unknown",
            Value::Null,
            Some(&user),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "POST",
            "/new",
            observation(1, START, 11),
            Some(DETECTOR),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "POST",
            "/new",
            observation(999, START, 1),
            Some(DETECTOR),
            StatusCode::NOT_FOUND,
        ),
        (
            "POST",
            "/new",
            invalid_source,
            Some(DETECTOR),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ];
    let mut actual = Vec::new();
    for (method, path, payload, token, _) in &cases {
        actual.push(request(&app, method, path, payload.clone(), *token).await);
    }

    //Assert
    assert_eq!(list_status, StatusCode::OK, "{visible}");
    assert_eq!(visible.as_array().unwrap().len(), 1);
    assert_eq!(visible[0]["zone_id"], 1);
    assert!(visible[0]["created_by_user_id"].is_null());
    for ((method, path, _, _, expected), (status, body)) in cases.into_iter().zip(actual) {
        assert_eq!(status, expected, "{method} {path}: {body}");
    }
}
