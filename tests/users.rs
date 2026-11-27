mod support;

use api_server_rust::{
    create_routes,
    features::users::repository::{UserAccess, find_user},
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

const JWT_SECRET: &str = "users-integration-test-signing-key";
const API_TOKEN: &str = "users-test-api-token";

async fn app(pool: PgPool) -> Router {
    let config = ApiConfig {
        api_token: API_TOKEN.into(),
        database_url: pool.connect_options().to_url_lossy().to_string(),
        jwt_secret: JWT_SECRET.into(),
        jwt_expire_seconds: 3600,
        smtp_host: String::new(),
        smtp_port: 25,
        smtp_username: String::new(),
        smtp_password: String::new(),
        smtp_from_email: "sender@example.com".into(),
        smtp_from_name: "Users Tests".into(),
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

fn token(user_id: u32, role: GlobalRole) -> String {
    create_access_token(user_id, 3600, role, JWT_SECRET)
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    payload: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(format!("/api/v3{path}"))
        .header("content-type", "application/json");
    if let Some(bearer) = bearer {
        builder = builder.header("authorization", format!("Bearer {bearer}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(payload.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, body)
}

async fn login(app: &Router, email: &str, password: &str) -> (StatusCode, Value) {
    request(
        app,
        "POST",
        "/auth/login",
        json!({"login": email, "password": password}),
        None,
    )
    .await
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth", "users"))
)]
async fn self_profile_and_admin_profile_include_membership_status(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let user = token(101, GlobalRole::User);
    let admin = token(102, GlobalRole::Admin);

    //Act
    let (me_status, me) = request(&app, "GET", "/users/me", Value::Null, Some(&user)).await;
    let (admin_status, profile) =
        request(&app, "GET", "/users/103", Value::Null, Some(&admin)).await;
    let (missing_status, _) = request(&app, "GET", "/users/999", Value::Null, Some(&admin)).await;

    //Assert
    assert_eq!(me_status, StatusCode::OK);
    assert_eq!(me["user"]["user_id"], 101);
    assert_eq!(me["user"]["global_role"], "user");
    assert_eq!(me["user"]["phone"], Value::Null);
    assert!(me["user"].get("hashed_password").is_none());
    assert!(me["user"].get("is_email_verified").is_none());
    assert_eq!(me["partner_memberships"][0]["read_scope"], "partner_all");
    assert_eq!(me["partner_memberships"][0]["is_active"], true);
    assert_eq!(admin_status, StatusCode::OK);
    assert_eq!(profile["user"]["is_active"], false);
    assert_eq!(profile["partner_memberships"][0]["is_active"], false);
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn self_update_uses_jwt_subject_and_only_profile_fields(pool: PgPool) {
    //Arrange
    let app = app(pool.clone()).await;
    let user = token(101, GlobalRole::User);
    let payload = json!({
        "full_name": "Updated User",
        "phone": "+79991234567",
        "user_id": 102,
        "email": "another@example.com",
        "global_role": "admin",
        "is_active": false
    });

    //Act
    let (status, body) = request(&app, "PUT", "/users/me", payload, Some(&user)).await;
    let (bad_phone, _) = request(
        &app,
        "PUT",
        "/users/me",
        json!({"phone":"invalid"}),
        Some(&user),
    )
    .await;
    let (long_name, _) = request(
        &app,
        "PUT",
        "/users/me",
        json!({"full_name": "a".repeat(201)}),
        Some(&user),
    )
    .await;
    let (empty, _) = request(&app, "PUT", "/users/me", json!({}), Some(&user)).await;
    let admin = find_user(&pool, 102, UserAccess::All)
        .await
        .unwrap()
        .unwrap();

    //Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], 101);
    assert_eq!(body["full_name"], "Updated User");
    assert_eq!(body["phone"], "+79991234567");
    assert_eq!(body["email"], "user@example.com");
    assert_eq!(body["global_role"], "user");
    assert_eq!(body["is_active"], true);
    assert_eq!(admin.full_name, "Test Admin");
    assert_eq!(bad_phone, StatusCode::BAD_REQUEST);
    assert_eq!(long_name, StatusCode::BAD_REQUEST);
    assert_eq!(empty, StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn authn_and_admin_permissions_are_required(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let user = token(101, GlobalRole::User);
    let routes = [
        ("GET", "/users/me", Value::Null, false),
        ("PUT", "/users/me", json!({"full_name":"Name"}), false),
        (
            "PUT",
            "/users/me/password",
            json!({"current_password":"current","new_password":"updated"}),
            false,
        ),
        ("GET", "/users", Value::Null, true),
        ("GET", "/users/101", Value::Null, true),
        (
            "POST",
            "/users",
            json!({"email":"new@example.com","password":"example-password"}),
            true,
        ),
        ("PUT", "/users/101", json!({"is_active":false}), true),
        ("DELETE", "/users/101", Value::Null, true),
    ];

    //Act + Assert
    for (method, path, payload, admin_only) in routes {
        for bearer in [None, Some(API_TOKEN)] {
            assert_eq!(
                request(&app, method, path, payload.clone(), bearer).await.0,
                StatusCode::UNAUTHORIZED,
                "{method} {path}"
            );
        }
        if admin_only {
            assert_eq!(
                request(&app, method, path, payload, Some(&user)).await.0,
                StatusCode::FORBIDDEN,
                "{method} {path}"
            );
        }
    }
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn list_uses_filters_and_counts_before_pagination(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let admin = token(102, GlobalRole::Admin);

    //Act
    let (status, page) = request(
        &app,
        "GET",
        "/users?q=EXAMPLE.COM&role=user&top=1&offset=1",
        Value::Null,
        Some(&admin),
    )
    .await;
    let (_, by_name) = request(
        &app,
        "GET",
        "/users?q=test%20admin&is_active=true",
        Value::Null,
        Some(&admin),
    )
    .await;
    let (_, inactive) = request(
        &app,
        "GET",
        "/users?is_active=false&offset=1",
        Value::Null,
        Some(&admin),
    )
    .await;
    let (invalid_top, _) = request(&app, "GET", "/users?top=101", Value::Null, Some(&admin)).await;
    let (invalid_offset, _) = request(
        &app,
        "GET",
        "/users?offset=9223372036854775808",
        Value::Null,
        Some(&admin),
    )
    .await;

    //Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["total"], 2);
    assert_eq!(page["top"], 1);
    assert_eq!(page["offset"], 1);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["user_id"], 103);
    assert_eq!(by_name["total"], 1);
    assert_eq!(by_name["items"][0]["user_id"], 102);
    assert_eq!(inactive["total"], 1);
    assert_eq!(inactive["items"], json!([]));
    assert_eq!(invalid_top, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_offset, StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn admin_creates_updates_and_deactivates_accounts(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let admin = token(102, GlobalRole::Admin);
    let payload = json!({
        "email": "new@example.com",
        "password": "example-original-password",
        "global_role": "admin",
        "is_active": false
    });

    //Act
    let (status, created) = request(&app, "POST", "/users", payload.clone(), Some(&admin)).await;
    let (duplicate, _) = request(&app, "POST", "/users", payload, Some(&admin)).await;
    let path = format!("/users/{}", created["user_id"]);
    let (initial_status, initial) = request(&app, "GET", &path, Value::Null, Some(&admin)).await;
    let (conflict, _) = request(
        &app,
        "PUT",
        &path,
        json!({"email":"user@example.com"}),
        Some(&admin),
    )
    .await;
    let (updated_status, updated) = request(&app, "PUT", &path, json!({
        "email":"renamed@example.com","full_name":"New Name","global_role":"user","is_active":true,
        "password":"must-not-be-applied"
    }), Some(&admin)).await;
    let (login_status, session) =
        login(&app, "renamed@example.com", "example-original-password").await;
    let bearer = session["access_token"].as_str();
    let (deactivated, body) = request(&app, "DELETE", &path, Value::Null, Some(&admin)).await;
    let (after_status, after) = request(&app, "GET", &path, Value::Null, Some(&admin)).await;
    let (blocked_token, _) = request(&app, "GET", "/auth/me", Value::Null, bearer).await;
    let (blocked_login, _) = login(&app, "renamed@example.com", "example-original-password").await;
    let (missing_delete, _) =
        request(&app, "DELETE", "/users/999", Value::Null, Some(&admin)).await;

    //Assert
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(duplicate, StatusCode::CONFLICT);
    assert_eq!(initial_status, StatusCode::OK);
    assert_eq!(initial["user"]["global_role"], "admin");
    assert_eq!(initial["user"]["is_active"], false);
    assert_eq!(conflict, StatusCode::CONFLICT);
    assert_eq!(updated_status, StatusCode::OK);
    assert_eq!(updated["is_active"], true);
    assert_eq!(updated["global_role"], "user");
    assert_eq!(updated["full_name"], "New Name");
    assert_eq!(login_status, StatusCode::OK);
    assert_eq!((deactivated, body), (StatusCode::NO_CONTENT, Value::Null));
    assert_eq!(after_status, StatusCode::OK);
    assert_eq!(after["user"]["is_active"], false);
    assert_eq!(blocked_token, StatusCode::FORBIDDEN);
    assert_eq!(blocked_login, StatusCode::UNAUTHORIZED);
    assert_eq!(missing_delete, StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn changed_role_rejects_jwt_with_old_privileges(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let old_admin = token(102, GlobalRole::Admin);

    //Act
    let (updated, _) = request(
        &app,
        "PUT",
        "/users/102",
        json!({"global_role":"user"}),
        Some(&old_admin),
    )
    .await;
    let (stale, _) = request(&app, "GET", "/users", Value::Null, Some(&old_admin)).await;
    let (current, _) = request(
        &app,
        "GET",
        "/users",
        Value::Null,
        Some(&token(102, GlobalRole::User)),
    )
    .await;

    //Assert
    assert_eq!(updated, StatusCode::OK);
    assert_eq!(stale, StatusCode::UNAUTHORIZED);
    assert_eq!(current, StatusCode::FORBIDDEN);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn password_change_checks_current_password_and_updates_login(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let (registered, session) = request(
        &app,
        "POST",
        "/auth/register",
        json!({
            "email":"password-test@example.com","password":"example-original-password"
        }),
        None,
    )
    .await;
    assert_eq!(registered, StatusCode::OK);
    let bearer = session["access_token"].as_str();
    let path = "/users/me/password";

    //Act
    let (wrong, _) = request(
        &app,
        "PUT",
        path,
        json!({"current_password":"incorrect","new_password":"example-new-password"}),
        bearer,
    )
    .await;
    let (short, _) = request(
        &app,
        "PUT",
        path,
        json!({"current_password":"example-original-password","new_password":"short"}),
        bearer,
    )
    .await;
    let (changed, body) = request(&app, "PUT", path, json!({"current_password":"example-original-password","new_password":"example-new-password"}), bearer).await;
    let (old, _) = login(
        &app,
        "password-test@example.com",
        "example-original-password",
    )
    .await;
    let (new, _) = login(&app, "password-test@example.com", "example-new-password").await;

    //Assert
    assert_eq!(wrong, StatusCode::FORBIDDEN);
    assert_eq!(short, StatusCode::BAD_REQUEST);
    assert_eq!((changed, body), (StatusCode::NO_CONTENT, Value::Null));
    assert_eq!(old, StatusCode::UNAUTHORIZED);
    assert_eq!(new, StatusCode::OK);
}
