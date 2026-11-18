// Authentication and password hashing.

use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

// Tokens expire after one day.
const TOKEN_LIFETIME_HOURS: i64 = 24;

// Hash a password for storage.
pub fn hash_password(password: &str) -> AppResult<String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    role: String,
    exp: usize,
}

// Create a signed token for a user.
pub fn create_token(user_id: i64, role: &str, secret: &str) -> AppResult<String> {
    let expires = Utc::now() + Duration::hours(TOKEN_LIFETIME_HOURS);
    let claims = Claims {
        sub: user_id,
        role: role.to_string(),
        exp: expires.timestamp() as usize,
    };

    let key = EncodingKey::from_secret(secret.as_bytes())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    encode(&Header::default(), &claims, &key)
        .map_err(|e| AppError::Internal(e.to_string()))

}

pub struct AuthUser {
    pub id: i64,
    pub role: String,
}

impl AuthUser {
    pub fn require_admin(&self) -> AppResult<()> {
        if self.role != "admin" {
            return Err(AppError::Forbidden);
        }
        Ok(())
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState)
    -> Result<Self, Self::Rejection> {

        let token = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let key = DecodingKey::from_secret(state.jwt_secret.as_bytes())
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let data = decode::<Claims>(&token, &key, &Validation::default())
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser{
            id: data.claims.sub,
            role: data.claims.role,
        })
    }

}

pub async fn create_first_admin(db: &PgPool, config: &Config) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await?;

    if count > 0 {
        return Ok(());
    }

    let hash = hash_password(&config.admin_password)?;
    sqlx::query("INSERT INTO USERS (name, email, password_hash, role)\
    VALUES ($1, $2, $3, 'admin')")
        .bind(&config.admin_name)
        .bind(&config.admin_email)
        .bind(&hash)
        .execute(db)
        .await?;

    tracing::info!("Created first admin user {}", config.admin_email);
    Ok(())
}
