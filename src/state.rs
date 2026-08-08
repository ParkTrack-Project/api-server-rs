use std::sync::Arc;

use jsonwebtoken::DecodingKey;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct ApiState {
    pub inner: Arc<ApiStateInner>,
}

pub struct ApiStateInner {
    pub config: ApiConfig,
    pub decoding_key: DecodingKey,
    pub pool: PgPool,
}

impl std::ops::Deref for ApiState {
    type Target = ApiStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ApiState {
    pub async fn default() -> Self {
        let config = ApiConfig::from_env();
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await
            .expect("Database connection should be set up");

        let inner = ApiStateInner {
            config,
            decoding_key,
            pool,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub async fn new(config: ApiConfig, decoding_key: DecodingKey, pool: PgPool) -> Self {
        Self {
            inner: Arc::new(ApiStateInner {
                config,
                decoding_key,
                pool,
            }),
        }
    }
}

#[derive(Clone)]
pub struct ApiConfig {
    pub api_token: String,

    pub database_url: String,

    pub jwt_secret: String,
    pub jwt_expire_seconds: u32,

    pub smtp_host: String,
    pub smtp_port: u32,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_use_tls: bool,
    pub smtp_use_ssl: bool,

    pub password_reset_return_token: bool,
    pub password_reset_ttl_minutes: u32,
    pub password_reset_login_url: String,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().expect("Failed to load .env file");

        use std::env;

        let api_token = env::var("API_TOKEN").expect("API_TOKEN must be set");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let jwt_expire_seconds: u32 = env::var("JWT_EXPIRE_SECONDS")
            .expect("JWT_EXPIRE_SECONDS must be set")
            .parse()
            .expect("JWT_EXPIRE_SECONDS must be positive integer");

        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_port: u32 = env::var("SMTP_PORT")
            .expect("SMTP_PORT must be set")
            .parse()
            .expect("SMTP_PORT must be a legal port");
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        let smtp_from_email = env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL must be set");
        let smtp_from_name = env::var("SMTP_FROM_NAME").expect("SMTP_FROM_NAME must be set");
        let smtp_use_tls: bool = env::var("SMTP_USE_TLS")
            .unwrap_or("1".to_string())
            .parse()
            .expect("SMTP_USE_TLS must be a boolean");
        let smtp_use_ssl: bool = env::var("SMTP_USE_SSL")
            .unwrap_or("0".to_string())
            .parse()
            .expect("SMTP_USE_SSL must be a boolean");
        let password_reset_login_url = env::var("PASSWORD_RESET_LOGIN_URL").unwrap_or_default();
        let password_reset_ttl_minutes: u32 = env::var("PASSWORD_TTL_RESET_MINUTES")
            .expect("PASSWORD_TTL_RESET_MINUTES must be set")
            .parse()
            .expect("PASSWORD_TTL_RESET_MINUTES must be positive integer");
        let password_reset_return_token: bool = env::var("PASSWORD_RESET_RETURN_TOKEN")
            .unwrap_or("0".to_string())
            .parse()
            .expect("SMTP_USE_SSL must be a boolean");

        Self {
            api_token,
            database_url,
            jwt_secret,
            jwt_expire_seconds,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from_email,
            smtp_from_name,
            smtp_use_tls,
            smtp_use_ssl,
            password_reset_return_token,
            password_reset_ttl_minutes,
            password_reset_login_url,
        }
    }
}
