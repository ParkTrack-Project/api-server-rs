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
use sqlx::PgPool;
use tower::ServiceExt;
const SECRET: &str = "partners-test-key";
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
        .uri(format!("/api/v3/partners{path}"))
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
fn payload() -> Value {
    json!({"name":"New Partner","slug":"new-partner","contact_email":"new@example.com","contact_phone":"+79990000003"})
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn partner_crud_and_soft_delete(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let (s, created) = request(&app, "POST", "", payload(), Some(GlobalRole::Admin)).await;
    let path = format!("/{}", created["partner_id"]);
    let (us, updated) = request(
        &app,
        "PUT",
        &path,
        json!({"name":"Renamed","slug":"renamed"}),
        Some(GlobalRole::Admin),
    )
    .await;
    let (ds, _) = request(&app, "DELETE", &path, Value::Null, Some(GlobalRole::Admin)).await;
    let (gs, disabled) = request(&app, "GET", &path, Value::Null, Some(GlobalRole::Admin)).await;
    //Assert
    assert_eq!(s, StatusCode::CREATED, "{created}");
    assert_eq!(us, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Renamed");
    assert_eq!(updated["slug"], "renamed");
    assert_eq!(ds, StatusCode::NO_CONTENT);
    assert_eq!(gs, StatusCode::OK);
    assert_eq!(disabled["is_active"], false);
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn organization_visibility_and_pagination(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let (_, member) = request(&app, "GET", "", Value::Null, Some(GlobalRole::User)).await;
    let (_, admin) = request(
        &app,
        "GET",
        "?top=1&offset=1",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    //Assert
    assert_eq!(member["total"], 1);
    assert_eq!(member["items"][0]["partner_id"], 101);
    assert_eq!(admin["total"], 2);
    assert_eq!(admin["items"].as_array().unwrap().len(), 1);
    assert_eq!(admin["items"][0]["partner_id"], 102);
    assert_eq!(
        request(&app, "GET", "/102", Value::Null, Some(GlobalRole::User))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "?q=second",
            Value::Null,
            Some(GlobalRole::Admin)
        )
        .await
        .1["total"],
        1
    );
    for (method, path) in [("POST", ""), ("PUT", "/101"), ("DELETE", "/101")] {
        assert_eq!(
            request(&app, method, path, payload(), Some(GlobalRole::User))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
    }
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn organization_validation_and_conflicts(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let mut duplicate = payload();
    duplicate["slug"] = json!("first-partner");
    let (status, _) = request(&app, "POST", "", duplicate, Some(GlobalRole::Admin)).await;
    //Assert
    assert_eq!(status, StatusCode::CONFLICT);
    for (key, v) in [
        ("name", json!("")),
        ("slug", json!("Upper Case")),
        ("contact_email", json!("invalid")),
        ("contact_phone", json!("bad")),
    ] {
        let mut p = payload();
        p[key] = v;
        assert!(
            !request(&app, "POST", "", p, Some(GlobalRole::Admin))
                .await
                .0
                .is_success()
        );
    }
    assert_eq!(
        request(&app, "PUT", "/101", json!({}), Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    for path in ["?top=0", "?top=101", "?offset=-1", "/0"] {
        assert!(
            !request(&app, "GET", path, Value::Null, Some(GlobalRole::Admin))
                .await
                .0
                .is_success()
        );
    }
    assert_eq!(
        request(&app, "GET", "/999", Value::Null, Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    for (method, path) in [
        ("GET", ""),
        ("POST", ""),
        ("GET", "/101"),
        ("PUT", "/101"),
        ("DELETE", "/101"),
    ] {
        assert_eq!(
            request(&app, method, path, payload(), None).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
}

fn member_payload() -> Value {
    json!({"user_id":103,"role":"operator","permissions":["zones.view","cameras.view"],"read_scope":"partner_all","write_scope":"none","delete_scope":"own"})
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn member_crud_and_soft_disable(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let (created, member) = request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::User),
    )
    .await;
    let (read, fetched) = request(
        &app,
        "GET",
        "/101/members/103",
        Value::Null,
        Some(GlobalRole::User),
    )
    .await;
    let (updated, changed) = request(
        &app,
        "PUT",
        "/101/members/103",
        json!({"role":"viewer","permissions":["sources.view"],"read_scope":"own"}),
        Some(GlobalRole::User),
    )
    .await;
    let (disabled, _) = request(
        &app,
        "DELETE",
        "/101/members/103",
        Value::Null,
        Some(GlobalRole::User),
    )
    .await;
    let (_, inactive) = request(
        &app,
        "GET",
        "/101/members/103",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    //Assert
    assert_eq!(created, StatusCode::CREATED, "{member}");
    assert_eq!(member["partner_id"], 101);
    assert_eq!(member["is_active"], true);
    assert_eq!(member["permissions"], json!(["cameras.view", "zones.view"]));
    assert_eq!(read, StatusCode::OK);
    assert_eq!(member, fetched);
    assert_eq!(updated, StatusCode::OK, "{changed}");
    assert_eq!(changed["permissions"], json!(["sources.view"]));
    assert_eq!(changed["role"], "viewer");
    assert_eq!(changed["read_scope"], "own");
    assert_eq!(disabled, StatusCode::NO_CONTENT);
    assert_eq!(inactive["is_active"], false);
    assert_eq!(inactive["permissions"], json!(["sources.view"]));
    let (s, restored) = request(
        &app,
        "PUT",
        "/101/members/103",
        json!({"is_active":true,"permissions":[]}),
        Some(GlobalRole::Admin),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(restored["permissions"], json!([]));
    assert_eq!(restored["is_active"], true);
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn members_filter_and_pagination(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::Admin),
    )
    .await;
    //Act
    let (status, page) = request(
        &app,
        "GET",
        "/101/members?top=1&offset=1",
        Value::Null,
        Some(GlobalRole::User),
    )
    .await;
    let (_, filtered) = request(
        &app,
        "GET",
        "/101/members?q=other&role=operator&is_active=true",
        Value::Null,
        Some(GlobalRole::User),
    )
    .await;
    //Assert
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(page["total"], 2);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["user_id"], 103);
    assert_eq!(filtered["total"], 1);
    assert_eq!(
        filtered["items"][0]["permissions"],
        json!(["cameras.view", "zones.view"])
    );
    let (_, empty) = request(
        &app,
        "GET",
        "/101/members?is_active=false",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    assert_eq!(empty["total"], 0);
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn membership_validation_conflicts_and_missing_rows(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let first = request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::Admin),
    )
    .await;
    let duplicate = request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::Admin),
    )
    .await;
    //Assert
    assert_eq!(first.0, StatusCode::CREATED);
    assert_eq!(duplicate.0, StatusCode::CONFLICT);
    for (field, value) in [
        ("role", json!("")),
        ("permissions", json!(["admin.partners.manage"])),
        ("permissions", json!(["zones.create"])),
        ("permissions", json!(["zones.view", "zones.view"])),
        ("read_scope", json!("bad")),
        ("user_id", json!(0)),
    ] {
        let mut body = member_payload();
        body[field] = value;
        assert_eq!(
            request(&app, "POST", "/101/members", body, Some(GlobalRole::Admin))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &app,
            "PUT",
            "/101/members/103",
            json!({}),
            Some(GlobalRole::Admin)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let mut body = member_payload();
    body["user_id"] = json!(999);
    assert_eq!(
        request(&app, "POST", "/101/members", body, Some(GlobalRole::Admin))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    for (method, path) in [
        ("GET", "/999/members"),
        ("GET", "/101/members/999"),
        ("PUT", "/101/members/999"),
        ("DELETE", "/101/members/999"),
    ] {
        assert_eq!(
            request(
                &app,
                method,
                path,
                json!({"role":"viewer"}),
                Some(GlobalRole::Admin)
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn membership_access_is_scoped_and_revoked_on_disable(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let hidden = request(
        &app,
        "GET",
        "/102/members",
        Value::Null,
        Some(GlobalRole::User),
    )
    .await;
    let disabled = request(
        &app,
        "DELETE",
        "/101/members/101",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    let (_, visible) = request(&app, "GET", "", Value::Null, Some(GlobalRole::User)).await;
    //Assert
    assert_eq!(hidden.0, StatusCode::FORBIDDEN);
    assert_eq!(disabled.0, StatusCode::NO_CONTENT);
    assert_eq!(visible["total"], 0);
    for (method, path) in [
        ("GET", "/101/members"),
        ("GET", "/101/members/103"),
        ("POST", "/101/members"),
        ("PUT", "/101/members/103"),
        ("DELETE", "/101/members/103"),
    ] {
        assert_eq!(
            request(&app, method, path, member_payload(), Some(GlobalRole::User))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            request(&app, method, path, member_payload(), None).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn member_permissions_are_checked_for_each_action(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    request(
        &app,
        "PUT",
        "/101/members/101",
        json!({"permissions":["partner_members.view"]}),
        Some(GlobalRole::Admin),
    )
    .await;
    //Act
    let read = request(
        &app,
        "GET",
        "/101/members",
        Value::Null,
        Some(GlobalRole::User),
    )
    .await;
    //Assert
    assert_eq!(read.0, StatusCode::OK);
    for (method, path) in [
        ("POST", "/101/members"),
        ("PUT", "/101/members/101"),
        ("DELETE", "/101/members/101"),
    ] {
        assert_eq!(
            request(&app, method, path, member_payload(), Some(GlobalRole::User))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
    }
    request(&app, "DELETE", "/101", Value::Null, Some(GlobalRole::Admin)).await;
    assert_eq!(
        request(
            &app,
            "GET",
            "/101/members",
            Value::Null,
            Some(GlobalRole::User)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/101/members",
            Value::Null,
            Some(GlobalRole::Admin)
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[sqlx::test(fixtures(
    path = "../fixtures",
    scripts("partners", "partner_permission_failure")
))]
async fn membership_and_permission_changes_roll_back_together(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    //Act
    let failed_create = request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::Admin),
    )
    .await;
    let missing = request(
        &app,
        "GET",
        "/101/members/103",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    let mut valid = member_payload();
    valid["permissions"] = json!(["sources.view"]);
    let created = request(&app, "POST", "/101/members", valid, Some(GlobalRole::Admin)).await;
    let failed_update = request(
        &app,
        "PUT",
        "/101/members/103",
        json!({"role":"changed","permissions":["zones.view"]}),
        Some(GlobalRole::Admin),
    )
    .await;
    let (_, unchanged) = request(
        &app,
        "GET",
        "/101/members/103",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    //Assert
    assert_eq!(failed_create.0, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(missing.0, StatusCode::NOT_FOUND);
    assert_eq!(created.0, StatusCode::CREATED);
    assert_eq!(failed_update.0, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(unchanged["role"], "operator");
    assert_eq!(unchanged["permissions"], json!(["sources.view"]));
}
#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn concurrent_permission_replacements_never_merge(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::Admin),
    )
    .await;
    //Act
    let (first, second) = tokio::join!(
        request(
            &app,
            "PUT",
            "/101/members/103",
            json!({"permissions":["sources.view"]}),
            Some(GlobalRole::Admin)
        ),
        request(
            &app,
            "PUT",
            "/101/members/103",
            json!({"permissions":["zones.view","zones.write"]}),
            Some(GlobalRole::Admin)
        ),
    );
    let (_, member) = request(
        &app,
        "GET",
        "/101/members/103",
        Value::Null,
        Some(GlobalRole::Admin),
    )
    .await;
    //Assert
    assert_eq!(first.0, StatusCode::OK, "{}", first.1);
    assert_eq!(second.0, StatusCode::OK, "{}", second.1);
    assert!(
        member["permissions"] == json!(["sources.view"])
            || member["permissions"] == json!(["zones.view", "zones.write"])
    );
}

#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn membership_scopes_accept_only_supported_values(pool: PgPool) {
    //Arrange
    let app = app(pool).await;
    let created = request(
        &app,
        "POST",
        "/101/members",
        member_payload(),
        Some(GlobalRole::Admin),
    )
    .await;
    assert_eq!(created.0, StatusCode::CREATED);
    for field in ["read_scope", "write_scope", "delete_scope"] {
        for scope in ["own", "partner_all", "none"] {
            //Act
            let (status, member) = request(
                &app,
                "PUT",
                "/101/members/103",
                json!({field: scope}),
                Some(GlobalRole::Admin),
            )
            .await;
            //Assert
            assert_eq!(status, StatusCode::OK, "{member}");
            assert_eq!(member[field], scope);
        }
        for scope in [
            json!("assigned"),
            json!("own_or_assigned"),
            json!("owned"),
            json!("all"),
            Value::Null,
        ] {
            //Act
            let (status, _) = request(
                &app,
                "PUT",
                "/101/members/103",
                json!({field: scope}),
                Some(GlobalRole::Admin),
            )
            .await;
            //Assert
            assert!(!status.is_success());
            let mut input = member_payload();
            input[field] = scope;
            assert!(
                !request(&app, "POST", "/101/members", input, Some(GlobalRole::Admin))
                    .await
                    .0
                    .is_success()
            );
        }
    }
    //Act
    let (status, member) = request(
        &app,
        "PUT",
        "/101/members/103",
        json!({"role":"viewer"}),
        Some(GlobalRole::Admin),
    )
    .await;
    //Assert
    assert_eq!(status, StatusCode::OK);
    for field in ["read_scope", "write_scope", "delete_scope"] {
        assert_eq!(member[field], "none");
    }
}

#[sqlx::test(fixtures(path = "../fixtures", scripts("partners")))]
async fn scope_migration_normalizes_legacy_rows_and_enforces_constraints(pool: PgPool) {
    use api_server_rust::partners::{
        member_repository::{self, MemberAccess},
        models::PartnerMemberships as M,
    };
    use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, Query};
    use sea_query_sqlx::SqlxBinder;
    //Arrange
    let migrator = sqlx::migrate!();
    migrator.undo(&pool, 21).await.unwrap();
    let (sql, values) = Query::update()
        .table(M::Table)
        .value(M::ReadScope, Option::<String>::None)
        .value(M::WriteScope, "own_or_assigned")
        .value(M::DeleteScope, "assigned")
        .and_where(Expr::col(M::UserId).eq(101))
        .build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await
        .unwrap();
    //Act
    migrator.run(&pool).await.unwrap();
    let row = member_repository::find(&pool, MemberAccess::Partner { partner_id: 101 }, 101)
        .await
        .unwrap()
        .unwrap();
    //Assert
    assert_eq!(row.read_scope, "none");
    assert_eq!(row.write_scope, "own");
    assert_eq!(row.delete_scope, "none");
    for col in [M::ReadScope, M::WriteScope, M::DeleteScope] {
        //Act
        let (sql, values) = Query::update()
            .table(M::Table)
            .value(col, Option::<String>::None)
            .build_sqlx(PostgresQueryBuilder);
        let result = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
            .execute(&pool)
            .await;
        //Assert
        let error = result.unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("23502")
        );
    }
    for col in [M::ReadScope, M::WriteScope, M::DeleteScope] {
        //Act
        let (sql, values) = Query::update()
            .table(M::Table)
            .value(col, "assigned")
            .build_sqlx(PostgresQueryBuilder);
        let result = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
            .execute(&pool)
            .await;
        //Assert
        let error = result.unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("23514")
        );
    }
}
