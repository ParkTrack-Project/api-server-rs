use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};

use crate::error::{ApiError, ApiResult};

pub fn hash_password(plain: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| ApiError::from(anyhow::anyhow!("hashing password: {e}")))
}

pub fn verify_password(plain: &str, hashed: &str) -> ApiResult<bool> {
    let parsed_hash = PasswordHash::new(hashed)
        .map_err(|e| ApiError::from(anyhow::anyhow!("parsing password hash: {e}")))?;

    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed_hash)
        .is_ok())
}