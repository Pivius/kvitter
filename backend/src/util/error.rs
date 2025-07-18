use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::{Serialize, Deserialize};
use serde_json::json;
use thiserror::Error;
use tracing::error;
use console::style;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AppError {
	#[error("Authentication failed: {0}")]
	Auth(String),
	#[error("Database error: {0}")]
	Database(String),
	#[error("Invalid input: {0}")]
	Validation(String),
	#[error("Internal error: {0}")]
	Internal(String),
	#[error("Not found: {0}")]
	NotFound(String),
	#[error("Bad request: {0}")]
	BadRequest(String),
	#[error("Forbidden: {0}")]
	Forbidden(String),
}

impl AppError {
	pub fn log(&self) {
		let (level, color) = match self {
			AppError::Internal(_) | AppError::Database(_) => ("ERROR", console::Color::Red),
			AppError::Auth(_) | AppError::Forbidden(_) => ("WARN", console::Color::Yellow),
			_ => ("INFO", console::Color::White),
		};

		let err_msg = format!("{}", self);
		let formatted = style(format!("[{}] {}", level, err_msg)).fg(color);

		error!(
			error_type = format!("{:?}", self).split('(').next().unwrap(),
			message = %err_msg,
			"Request failed"
		);
		
		eprintln!("{}", formatted);
	}
}


pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, message) = match &self {
			AppError::Auth(msg) => 
				(StatusCode::UNAUTHORIZED, 
					"Authentication failed: ".to_string() + &msg
				),
			AppError::Internal(msg) => 
				(StatusCode::INTERNAL_SERVER_ERROR, 
					"Internal server error: ".to_string() + &msg
				),
			AppError::NotFound(msg) => 
				(StatusCode::NOT_FOUND, "Not found: ".to_string() + &msg),
			AppError::BadRequest(msg) => 
				(StatusCode::BAD_REQUEST, "Bad request: ".to_string() + &msg),
			AppError::Forbidden(msg) => 
				(StatusCode::FORBIDDEN, "Forbidden: ".to_string() + &msg),
			AppError::Database(msg) => 
				(StatusCode::INTERNAL_SERVER_ERROR, "Database error: ".to_string() + &msg),
			AppError::Validation(msg) => 
				(StatusCode::BAD_REQUEST, "Validation error: ".to_string() + &msg),
		};

		self.log();
		(status, Json(json!({ "error": message }))).into_response()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_app_error_logging() {
		let error = AppError::Auth("Invalid credentials".into());
		error.log();
	}
}