use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::{RngExt, distr::Alphanumeric};
use sha2::{Digest, Sha256};

use crate::error::{ApiError, ApiResult};

pub fn hash_password(plain: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| ApiError::from(anyhow::anyhow!("hashing password: {e}")))
}

pub fn verify_password(provided: &str, stored: &str) -> ApiResult<bool> {
    let parsed_hash = PasswordHash::new(stored)
        .map_err(|e| ApiError::from(anyhow::anyhow!("parsing password hash: {e}")))?;

    Ok(Argon2::default()
        .verify_password(provided.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_password_reset_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

pub fn hash_password_reset_token(token: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(token.as_bytes());

    hex::encode(hasher.finalize())
}

pub fn verify_password_reset_token(provided: &str, stored: &str) -> bool {
    constant_time_eq::constant_time_eq(
        hash_password_reset_token(provided).as_bytes(),
        stored.as_bytes(),
    )
}
