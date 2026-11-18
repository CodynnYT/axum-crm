use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::AssertSqlSafe;

use crate::auth::AuthUser;
use crate::error::{require, AppError, AppResult};
use crate::handlers::deals::Deal;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct Report {
    pub id: i64,
    pub title: String,
    pub date_from: NaiveDate,
    pub date_to: NaiveDate,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

const SELECT_REPORT: &str = "
    SELECT r.id, r.title, r.date_from, r.date_to,
           u.name AS created_by_name, r.created_at
    FROM reports r
    LEFT JOIN users u ON u.id = r.created_by";

#[derive(Serialize, sqlx::FromRow)]
pub struct Summary {
    pub new_leads: i64,
    pub new_customers: i64,
    pub deals_won: i64,
    pub revenue_won: f64,
    pub deals_lost: i64,
}

#[derive(Serialize)]
pub struct ReportWithSummary {
    pub report: Report,
    pub summary: Summary,
}

#[derive(Deserialize)]
pub struct ReportInput {
    pub title: String,
    pub date_from: NaiveDate,
    pub date_to: NaiveDate,
}

pub async fn list(State(state): State<AppState>, _user: AuthUser) -> AppResult<Json<Vec<Report>>> {
    let reports = sqlx::query_as::<_, Report>(AssertSqlSafe(format!("{SELECT_REPORT} ORDER BY r.created_at DESC")))
        .fetch_all(&state.db)
        .await?;
    Ok(Json(reports))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<ReportInput>,
) -> AppResult<(StatusCode, Json<Report>)> {
    require(&input.title, "Report title")?;
    if input.date_from > input.date_to {
        return Err(AppError::BadRequest("Start date must be before end date".into()));
    }

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO reports (title, date_from, date_to, created_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
        .bind(input.title.trim())
        .bind(input.date_from)
        .bind(input.date_to)
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;

    Ok((StatusCode::CREATED, Json(find(&state, id).await?)))
}

pub async fn get_one(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ReportWithSummary>> {
    let report = find(&state, id).await?;

    let summary = sqlx::query_as::<_, Summary>(
        "SELECT
            (SELECT COUNT(*) FROM leads     WHERE created_at::date BETWEEN $1 AND $2) AS new_leads,
            (SELECT COUNT(*) FROM customers WHERE created_at::date BETWEEN $1 AND $2) AS new_customers,
            (SELECT COUNT(*) FROM deals WHERE stage = 'Won'
                AND COALESCE(expected_close, created_at::date) BETWEEN $1 AND $2) AS deals_won,
            (SELECT COALESCE(SUM(amount), 0) FROM deals WHERE stage = 'Won'
                AND COALESCE(expected_close, created_at::date) BETWEEN $1 AND $2) AS revenue_won,
            (SELECT COUNT(*) FROM deals WHERE stage = 'Lost'
                AND COALESCE(expected_close, created_at::date) BETWEEN $1 AND $2) AS deals_lost",
    )
        .bind(report.date_from)
        .bind(report.date_to)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(ReportWithSummary { report, summary }))
}

pub async fn csv(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let report = find(&state, id).await?;

    let deals = sqlx::query_as::<_, Deal>(
        "SELECT d.id, d.name, d.customer_id, c.name AS customer_name,
                d.amount, d.stage, d.expected_close, d.created_at
         FROM deals d
         LEFT JOIN customers c ON c.id = d.customer_id
         WHERE d.created_at::date BETWEEN $1 AND $2
         ORDER BY d.created_at",
    )
        .bind(report.date_from)
        .bind(report.date_to)
        .fetch_all(&state.db)
        .await?;

    let mut body = String::from("Deal,Customer,Amount,Stage,Expected close,Created\n");
    for deal in deals {
        body.push_str(&format!(
            "{},{},{:.2},{},{},{}\n",
            csv_field(&deal.name),
            csv_field(deal.customer_name.as_deref().unwrap_or("")),
            deal.amount,
            deal.stage,
            deal.expected_close.map(|d| d.to_string()).unwrap_or_default(),
            deal.created_at.date_naive(),
        ));
    }

    let filename = format!("attachment; filename=\"report-{id}.csv\"");
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, filename),
        ],
        body,
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM reports WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Report"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn find(state: &AppState, id: i64) -> AppResult<Report> {
    sqlx::query_as::<_, Report>(AssertSqlSafe(format!("{SELECT_REPORT} WHERE r.id = $1")))
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Report"))
}

fn csv_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_fields_are_quoted() {
        assert_eq!(csv_field("Globex, Inc"), "\"Globex, Inc\"");
        assert_eq!(csv_field("Acme \"Pro\""), "\"Acme \"\"Pro\"\"\"");
    }
}