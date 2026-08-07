use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    http::permissions::GlobalRole,
};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Claims {
    pub(crate) sub: u32,
    exp: usize,
    pub(crate) role: GlobalRole,
}

pub fn create_access_token(user_id: u32) -> ApiResult<String> {
    todo!()
}

pub fn decode_token(token: &str, decoding_key: &DecodingKey) -> ApiResult<TokenData<Claims>> {
    let decoded =
        decode::<Claims>(&token, &decoding_key, &Validation::default()).map_err(|err| match err
            .kind()
        {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ApiError::TokenExpired,
            _ => ApiError::InvalidToken,
        })?;

    decoded
        .claims
        .validate()
        .map_err(|_| ApiError::InvalidToken)?;

    Ok(decoded)
}
