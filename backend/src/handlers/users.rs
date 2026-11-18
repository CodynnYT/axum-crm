// User management. Authenticated users can list users; only admins can add or remove them.

use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use sqlx::AssertSqlSafe;

use crate::auth::{hash_password, AuthUser};
use crate::error::{one_of, require, AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

pub const USER_COLUMNS: &str = "id, name, email, role, created_at";

#[derive(Deserialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

// GET /api/users
pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser
) -> AppResult<Json<Vec<User>>> {
    let users = sqlx::query_as::<_, User>(AssertSqlSafe(format!(
        "SELECT {} FROM users ORDER BY name",
        USER_COLUMNS
    )))
        .fetch_all(&state.db)
        .await?;
    Ok(Json(users))
}

// POST /api/users (admin only)
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<NewUser>
) -> AppResult<(StatusCode, Json<User>)> {
    user.require_admin()?;

    require(&input.name, "Name")?;
    require(&input.email, "Email")?;
    one_of(&input.role, &["admin", "member"], "Role",)?;

    if input.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters long".into()));
    }

    let created = sqlx::query_as::<_, User>(AssertSqlSafe(format!(
        "INSERT INTO users (name,email,password_hash,role)\
         VALUES ($1, $2, $3, $4)
         RETURNING {USER_COLUMNS}
        "
    )))
        .bind(input.name.trim())
        .bind(input.email.trim().to_lowercase())
        .bind(hash_password(&input.password)?)
        .bind(&input.role)
        .fetch_one(&state.db)
        .await?;

    Ok((StatusCode::CREATED, Json(created)))
}


// DELETE /api/users/{id} (admin only; self-deletion is prohibited)
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>
) -> AppResult<StatusCode> {
    user.require_admin()?;

    if id == user.id {
        return Err(AppError::BadRequest("You cannot delete yourself".into()));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

