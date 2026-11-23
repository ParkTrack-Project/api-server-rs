use std::sync::LazyLock;

use sqlx::migrate::Migrator;

mod safety;
use safety::validate_urls;

/// SQLx evaluates this argument before `TestFn::run_test` creates its master pool.
/// A guard inside the async test body would run after database creation/migrations.
pub static MIGRATOR: LazyLock<Migrator> = LazyLock::new(|| {
    let test_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        panic!("test database safety: TEST_DATABASE_URL is required; use scripts/test.sh")
    });
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| panic!("test database safety: explicit DATABASE_URL is required; .env fallback is forbidden; use scripts/test.sh"));
    validate_urls(&test_url, &database_url)
        .unwrap_or_else(|reason| panic!("test database safety: {reason}"));
    sqlx::migrate!()
});
