// Sign in, current user, and account settings.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::AssertSqlSafe;

use crate::auth::{create_token, hash_password, verify_password, AuthUser};
use crate::error::{require, AppError, AppResult};
use crate::handlers::users::{User, USER_COLUMNS};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

// Internal login data; password hashes are never returned to the client.
#[derive(sqlx::FromRow)]
struct UserWithHash {
    id: i64,
    role: String,
    password_hash: String,
}

// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> AppResult<Json<LoginResponse>> {
    // Normalize email consistently with user creation.
    let email = input.email.trim().to_lowercase();

    let found = sqlx::query_as::<_, UserWithHash>(
        "SELECT id, role, password_hash FROM users WHERE email = $1",
    )
        .bind(&email)
        .fetch_optional(&state.db)
        .await?;

    // Same message for "no such user" and "wrong password" so attackers can't tell which.
    let found = match found {
        Some(u) if verify_password(&input.password, &u.password_hash) => u,
        _ => return Err(AppError::BadRequest("Incorrect email or password".into())),
    };

    let token = create_token(found.id, &found.role, &state.jwt_secret)?;
    let user = find_user(&state, found.id).await?;
    Ok(Json(LoginResponse { token, user }))
}

// GET /api/auth/me
pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<User>> {
    Ok(Json(find_user(&state, user.id).await?))
}

#[derive(Deserialize)]
pub struct UpdateMeInput {
    pub name: String,
}

// PUT /api/auth/me
pub async fn update_me(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<UpdateMeInput>,
) -> AppResult<Json<User>> {
    require(&input.name, "Name")?;
    let updated = sqlx::query_as::<_, User>(AssertSqlSafe(format!(
        "UPDATE users SET name = $1 WHERE id = $2 RETURNING {USER_COLUMNS}"
    )))
        .bind(input.name.trim())
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(updated))
}

#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

// PUT /api/auth/password
pub async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<ChangePasswordInput>,
) -> AppResult<StatusCode> {
    if input.new_password.len() < 8 {
        return Err(AppError::BadRequest("New password must be at least 8 characters".into()));
    }

    let current_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
    if !verify_password(&input.current_password, &current_hash) {
        return Err(AppError::BadRequest("Current password is incorrect".into()));
    }

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(hash_password(&input.new_password)?)
        .bind(user.id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn find_user(state: &AppState, id: i64) -> AppResult<User> {
    sqlx::query_as::<_, User>(AssertSqlSafe(format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1")))
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        // Reject tokens for users who no longer exist.
        .ok_or(AppError::Unauthorized)
}


