#[path = "support/safety.rs"]
mod safety;

#[test]
fn rejects_production_and_implicit_or_ambiguous_targets() {
    //Arrange
    let safe = "postgres://sqlx_test:test-only@127.0.0.1:55433/api_server_rust_tests";
    let unsafe_urls = [
        "postgres://app:test-only@127.0.0.1:5432/production",
        "postgres://sqlx_test:test-only@production.example.com/api_server_rust_tests",
        "postgres://postgres:test-only@127.0.0.1/api_server_rust_tests",
        "postgres://sqlx_test:test-only@127.0.0.1/production",
        "not a URL",
        "postgres://sqlx_test:test-only@localhost/api_server_rust_tests?host=production.example.com",
        "postgres://sqlx_test:test-only@localhost/api_server_rust_tests?dbname=production",
        "postgres://sqlx_test:test-only@localhost/api_server_rust_tests?user=app",
        "postgres://sqlx_test:test-only@localhost/api_server_rust_tests?host=/var/run/postgresql",
    ];
    for url in unsafe_urls {
        //Act
        let result = safety::validate_urls(url, url);
        //Assert
        assert!(result.is_err());
    }
    //Act
    let mismatch = safety::validate_urls(safe, unsafe_urls[0]);
    //Assert
    assert!(mismatch.is_err());
}

#[test]
fn accepts_explicit_dedicated_local_test_database() {
    //Arrange
    let url = "postgres://sqlx_test:test-only@localhost:55433/api_server_rust_tests";
    //Act
    let result = safety::validate_urls(url, url);
    //Assert
    assert!(result.is_ok());
}
