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
const SECRET: &str = "partner-resources-key";
async fn app(pool: PgPool) -> Router {
    let config = ApiConfig {
        api_token: "test-api-token".into(),
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

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
    user: Option<u32>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(format!("/api/v3{path}"))
        .header("content-type", "application/json");
    if let Some(id) = user {
        request = request.header(
            "authorization",
            format!(
                "Bearer {}",
                create_access_token(
                    id,
                    3600,
                    if id == 102 {
                        GlobalRole::Admin
                    } else {
                        GlobalRole::User
                    },
                    SECRET
                )
            ),
        );
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
fn camera() -> Value {
    json!({"title":"Created Camera","source":"rtsp://new.example.com/stream","image_width":1920,"image_height":1080,"latitude":60.,"longitude":30.,"partner_id":102,"created_by_user_id":103})
}
fn zone(camera_id: i32) -> Value {
    json!({"camera_id":camera_id,"zone_type":"standard","capacity":10,"pay":0,"geometry":{"type":"Polygon","coordinates":[[[30,60],[31,60],[31,61],[30,60]]]},"image_polygon":[[0,0],[10,0],[10,10],[0,10]],"partner_id":102,"created_by_user_id":103})
}
fn items(body: &Value) -> &Vec<Value> {
    body.as_array()
        .or_else(|| body["Full"].as_array())
        .or_else(|| body["Map"].as_array())
        .unwrap()
}
fn update(feature: &str) -> Value {
    if feature == "cameras" {
        json!({"title":"Changed"})
    } else {
        json!({"pay":20})
    }
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("partner_resources"))
)]
async fn own_scope_filters_every_resource_operation(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    for feature in ["cameras", "zones"] {
        let base = format!("/partners/101/{feature}");
        //Act
        let (status, body) = request(&app, "GET", &base, Value::Null, Some(101)).await;
        //Assert
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(items(&body).len(), 1);
        assert_eq!(
            request(&app, "GET", &format!("{base}/101"), Value::Null, Some(101))
                .await
                .0,
            StatusCode::OK
        );
        for id in [102, 103, 104, 105, 999] {
            for method in ["GET", "PUT", "DELETE"] {
                assert_eq!(
                    request(
                        &app,
                        method,
                        &format!("{base}/{id}"),
                        update(feature),
                        Some(101)
                    )
                    .await
                    .0,
                    StatusCode::NOT_FOUND,
                    "{feature} {method} {id}"
                );
            }
        }
        assert_eq!(
            request(
                &app,
                "PUT",
                &format!("{base}/101"),
                update(feature),
                Some(101)
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    //Act
    let zone = request(
        &app,
        "DELETE",
        "/partners/101/zones/101",
        Value::Null,
        Some(101),
    )
    .await;
    let camera = request(
        &app,
        "DELETE",
        "/partners/101/cameras/101",
        Value::Null,
        Some(101),
    )
    .await;
    //Assert
    assert_eq!(zone.0, StatusCode::NO_CONTENT);
    assert_eq!(camera.0, StatusCode::NO_CONTENT);
}
#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("partner_resources"))
)]
async fn partner_all_and_admin_remain_tenant_bound(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    for user in [103, 102] {
        for feature in ["cameras", "zones"] {
            let base = format!("/partners/101/{feature}");
            //Act
            let (status, body) = request(&app, "GET", &base, Value::Null, Some(user)).await;
            //Assert
            assert_eq!(status, StatusCode::OK, "{body}");
            assert_eq!(items(&body).len(), 3);
            for id in [101, 102, 105] {
                assert_eq!(
                    request(
                        &app,
                        "GET",
                        &format!("{base}/{id}"),
                        Value::Null,
                        Some(user)
                    )
                    .await
                    .0,
                    StatusCode::OK
                );
            }
            for method in ["GET", "PUT", "DELETE"] {
                assert_eq!(
                    request(
                        &app,
                        method,
                        &format!("{base}/103"),
                        update(feature),
                        Some(user)
                    )
                    .await
                    .0,
                    StatusCode::NOT_FOUND
                );
            }
            assert_eq!(
                request(
                    &app,
                    "PUT",
                    &format!("{base}/102"),
                    update(feature),
                    Some(user)
                )
                .await
                .0,
                StatusCode::OK
            );
        }
    }
    //Act + Assert
    for feature in ["cameras", "zones"] {
        assert_eq!(
            request(
                &app,
                "GET",
                &format!("/partners/999/{feature}"),
                Value::Null,
                Some(102)
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        request(
            &app,
            "GET",
            "/partners/101/zones?partner_id=102",
            Value::Null,
            Some(103)
        )
        .await
        .1,
        json!([])
    );
    let (_, mapped) = request(
        &app,
        "GET",
        "/partners/101/cameras?view=map&q=Own",
        Value::Null,
        Some(103),
    )
    .await;
    assert_eq!(items(&mapped).len(), 1);
    let (_, mapped) = request(
        &app,
        "GET",
        "/partners/101/zones?view=map&camera_id=101&bbox=30,60,31,61",
        Value::Null,
        Some(103),
    )
    .await;
    assert_eq!(items(&mapped).len(), 1);
    assert!(items(&mapped)[0].get("image_polygon").is_none());
}
#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("partner_resources"))
)]
async fn denied_memberships_permissions_and_none_scope(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    for feature in ["cameras", "zones"] {
        let base = format!("/partners/101/{feature}");
        let create = if feature == "cameras" {
            camera()
        } else {
            zone(101)
        };
        for user in [None, Some(104), Some(105), Some(107), Some(108)] {
            for (method, suffix) in [
                ("GET", ""),
                ("GET", "/101"),
                ("POST", "/new"),
                ("PUT", "/101"),
                ("DELETE", "/101"),
            ] {
                //Act
                let (status, body) = request(
                    &app,
                    method,
                    &format!("{base}{suffix}"),
                    if method == "POST" {
                        create.clone()
                    } else {
                        update(feature)
                    },
                    user,
                )
                .await;
                //Assert
                assert_eq!(
                    status,
                    if user.is_none() {
                        StatusCode::UNAUTHORIZED
                    } else {
                        StatusCode::FORBIDDEN
                    },
                    "{feature} {method} {user:?}: {body}"
                );
            }
        }
        //Act + Assert
        assert_eq!(
            request(&app, "GET", &base, Value::Null, Some(106)).await.0,
            StatusCode::OK
        );
        for (method, suffix) in [("POST", "/new"), ("PUT", "/101"), ("DELETE", "/101")] {
            assert_eq!(
                request(
                    &app,
                    method,
                    &format!("{base}{suffix}"),
                    create.clone(),
                    Some(106)
                )
                .await
                .0,
                StatusCode::FORBIDDEN
            );
        }
        assert_eq!(
            request(
                &app,
                "GET",
                &format!("/partners/103/{feature}"),
                Value::Null,
                Some(101)
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
}
#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("partner_resources"))
)]
async fn creation_sets_tenant_and_actor_and_validates_zone_camera(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let (cs, created) = request(
        &app,
        "POST",
        "/partners/101/cameras/new",
        camera(),
        Some(101),
    )
    .await;
    let camera_id = created["camera_id"].as_i64().unwrap();
    let (_, stored) = request(
        &app,
        "GET",
        &format!("/partners/101/cameras/{camera_id}"),
        Value::Null,
        Some(101),
    )
    .await;
    let (zs, created_zone) = request(
        &app,
        "POST",
        "/partners/101/zones/new",
        zone(102),
        Some(101),
    )
    .await;
    let zone_id = created_zone["zone_id"].as_i64().unwrap();
    let (_, stored_zone) = request(
        &app,
        "GET",
        &format!("/partners/101/zones/{zone_id}"),
        Value::Null,
        Some(101),
    )
    .await;
    //Assert
    assert_eq!(cs, StatusCode::CREATED, "{created}");
    assert_eq!(zs, StatusCode::CREATED, "{created_zone}");
    for body in [stored, stored_zone] {
        assert_eq!(body["partner_id"], 101);
        assert_eq!(body["created_by_user_id"], 101);
    }
    for id in [103, 104, 999] {
        for user in [101, 102] {
            assert_eq!(
                request(
                    &app,
                    "POST",
                    "/partners/101/zones/new",
                    zone(id),
                    Some(user)
                )
                .await
                .0,
                StatusCode::NOT_FOUND
            );
            assert_eq!(
                request(
                    &app,
                    "PUT",
                    &format!("/partners/101/zones/{zone_id}"),
                    json!({"camera_id":id}),
                    Some(user)
                )
                .await
                .0,
                StatusCode::NOT_FOUND
            );
        }
    }
    let (status, moved) = request(
        &app,
        "PUT",
        &format!("/partners/101/zones/{zone_id}"),
        json!({"camera_id":camera_id,"partner_id":102,"created_by_user_id":103}),
        Some(101),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{moved}");
    assert_eq!(moved["partner_id"], 101);
    assert_eq!(moved["created_by_user_id"], 101);
    assert_eq!(moved["camera_id"], camera_id);
}
#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("partner_resources"))
)]
async fn operation_scopes_are_independent(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    request(
        &app,
        "PUT",
        "/partners/101/members/101",
        json!({"read_scope":"none","write_scope":"partner_all","delete_scope":"own"}),
        Some(102),
    )
    .await;
    for feature in ["cameras", "zones"] {
        let base = format!("/partners/101/{feature}");
        //Act
        let read = request(&app, "GET", &format!("{base}/102"), Value::Null, Some(101)).await;
        let write = request(
            &app,
            "PUT",
            &format!("{base}/102"),
            update(feature),
            Some(101),
        )
        .await;
        let delete = request(
            &app,
            "DELETE",
            &format!("{base}/102"),
            Value::Null,
            Some(101),
        )
        .await;
        //Assert
        assert_eq!(read.0, StatusCode::FORBIDDEN);
        assert_eq!(write.0, StatusCode::OK, "{}", write.1);
        assert_eq!(delete.0, StatusCode::NOT_FOUND);
    }
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("partner_resources"))
)]
async fn api_tokens_and_logged_out_tokens_cannot_access_partner_resources(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let token = create_access_token(103, 3600, GlobalRole::User, SECRET);
    //Act
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v3/auth/logout")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    //Assert
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    for bearer in ["test-api-token", token.as_str()] {
        for feature in ["cameras", "zones"] {
            //Act
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v3/partners/101/{feature}"))
                        .header("authorization", format!("Bearer {bearer}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            //Assert
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }
}
