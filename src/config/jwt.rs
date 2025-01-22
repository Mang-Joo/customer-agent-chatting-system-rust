use std::{fmt, sync::Arc};

use axum::extract::FromRequestParts;
use chrono::{Duration, Utc};
use http::header;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::{error::AppError, MangJooResult};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct JwtClaims {
    pub sub: i64,
    pub exp: i64,
    pub iat: i64,
    pub role: String,
}

impl JwtClaims {
    pub fn new(sub: i64, exp: i64, iat: i64, role: String) -> Self {
        Self {
            sub,
            exp,
            iat,
            role,
        }
    }
}

#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decondig_key: DecodingKey,
}

// Debug를 수동으로 구현
impl fmt::Debug for JwtManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtManager")
            // EncodingKey는 민감한 정보이므로 실제 값 대신 placeholder 출력
            .field("encoding_key", &"[REDACTED]")
            .field("decondig_key", &"[REDACTED]")
            // ... 다른 필드들
            .finish()
    }
}

impl JwtManager {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decondig_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn generate_token(&self, user_id: i64, role: &str) -> MangJooResult<String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let claims = JwtClaims {
            sub: user_id.to_owned(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            role: role.to_owned(),
        };

        let encoded_token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|err| AppError::InternalError(format!("{}", err)))?;

        Ok(encoded_token)
    }

    pub fn verify_token(&self, token: &str) -> MangJooResult<JwtClaims> {
        let validation = Validation::default();

        let token_data = decode::<JwtClaims>(token, &self.decondig_key, &validation)
            .map_err(|_| AppError::Unauthorized("Not valid token".to_string()))?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct JwtValidationExtractor(pub i64);

impl<S> FromRequestParts<S> for JwtValidationExtractor
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> MangJooResult<Self> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or(AppError::Unauthorized(format!("Token Valid Error")))?;

        let token = auth_header
            .strip_prefix("Token ")
            .ok_or(AppError::Unauthorized(String::from("Token invalid")))?;

        let jwt_manager = parts.extensions.get::<Arc<JwtManager>>().unwrap();

        let verify_token = jwt_manager.verify_token(token);
        let user_id = verify_token
            .map(|claims| claims.sub)
            .map_err(|_| AppError::Unauthorized(format!("Token Valid Error")))?;

        return Ok(JwtValidationExtractor(user_id));
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OptionalJwtValidationExtractor(pub Option<i64>);

impl<S> FromRequestParts<S> for OptionalJwtValidationExtractor
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> MangJooResult<Self> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let auth_header = match auth_header {
            Some(auth_header) => auth_header,
            None => return Ok(OptionalJwtValidationExtractor(None)),
        };

        let token = auth_header.strip_prefix("Bearer ");
        if let Some(token) = token {
            let jwt_manager = parts.extensions.get::<Arc<JwtManager>>().unwrap();

            let verify_token = jwt_manager.verify_token(token);
            let user_id = verify_token
                .map(|claims| claims.sub)
                .map_err(|_| AppError::Unauthorized(format!("Token Valid Error")))?;

            return Ok(OptionalJwtValidationExtractor(Some(user_id)));
        } else {
            return Err(AppError::Unauthorized(String::from("Token Error")));
        }
    }
}
