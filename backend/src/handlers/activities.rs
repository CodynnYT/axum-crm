use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::AssertSqlSafe;

use crate::auth::AuthUser;
use crate::error::{one_of, require, AppError, AppResult};
use crate::handlers::SearchQuery;
use crate::state::AppState;

pub const TYPES: &[&str] = &["Call", "Meeting", "Email", "Task"];
pub const STATUSES: &[&str] = &["Pending", "Completed"];

#[derive(Serialize, sqlx::FromRow)]
pub struct Activity {
    pub id: i64,
    pub title: String,
    pub related_to: String,
    pub activity_type: String,
    pub due_at: Option<DateTime<Utc>>,
    pub status: String,
    pub notes: String,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub const SELECT_ACTIVITY: &str = "
    SELECT a.id, a.title, a.related_to, a.activity_type, a.due_at, a.status, a.notes,
           u.name AS created_by_name, a.created_at
    FROM activities a
    LEFT JOIN users u ON u.id = a.created_by";

#[derive(Deserialize)]
pub struct ActivityInput {
    pub title: String,
    #[serde(default)]
    pub related_to: String,
    pub activity_type: String,
    pub due_at: Option<DateTime<Utc>>,
    pub status: String,
    #[serde(default)]
    pub notes: String,
}

impl ActivityInput {
    fn validate(&self) -> AppResult<()> {
        require(&self.title, "Activity")?;
        one_of(&self.activity_type, TYPES, "Type")?;
        one_of(&self.status, STATUSES, "Status")
    }
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<Activity>>> {
    let activities = sqlx::query_as::<_, Activity>(AssertSqlSafe(format!(
        "{SELECT_ACTIVITY}
         WHERE a.title ILIKE $1 OR a.related_to ILIKE $1
         ORDER BY a.due_at DESC NULLS LAST"
    )))
        .bind(query.pattern())
        .fetch_all(&state.db)
        .await?;
    Ok(Json(activities))
}

pub async fn get_one(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Activity>> {
    Ok(Json(find(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<ActivityInput>,
) -> AppResult<(StatusCode, Json<Activity>)> {
    input.validate()?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO activities (title, related_to, activity_type, due_at, status, notes, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
        .bind(input.title.trim())
        .bind(input.related_to.trim())
        .bind(&input.activity_type)
        .bind(input.due_at)
        .bind(&input.status)
        .bind(input.notes.trim())
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
    Ok((StatusCode::CREATED, Json(find(&state, id).await?)))
}

pub async fn update(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<ActivityInput>,
) -> AppResult<Json<Activity>> {
    input.validate()?;
    let result = sqlx::query(
        "UPDATE activities
         SET title = $1, related_to = $2, activity_type = $3, due_at = $4, status = $5, notes = $6
         WHERE id = $7",
    )
        .bind(input.title.trim())
        .bind(input.related_to.trim())
        .bind(&input.activity_type)
        .bind(input.due_at)
        .bind(&input.status)
        .bind(input.notes.trim())
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Activity"));
    }
    Ok(Json(find(&state, id).await?))
}

pub async fn complete(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Activity>> {
    let result = sqlx::query("UPDATE activities SET status = 'Completed' WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Activity"));
    }
    Ok(Json(find(&state, id).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM activities WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Activity"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn find(state: &AppState, id: i64) -> AppResult<Activity> {
    sqlx::query_as::<_, Activity>(AssertSqlSafe(format!("{SELECT_ACTIVITY} WHERE a.id = $1")))
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Activity"))
}