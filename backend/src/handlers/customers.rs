use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::error::{require, AppError, AppResult};
use crate::handlers::SearchQuery;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    pub company: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CustomerInput {
    pub name: String,
    #[serde(default)]
    pub company: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub notes: String,
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<Customer>>> {
    let customers = sqlx::query_as::<_, Customer>(
        "SELECT * FROM customers
         WHERE name ILIKE $1 OR company ILIKE $1 OR email ILIKE $1
         ORDER BY created_at DESC",
    )
        .bind(query.pattern())
        .fetch_all(&state.db)
        .await?;
    Ok(Json(customers))
}

pub async fn get_one(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Customer>> {
    let customer = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Customer"))?;
    Ok(Json(customer))
}

pub async fn create(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(input): Json<CustomerInput>,
) -> AppResult<(StatusCode, Json<Customer>)> {
    require(&input.name, "Customer name")?;
    let customer = sqlx::query_as::<_, Customer>(
        "INSERT INTO customers (name, company, email, phone, notes)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *",
    )
        .bind(input.name.trim())
        .bind(input.company.trim())
        .bind(input.email.trim())
        .bind(input.phone.trim())
        .bind(input.notes.trim())
        .fetch_one(&state.db)
        .await?;
    Ok((StatusCode::CREATED, Json(customer)))
}

pub async fn update(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<CustomerInput>,
) -> AppResult<Json<Customer>> {
    require(&input.name, "Customer name")?;
    let customer = sqlx::query_as::<_, Customer>(
        "UPDATE customers SET name = $1, company = $2, email = $3, phone = $4, notes = $5
         WHERE id = $6
         RETURNING *",
    )
        .bind(input.name.trim())
        .bind(input.company.trim())
        .bind(input.email.trim())
        .bind(input.phone.trim())
        .bind(input.notes.trim())
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Customer"))?;
    Ok(Json(customer))
}

pub async fn delete(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM customers WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Customer"));
    }
    Ok(StatusCode::NO_CONTENT)
}