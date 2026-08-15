use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
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

const JWT_ALGORITHM: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS256;
const JWT_LEEWAY_SECONDS: u64 = 20;

pub fn create_access_token(
    user_id: u32,
    expire_seconds: u64,
    role: GlobalRole,
    secret: &str
) -> String {
    let now = OffsetDateTime::now_utc();

    let expire = now + Duration::from_secs(expire_seconds);

    let claims = Claims {
        sub: user_id,
        exp: expire.unix_timestamp() as usize,
        role
    };

    let header = Header::new(JWT_ALGORITHM);
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    encode(&header, &claims, &encoding_key).expect("Failed to encode JWT")
}

pub fn decode_token(token: &str, decoding_key: &DecodingKey) -> ApiResult<TokenData<Claims>> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.leeway = JWT_LEEWAY_SECONDS;

    let decoded =
        decode::<Claims>(&token, &decoding_key, &validation).map_err(|err| match err
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

#[cfg(test)]
mod tests {
    use std::{assert_matches, thread};

    use jsonwebtoken::DecodingKey;

    use crate::{error::ApiError, http::{permissions::GlobalRole, token::JWT_LEEWAY_SECONDS}};

    use super::{decode_token, create_access_token};

    #[test]
    fn encode_token_successfully() {
        // Arrange
        let secret = "he was boiling shit in a kettle";

        // Act
        let _ = create_access_token(1, 5, GlobalRole::User, secret);

        // Assert
    }

    #[test]
    fn decode_token_malformed_returns_api_error() {
        // Arrange
        let token = "shit";
        let decoding_key = DecodingKey::from_secret(&[1,2,3,4]);

        // Act
        let res = decode_token(token, &decoding_key);

        // Assert
        assert_matches!(res, Err(ApiError::InvalidToken));
    }

    #[test]
    #[ignore]
    fn decode_token_expired_returns_api_error() {
        // Arrange
        let secret = "he was boiling shit in a kettle";
        let exp = 3;
        let token = create_access_token(1, exp, GlobalRole::User, &secret);
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        // Act
        thread::sleep(std::time::Duration::from_secs(JWT_LEEWAY_SECONDS + exp + 1));
        let res = decode_token(&token, &decoding_key);

        // Assert
        assert_matches!(res, Err(ApiError::TokenExpired));
    }
}
