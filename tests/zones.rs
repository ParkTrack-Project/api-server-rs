use api_server_rust::{
    create_routes,
    http::{permissions::GlobalRole, token::create_access_token},
    state::{ApiConfig, ApiState},
    zones::repository::{self, ZoneAccess},
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use jsonwebtoken::DecodingKey;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
const SECRET: &str = "zones-test-key";
async fn app(pool: PgPool) -> Router {
    let config = ApiConfig {
        api_token: "test-api-token".into(),
        database_url: String::new(),
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

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    payload: Value,
    role: Option<GlobalRole>,
) -> (StatusCode, Value) {
    let mut r = Request::builder()
        .method(method)
        .uri(format!("/api/v3/zones{path}"))
        .header("content-type", "application/json");
    if let Some(role) = role {
        let id = if role == GlobalRole::Admin { 102 } else { 101 };
        r = r.header(
            "authorization",
            format!("Bearer {}", create_access_token(id, 3600, role, SECRET)),
        );
    }
    let response = app
        .clone()
        .oneshot(r.body(Body::from(payload.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
fn payload(camera: i32) -> Value {
    json!({
        "camera_id":camera,
        "zone_type":"standard",
        "capacity":10,
        "pay":0,
        "geometry":{
            "type":"Polygon",
            "coordinates":[[[30.,60.],[31.,60.],[31.,61.],[30.,60.]]]
        },
        "image_polygon":[[0,0],[10,0],[10,10],[0,10]],
        "location_type":"street",
        "is_private":false,
        "is_accessible":false
    })
}
async fn create(app: &Router, camera: i32) -> i64 {
    let (s, b) = request(
        app,
        "POST",
        "/new",
        payload(camera),
        Some(GlobalRole::Admin),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "{b}");
    b["zone_id"].as_i64().unwrap()
}

#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn zone_crud_and_occupancy(pool: PgPool) {
    // Arrange
    let app = app(pool).await;
    // Act
    let id = create(&app, 1).await;
    let path = format!("/{id}");
    let (s, initial) = request(&app, "GET", &path, Value::Null, Some(GlobalRole::User)).await;
    let (updated, body) = request(
        &app,
        "PUT",
        &path,
        json!({"occupied":4,"confidence":0.8}),
        Some(GlobalRole::Admin),
    )
    .await;
    // Assert
    assert_eq!(s, StatusCode::OK);
    assert_eq!(initial["free_count"], 10);
    assert!(initial["occupancy_updated_at"].is_null());
    assert_eq!(updated, StatusCode::OK, "{body}");
    assert_eq!(body["free_count"], 6);
    assert!(body["occupancy_updated_at"].is_string());
    assert_eq!(body["created_by_user_id"], 102);
    assert_eq!(
        request(
            &app,
            "PUT",
            &path,
            json!({"capacity":3}),
            Some(GlobalRole::Admin)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &app,
            "PUT",
            &path,
            json!({"occupied":11}),
            Some(GlobalRole::Admin)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(&app, "DELETE", &path, Value::Null, Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, "GET", &path, Value::Null, Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&app, "DELETE", &path, Value::Null, Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn permissions_and_partner_visibility(pool: PgPool) {
    // Arrange
    let app = app(pool).await;
    let id = create(&app, 7).await;
    // Act
    let (s, body) = request(&app, "GET", "", Value::Null, Some(GlobalRole::User)).await;
    // Assert
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body, json!([]));
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("/{id}"),
            Value::Null,
            Some(GlobalRole::User)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&app, "POST", "/new", payload(1), Some(GlobalRole::User))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    for (method, path) in [
        ("GET", ""),
        ("GET", "/1"),
        ("POST", "/new"),
        ("PUT", "/1"),
        ("DELETE", "/1"),
    ] {
        assert_eq!(
            request(&app, method, path, payload(1), None).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        request(
            &app,
            "PUT",
            &format!("/{id}"),
            json!({"pay":5}),
            Some(GlobalRole::User)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &app,
            "DELETE",
            &format!("/{id}"),
            Value::Null,
            Some(GlobalRole::User)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn list_filters_and_map_projection(pool: PgPool) {
    // Arrange
    let app = app(pool).await;
    let id = create(&app, 1).await;
    create(&app, 7).await;
    // Act
    let(s,body)=request(&app,"GET","?camera_id=1&view=map&min_free_count=2&min_confidence=0&max_pay=0&include_private=false&include_accessible=false&hide_location_types=yard&bbox=30,60,32,62",Value::Null,Some(GlobalRole::Admin)).await;
    // Assert
    assert_eq!(s, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["zone_id"], id);
    assert!(body[0].get("image_polygon").is_none());
    assert!(body[0].get("camera_id").is_none());
    for query in [
        "?camera_id=2",
        "?partner_id=101",
        "?is_active=false",
        "?min_free_count=11",
        "?min_confidence=0.1",
        "?hide_location_types=street",
        "?bbox=0,0,1,1",
    ] {
        assert_eq!(
            request(&app, "GET", query, Value::Null, Some(GlobalRole::Admin))
                .await
                .1,
            json!([])
        );
    }
    request(
        &app,
        "PUT",
        &format!("/{id}"),
        json!({"geometry":{"type":"Polygon","coordinates":[[[0.,0.],[1.,0.],[1.,1.],[0.,0.]]]}}),
        Some(GlobalRole::Admin),
    )
    .await;
    assert_eq!(
        request(
            &app,
            "GET",
            "?camera_id=1&bbox=0,0,1,1",
            Value::Null,
            Some(GlobalRole::Admin)
        )
        .await
        .1
        .as_array()
        .unwrap()
        .len(),
        1
    );
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn validation_and_missing_camera(pool: PgPool) {
    // Arrange
    let app = app(pool).await;
    // Act + Assert
    for (key, value) in [
        ("camera_id", json!(0)),
        ("capacity", json!(-1)),
        ("pay", json!(-1)),
        ("zone_type", json!("invalid")),
        ("location_type", json!("invalid")),
        ("geometry", json!({})),
        ("image_polygon", json!([[0, 0], [1, 1]])),
    ] {
        let mut p = payload(1);
        p[key] = value;
        assert!(
            !request(&app, "POST", "/new", p, Some(GlobalRole::Admin))
                .await
                .0
                .is_success()
        );
    }
    assert_eq!(
        request(&app, "POST", "/new", payload(999), Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    for query in [
        "?bbox=1,2,3",
        "?bbox=3,2,1,4",
        "?min_confidence=2",
        "?hide_location_types=invalid",
    ] {
        assert!(
            !request(&app, "GET", query, Value::Null, Some(GlobalRole::Admin))
                .await
                .0
                .is_success()
        );
    }
    let mut p = payload(1);
    p["capacity"] = json!(0);
    assert_eq!(
        request(&app, "POST", "/new", p, Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::CREATED
    );
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn repository_scopes(pool: PgPool) {
    // Arrange
    let app = app(pool.clone()).await;
    let id = create(&app, 7).await as i32;
    // Act
    let hidden = repository::find(&pool, id, ZoneAccess::Public)
        .await
        .unwrap();
    let visible = repository::find(&pool, id, ZoneAccess::Partner { partner_id: 1 })
        .await
        .unwrap();
    // Assert
    assert!(hidden.is_none());
    assert!(visible.is_some());
    assert!(
        repository::find(
            &pool,
            id,
            ZoneAccess::PartnerOwned {
                partner_id: 1,
                user_id: 101
            }
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(
        repository::find(
            &pool,
            id,
            ZoneAccess::PartnerOwned {
                partner_id: 1,
                user_id: 102
            }
        )
        .await
        .unwrap()
        .is_some()
    );
    assert_eq!(
        repository::delete_zone(&pool, id, ZoneAccess::PublicOwned { user_id: 102 })
            .await
            .unwrap(),
        0
    );
}

#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn nullable_filters_and_server_owned_fields(pool: PgPool) {
    // Arrange
    let app = app(pool).await;
    let mut input = payload(1);
    input.as_object_mut().unwrap().remove("is_private");
    input.as_object_mut().unwrap().remove("is_accessible");
    input.as_object_mut().unwrap().remove("location_type");
    input["partner_id"] = json!(1);
    input["created_by_user_id"] = json!(101);
    // Act
    let (status, created) = request(&app, "POST", "/new", input, Some(GlobalRole::Admin)).await;
    let path = format!("/{}", created["zone_id"]);
    let (_, body) = request(&app, "GET", &path, Value::Null, Some(GlobalRole::Admin)).await;
    // Assert
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["partner_id"].is_null());
    assert_eq!(body["created_by_user_id"], 102);
    assert!(body["is_private"].is_null());
    for query in [
        "?include_private=true",
        "?include_private=false",
        "?include_accessible=true",
        "?include_accessible=false",
        "?hide_location_types=yard",
    ] {
        assert_eq!(
            request(&app, "GET", query, Value::Null, Some(GlobalRole::Admin))
                .await
                .1,
            json!([])
        );
    }
    assert_eq!(
        request(&app, "PUT", &path, json!({}), Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &app,
            "PUT",
            &path,
            json!({"camera_id":999}),
            Some(GlobalRole::Admin)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, updated) = request(
        &app,
        "PUT",
        &path,
        json!({"pay":20,"free_count":999,"partner_id":1,"created_by_user_id":101}),
        Some(GlobalRole::Admin),
    )
    .await;
    assert_eq!(updated["free_count"], 10);
    assert!(updated["partner_id"].is_null());
    assert_eq!(updated["created_by_user_id"], 102);
}

#[sqlx::test(fixtures(path = "../fixtures", scripts("zones")))]
async fn concurrent_updates_preserve_capacity_invariant(pool: PgPool) {
    // Arrange
    let app = app(pool).await;
    let id = create(&app, 1).await;
    let path = format!("/{id}");
    // Act
    let (first, second) = tokio::join!(
        request(
            &app,
            "PUT",
            &path,
            json!({"occupied":8}),
            Some(GlobalRole::Admin)
        ),
        request(
            &app,
            "PUT",
            &path,
            json!({"capacity":3}),
            Some(GlobalRole::Admin)
        ),
    );
    let (_, zone) = request(&app, "GET", &path, Value::Null, Some(GlobalRole::Admin)).await;
    // Assert
    assert!([first.0, second.0].contains(&StatusCode::BAD_REQUEST));
    assert!([first.0, second.0].contains(&StatusCode::OK));
    assert!(zone["occupied"].as_i64().unwrap() <= zone["capacity"].as_i64().unwrap());
}
