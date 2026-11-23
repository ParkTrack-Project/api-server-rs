use sqlx::postgres::PgConnectOptions;

pub fn validate_urls(test_url: &str, database_url: &str) -> Result<(), &'static str> {
    if test_url != database_url {
        return Err("DATABASE_URL must match TEST_DATABASE_URL");
    }
    let options: PgConnectOptions = test_url.parse().map_err(|_| "invalid test URL")?;
    if !matches!(options.get_host(), "localhost" | "127.0.0.1" | "::1")
        || options.get_socket().is_some()
    {
        return Err("only a local TCP test PostgreSQL is allowed");
    }
    if options.get_username() != "sqlx_test"
        || options.get_database() != Some("api_server_rust_tests")
    {
        return Err("dedicated sqlx_test user and api_server_rust_tests database are required");
    }
    Ok(())
}
