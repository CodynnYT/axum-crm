// API errors and validation helpers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound(&'static str),
    Conflict(String),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Please sign in".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "You are not allowed to do that".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, format!("{msg} not found")),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".to_string())
            },
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {

    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record"),

            sqlx::Error::Database(db) if db.is_unique_violation() => {
                AppError::Conflict("Record already exists".to_string())
            }

            sqlx::Error::Database(db) if db.is_check_violation() => {
                AppError::BadRequest("Invalid input".to_string())
            }

            _ => AppError::Internal(err.to_string())

        }
    }

}

pub fn one_of (value: &str, allowed: &[&str], field: &str) -> AppResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "{field} must be one of: {}",
            allowed.join(", ")
        )))
    }
}

pub fn require(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::BadRequest(format!(
            "{field} is required",
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blank_text_rejected() {
        assert!(require(" ", "Name").is_err());
        assert!(require("Sarah", "Name").is_ok());
    }

    #[test]
    fn only_listed_values_pass() {
        assert!(one_of("Sarah", &["Sarah", "John"], "Name").is_ok());
        assert!(one_of("Mary", &["Sarah", "John"], "Name").is_err());
    }
}
