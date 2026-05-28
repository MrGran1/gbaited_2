use crate::{config::Config, errors::AppError};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── JWT Claims ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// user id (subject)
    pub sub: String,
    /// issued-at  (unix seconds)
    pub iat: usize,
    /// expiry     (unix seconds)
    pub exp: usize,
}

pub fn create_token(user_id: &str, config: &Config) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + (config.jwt_expiry_hours as usize * 3_600),
    };

    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?)
}

pub fn verify_token(token: &str, config: &Config) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized("Token invalide ou expiré".into()))?;

    Ok(data.claims)
}

// ─── AuthUser extractor ───────────────────────────────────────────────────────

/// Injects the authenticated user's id into any handler that declares it.
/// Returns 401 if the `Authorization: Bearer <token>` header is missing or invalid.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    Config: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);

        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("Header Authorization manquant".into()))?;

        let claims = verify_token(token, &config)?;
        Ok(AuthUser { user_id: claims.sub })
    }
}
