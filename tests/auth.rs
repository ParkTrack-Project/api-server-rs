mod support;

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use api_server_rust::{
    create_routes,
    error::{ApiError, ApiResult},
    features::auth::repository,
    http::{permissions::GlobalRole, token::create_access_token},
    state::{ApiConfig, ApiState},
    utils::{hasher, smtp_service::EmailService},
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

const PASSWORD: &str = "initial-password";
const SECRET: &str = "auth-tests-only-signing-key";

#[derive(Default)]
struct MockEmail {
    messages: Mutex<Vec<(String, String)>>,
    fail: bool,
}

impl EmailService for MockEmail {
    fn send_password_reset_email<'a>(
        &'a self,
        email: &'a str,
        token: &'a str,
        _config: &'a ApiConfig,
    ) -> Pin<Box<dyn Future<Output = ApiResult<()>> + Send + 'a>> {
        Box::pin(async move {
            if self.fail {
                return Err(ApiError::ServiceUnavailable("mock email failure".into()));
            }
            self.messages
                .lock()
                .unwrap()
                .push((email.to_owned(), token.to_owned()));
            Ok(())
        })
    }
}

async fn app(pool: PgPool, mail: Arc<MockEmail>) -> Router {
    for user_id in [101, 102, 103] {
        repository::update_user_password(&pool, user_id, PASSWORD)
            .await
            .unwrap();
    }
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
    let state = ApiState::new(config, DecodingKey::from_secret(SECRET.as_bytes()), pool)
        .await
        .with_email_service(mail);
    create_routes().with_state(state)
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    payload: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(format!("/api/v3/auth/{path}"))
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(payload.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn login(app: &Router, email: &str, password: &str) -> (StatusCode, Value) {
    request(
        app,
        "POST",
        "login",
        json!({"login": email, "password": password}),
        None,
    )
    .await
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn register_creates_user_and_usable_access_token(pool: PgPool) {
    // Arrange
    let app = app(pool.clone(), Arc::default()).await;
    let payload =
        json!({"email": "new@example.com", "password": PASSWORD, "full_name": "New User"});

    // Act
    let (status, body) = request(&app, "POST", "register", payload.clone(), None).await;

    // Assert
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["user"]["email"], "new@example.com");
    assert_eq!(body["user"]["full_name"], "New User");
    assert_eq!(body["user"]["global_role"], "user");
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);
    let token = body["access_token"].as_str().unwrap();
    assert_eq!(
        request(&app, "GET", "me", Value::Null, Some(token)).await.0,
        StatusCode::OK
    );
    let stored = repository::find_creds_by_email(&pool, "new@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_ne!(stored.hashed_password, PASSWORD);
    assert!(hasher::verify_password(PASSWORD, &stored.hashed_password).unwrap());
    assert_eq!(
        request(&app, "POST", "register", payload, None).await.0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        login(&app, "new@example.com", PASSWORD).await.0,
        StatusCode::OK
    );
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn register_validates_input_and_accepts_optional_fields(pool: PgPool) {
    // Arrange
    let app = app(pool, Arc::default()).await;

    // Act
    for payload in [
        json!({"email": "invalid", "password": PASSWORD}),
        json!({"email": "new@example.com", "password": "short"}),
        json!({"email": "new@example.com", "password": PASSWORD, "phone": "invalid"}),
        json!({"email": "new@example.com", "password": PASSWORD, "full_name": "a".repeat(256)}),
    ] {
        // Assert
        assert_eq!(
            request(&app, "POST", "register", payload, None).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    let (status, body) = request(
        &app,
        "POST",
        "register",
        json!({"email": "new@example.com", "password": PASSWORD}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn login_checks_password_account_status_and_role(pool: PgPool) {
    // Arrange
    let app = app(pool, Arc::default()).await;

    // Act
    for email in ["user@example.com", "admin@example.com"] {
        let (status, body) = login(&app, email, PASSWORD).await;

        // Assert
        assert_eq!(status, StatusCode::OK, "{body}");
        let role = if email.starts_with("admin") {
            "admin"
        } else {
            "user"
        };
        assert_eq!(body["user"]["global_role"], role);
        assert!(body["access_token"].is_string());
    }
    for (email, password) in [
        ("user@example.com", "wrong"),
        ("missing@example.com", PASSWORD),
        ("disabled@example.com", PASSWORD),
    ] {
        assert_eq!(
            login(&app, email, password).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn me_returns_memberships_and_requires_user_token(pool: PgPool) {
    // Arrange
    let app = app(pool, Arc::default()).await;

    // Act
    for (id, role) in [(101, GlobalRole::User), (102, GlobalRole::Admin)] {
        let token = create_access_token(id, 3600, role, SECRET);
        let (status, body) = request(&app, "GET", "me", Value::Null, Some(&token)).await;

        // Assert
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["user_id"], id);
        assert!(!body["permissions"].as_array().unwrap().is_empty());
        if id == 101 {
            assert_eq!(body["partner_memberships"][0]["partner_id"], 101);
            assert_eq!(
                body["partner_memberships"][0]["permissions"],
                json!(["cameras:read"])
            );
            assert_eq!(body["partner_memberships"][0]["read_scope"], "partner_all");
        } else {
            assert_eq!(body["partner_memberships"], json!([]));
        }
    }
    for token in [None, Some("broken"), Some("test-api-token")] {
        assert_eq!(
            request(&app, "GET", "me", Value::Null, token).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    let token = create_access_token(999, 3600, GlobalRole::User, SECRET);
    assert_eq!(
        request(&app, "GET", "me", Value::Null, Some(&token))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn password_reset_request_hashes_token_and_hides_unknown_email(pool: PgPool) {
    // Arrange
    let mail = Arc::new(MockEmail::default());
    let app = app(pool.clone(), mail.clone()).await;

    // Act
    for email in ["user@example.com", "missing@example.com"] {
        let (status, body) = request(
            &app,
            "POST",
            "password-reset/request",
            json!({"email": email}),
            None,
        )
        .await;

        // Assert
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
        assert_eq!(body, Value::Null);
    }
    let messages = mail.messages.lock().unwrap().clone();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].0, "user@example.com");
    let stored = repository::find_password_reset_token_by_email(&pool, &messages[0].0)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.token_hash,
        hasher::hash_password_reset_token(&messages[0].1)
    );
    assert!(stored.used_at.is_none());
    assert!(stored.expires_at > chrono::Utc::now());
    assert!(stored.expires_at <= chrono::Utc::now() + chrono::Duration::minutes(15));
    assert!(
        repository::find_password_reset_token_by_email(&pool, "missing@example.com")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "password-reset/request",
            json!({"email": "invalid"}),
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn password_reset_reports_email_delivery_failure(pool: PgPool) {
    // Arrange
    let mail = Arc::new(MockEmail {
        fail: true,
        ..Default::default()
    });
    let app = app(pool, mail).await;

    // Act

    // Assert
    assert_eq!(
        request(
            &app,
            "POST",
            "password-reset/request",
            json!({"email": "user@example.com"}),
            None
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn password_reset_confirm_changes_password_once_and_allows_new_request(pool: PgPool) {
    // Arrange
    let mail = Arc::new(MockEmail::default());
    let app = app(pool.clone(), mail.clone()).await;

    // Act
    for _ in 0..2 {
        // Assert
        assert_eq!(
            request(
                &app,
                "POST",
                "password-reset/request",
                json!({"email": "user@example.com"}),
                None
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
        let token = mail.messages.lock().unwrap().last().unwrap().1.clone();
        let payload = json!({"email": "user@example.com", "token": token, "new_password": "changed-password"});
        let (status, body) = request(
            &app,
            "POST",
            "password-reset/confirm",
            payload.clone(),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
        assert_eq!(
            request(&app, "POST", "password-reset/confirm", payload, None)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert!(
            repository::find_password_reset_token_by_email(&pool, "user@example.com")
                .await
                .unwrap()
                .unwrap()
                .used_at
                .is_some()
        );
    }
    assert_eq!(
        login(&app, "user@example.com", PASSWORD).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "user@example.com", "changed-password").await.0,
        StatusCode::OK
    );
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn password_reset_rejects_invalid_expired_and_replaced_tokens(pool: PgPool) {
    // Arrange
    let app = app(pool.clone(), Arc::default()).await;
    let future = chrono::Utc::now().fixed_offset() + chrono::Duration::minutes(15);
    repository::create_password_reset_token(&pool, 101, "old-token", future)
        .await
        .unwrap();
    repository::create_password_reset_token(&pool, 101, "new-token", future)
        .await
        .unwrap();

    // Act
    for token in ["old-token", "wrong-token"] {
        // Assert
        assert_eq!(request(&app, "POST", "password-reset/confirm", json!({"email": "user@example.com", "token": token, "new_password": "changed-password"}), None).await.0, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(
        request(
            &app,
            "POST",
            "password-reset/confirm",
            json!({"email": "user@example.com", "token": "new-token", "new_password": "short"}),
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert!(
        repository::find_password_reset_token_by_email(&pool, "user@example.com")
            .await
            .unwrap()
            .unwrap()
            .used_at
            .is_none()
    );
    repository::create_password_reset_token(
        &pool,
        101,
        "expired-token",
        chrono::Utc::now().fixed_offset() - chrono::Duration::seconds(1),
    )
    .await
    .unwrap();
    assert_eq!(request(&app, "POST", "password-reset/confirm", json!({"email": "user@example.com", "token": "expired-token", "new_password": "changed-password"}), None).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        login(&app, "user@example.com", PASSWORD).await.0,
        StatusCode::OK
    );
    assert_eq!(request(&app, "POST", "password-reset/confirm", json!({"email": "missing@example.com", "token": "unused", "new_password": "changed-password"}), None).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn concurrent_password_reset_only_succeeds_once(pool: PgPool) {
    // Arrange
    let app = app(pool.clone(), Arc::default()).await;
    repository::create_password_reset_token(
        &pool,
        101,
        "shared-token",
        chrono::Utc::now().fixed_offset() + chrono::Duration::minutes(15),
    )
    .await
    .unwrap();
    let payload = json!({"email": "user@example.com", "token": "shared-token", "new_password": "changed-password"});

    // Act
    let (first, second) = tokio::join!(
        request(
            &app,
            "POST",
            "password-reset/confirm",
            payload.clone(),
            None
        ),
        request(&app, "POST", "password-reset/confirm", payload, None),
    );
    let mut statuses = [first.0, second.0];
    statuses.sort();

    // Assert
    assert_eq!(statuses, [StatusCode::NO_CONTENT, StatusCode::UNAUTHORIZED]);
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn logout_revokes_only_current_token(pool: PgPool) {
    // Arrange
    let app = app(pool.clone(), Arc::default()).await;
    let (_, first) = login(&app, "user@example.com", PASSWORD).await;
    let (_, second) = login(&app, "user@example.com", PASSWORD).await;
    let token = first["access_token"].as_str().unwrap();
    let other_token = second["access_token"].as_str().unwrap();

    // Assert
    assert_ne!(token, other_token);

    // Act
    assert_eq!(
        request(&app, "POST", "logout", Value::Null, Some(token))
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert!(repository::is_token_revoked(&pool, token).await.unwrap());
    assert_eq!(
        request(&app, "GET", "me", Value::Null, Some(token)).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, "POST", "logout", Value::Null, Some(token))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, "GET", "me", Value::Null, Some(other_token))
            .await
            .0,
        StatusCode::OK
    );
    for token in [None, Some("broken"), Some("test-api-token")] {
        assert_eq!(
            request(&app, "POST", "logout", Value::Null, token).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn protected_handlers_reject_expired_and_forged_tokens(pool: PgPool) {
    // Arrange
    let app = app(pool, Arc::default()).await;
    let expired = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &json!({"sub": 101, "role": "user", "exp": chrono::Utc::now().timestamp() - 60}),
        &jsonwebtoken::EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap();
    let forged = create_access_token(101, 3600, GlobalRole::User, "incorrect-signing-key");

    // Act
    for token in [expired, forged] {
        for (method, path) in [("GET", "me"), ("POST", "logout")] {
            // Assert
            assert_eq!(
                request(&app, method, path, Value::Null, Some(&token))
                    .await
                    .0,
                StatusCode::UNAUTHORIZED
            );
        }
    }
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn logout_supports_legacy_tokens_and_revokes_access_to_other_features(pool: PgPool) {
    // Arrange
    let app = app(pool, Arc::default()).await;
    // Tokens issued before jti was introduced must remain valid and revocable.
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &json!({"sub": 102, "role": "admin", "exp": chrono::Utc::now().timestamp() + 3600}),
        &jsonwebtoken::EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap();
    let camera_request = || {
        Request::builder()
            .uri("/api/v3/cameras")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    };

    // Act

    // Assert
    assert_eq!(
        app.clone()
            .oneshot(camera_request())
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        request(&app, "POST", "logout", Value::Null, Some(&token))
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        app.clone()
            .oneshot(camera_request())
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(
    migrator = "crate::support::MIGRATOR",
    fixtures(path = "../fixtures", scripts("auth"))
)]
async fn password_reset_follows_account_when_email_changes(pool: PgPool) {
    use api_server_rust::features::users::models::Users;
    use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, Query};
    use sea_query_sqlx::SqlxBinder;

    // Arrange
    let app = app(pool.clone(), Arc::default()).await;
    repository::create_password_reset_token(
        &pool,
        101,
        "account-token",
        chrono::Utc::now().fixed_offset() + chrono::Duration::minutes(15),
    )
    .await
    .unwrap();

    // Act
    for (user_id, email) in [(101, "changed@example.com"), (102, "user@example.com")] {
        let (sql, values) = Query::update()
            .table(Users::Table)
            .value(Users::Email, email)
            .and_where(Expr::col(Users::UserId).eq(user_id))
            .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
            .execute(&pool)
            .await
            .unwrap();
    }
    let (wrong_status, _) = request(&app, "POST", "password-reset/confirm",
        json!({"email": "user@example.com", "token": "account-token", "new_password": "changed-password"}), None).await;
    let (status, body) = request(&app, "POST", "password-reset/confirm",
        json!({"email": "changed@example.com", "token": "account-token", "new_password": "changed-password"}), None).await;

    // Assert
    assert_eq!(wrong_status, StatusCode::NOT_FOUND);
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(
        login(&app, "changed@example.com", "changed-password")
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        login(&app, "user@example.com", PASSWORD).await.0,
        StatusCode::OK
    );
    assert_eq!(
        login(&app, "user@example.com", "changed-password").await.0,
        StatusCode::UNAUTHORIZED
    );
}
