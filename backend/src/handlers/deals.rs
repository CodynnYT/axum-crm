use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::AssertSqlSafe;

use crate::auth::AuthUser;
use crate::error::{one_of, require, AppError, AppResult};
use crate::handlers::SearchQuery;
use crate::state::AppState;

pub const STAGES: &[&str] = &["New", "Qualified", "Proposal", "Negotiation", "Won", "Lost"];

#[derive(Serialize, sqlx::FromRow)]
pub struct Deal {
    pub id: i64,
    pub name: String,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: f64,
    pub stage: String,
    pub expected_close: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

const SELECT_DEAL: &str = "
    SELECT d.id, d.name, d.customer_id, c.name AS customer_name,
           d.amount, d.stage, d.expected_close, d.created_at
    FROM deals d
    LEFT JOIN customers c ON c.id = d.customer_id";

#[derive(Deserialize)]
pub struct DealInput {
    pub name: String,
    pub customer_id: Option<i64>,
    pub amount: f64,
    pub stage: String,
    pub expected_close: Option<NaiveDate>,
}

impl DealInput {
    fn validate(&self) -> AppResult<()> {
        require(&self.name, "Deal name")?;
        one_of(&self.stage, STAGES, "Stage")?;
        if self.amount < 0.0 || !self.amount.is_finite() {
            return Err(AppError::BadRequest("Amount must be zero or more".into()));
        }
        Ok(())
    }
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<Deal>>> {
    let stage = query.stage.clone().filter(|s| !s.is_empty());
    let deals = sqlx::query_as::<_, Deal>(AssertSqlSafe(format!(
        "{SELECT_DEAL}
         WHERE (d.name ILIKE $1 OR COALESCE(c.name, '') ILIKE $1)
           AND ($2::TEXT IS NULL OR d.stage = $2)
         ORDER BY d.created_at DESC"
    )))
        .bind(query.pattern())
        .bind(stage)
        .fetch_all(&state.db)
        .await?;
    Ok(Json(deals))
}

pub async fn get_one(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Deal>> {
    Ok(Json(find(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(input): Json<DealInput>,
) -> AppResult<(StatusCode, Json<Deal>)> {
    input.validate()?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO deals (name, customer_id, amount, stage, expected_close)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id",
    )
        .bind(input.name.trim())
        .bind(input.customer_id)
        .bind(input.amount)
        .bind(&input.stage)
        .bind(input.expected_close)
        .fetch_one(&state.db)
        .await?;
    Ok((StatusCode::CREATED, Json(find(&state, id).await?)))
}

pub async fn update(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<DealInput>,
) -> AppResult<Json<Deal>> {
    input.validate()?;
    let result = sqlx::query(
        "UPDATE deals SET name = $1, customer_id = $2, amount = $3, stage = $4, expected_close = $5
         WHERE id = $6",
    )
        .bind(input.name.trim())
        .bind(input.customer_id)
        .bind(input.amount)
        .bind(&input.stage)
        .bind(input.expected_close)
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Deal"));
    }
    Ok(Json(find(&state, id).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM deals WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Deal"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn find(state: &AppState, id: i64) -> AppResult<Deal> {
    sqlx::query_as::<_, Deal>(AssertSqlSafe(format!("{SELECT_DEAL} WHERE d.id = $1")))
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Deal"))
}