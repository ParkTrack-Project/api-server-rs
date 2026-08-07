use std::sync::Arc;

use jsonwebtoken::DecodingKey;
use sqlx::PgPool;

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

#[derive(Clone)]
pub struct ApiConfig {
    pub api_token: String,

    pub database_url: String,

    pub jwt_secret: String,
    pub jwt_expire_seconds: u32,

    pub password_reset_return_token: String,
    pub password_reset_ttl_minutes: u32,
    pub password_reset_login_url: String,

    pub smtp_host: String,
    pub smtp_port: u32,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_use_tls: bool,
    pub smtp_use_ssl: bool,
}
