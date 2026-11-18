// Leads represent potential customers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::error::{one_of, require, AppError, AppResult};
use crate::handlers::customers::Customer;
use crate::handlers::SearchQuery;
use crate::state::AppState;

pub const SOURCES: [&str; 6] = ["Website", "Referral", "Social Media", "Email", "Cold Call", "Other"];
pub const STAGES: &[&str] = &["New", "Contacted", "Qualified", "Unqualified"];
pub const STATUSES: &[&str] = &["Active", "Inactive"];

#[derive(Serialize, sqlx::FromRow)]
pub struct Lead {
    pub id: i64,
    pub name: String,
    pub company: String,
    pub email: String,
    pub phone: String,
    pub source: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct LeadInput {
    pub name: String,

    #[serde(default)]
    pub company: String,

    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub phone: String,

    pub source: String,

    pub stage: String,

    pub status: String,
}

impl LeadInput {
    fn validate(&self) -> AppResult<()> {
        require(&self.name, "Name")?;
        one_of(&self.source, &SOURCES, "Source")?;
        one_of(&self.stage, STAGES, "Stage")?;
        one_of(&self.status, STATUSES, "Status")
    }
}

// GET /api/leads?q=
pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<Lead>>> {
    // Search name, company, and email, returning the newest leads first.
    let leads = sqlx::query_as::<_, Lead>(
        "SELECT * FROM leads
         WHERE name ILIKE $1 OR company ILIKE $1 OR email ILIKE $1
         ORDER BY created_at DESC",
    )
        .bind(query.pattern())
        .fetch_all(&state.db)
        .await?;
    Ok(Json(leads))
}

// POST /api/leads/{id}/convert
pub async fn convert(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<(StatusCode, Json<Customer>)> {
    let mut tx = state.db.begin().await?;

    let lead = sqlx::query_as::<_, Lead>("DELETE FROM leads WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("Lead"))?;

    let customer = sqlx::query_as::<_, Customer>(
        "INSERT INTO customers (name, company, email, phone, notes)
         VALUES ($1, $2, $3, $4, 'Converted from lead')
         RETURNING *",
    )
        .bind(&lead.name)
        .bind(&lead.company)
        .bind(&lead.email)
        .bind(&lead.phone)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(customer)))
}

// GET /api/leads/{id}
pub async fn get_one(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Lead>> {
    Ok(Json(find(&state, id).await?))
}

// POST /api/leads
pub async fn create(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(input): Json<LeadInput>,
) -> AppResult<(StatusCode, Json<Lead>)> {
    input.validate()?;
    let lead = sqlx::query_as::<_, Lead>(
        "INSERT INTO leads (name, company, email, phone, source, stage, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *",
    )
        .bind(input.name.trim())
        .bind(input.company.trim())
        .bind(input.email.trim())
        .bind(input.phone.trim())
        .bind(&input.source)
        .bind(&input.stage)
        .bind(&input.status)
        .fetch_one(&state.db)
        .await?;
    Ok((StatusCode::CREATED, Json(lead)))
}

// PUT /api/leads/{id}
pub async fn update(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<LeadInput>,
) -> AppResult<Json<Lead>> {
    input.validate()?;
    let lead = sqlx::query_as::<_, Lead>(
        "UPDATE leads
         SET name = $1, company = $2, email = $3, phone = $4, source = $5, stage = $6, status = $7
         WHERE id = $8
         RETURNING *",
    )
        .bind(input.name.trim())
        .bind(input.company.trim())
        .bind(input.email.trim())
        .bind(input.phone.trim())
        .bind(&input.source)
        .bind(&input.stage)
        .bind(&input.status)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Lead"))?;
    Ok(Json(lead))
}

// DELETE /api/leads/{id}
pub async fn delete(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM leads WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Lead"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn find(state: &AppState, id: i64) -> AppResult<Lead> {
    sqlx::query_as::<_, Lead>("SELECT * FROM leads WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Lead"))
}
