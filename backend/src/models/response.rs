use serde::{Serialize, Deserialize, Deserializer, de::DeserializeOwned};
use axum::{response::IntoResponse, Json};
use axum::http::StatusCode;
use crate::util::error::{AppError, AppResult};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> 
where 
	T: Serialize
{
	pub status: u16,
	pub data: Option<T>,
	pub error: Option<String>,
}

impl<T> ApiResponse<T> 
where 
	T: Serialize
{
	pub fn success(data: T) -> Self {
		Self::with_status(data, StatusCode::OK)
	}

	pub fn with_status(data: T, status: StatusCode) -> Self {
		Self {
			status: status.as_u16(),
			data: Some(data),
			error: None,
		}
	}

	pub fn error(error: &AppError) -> Self {
		let status = match error {
			AppError::Auth(_) => StatusCode::UNAUTHORIZED,
			AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
			AppError::Validation(_) => StatusCode::BAD_REQUEST,
			AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
			AppError::NotFound(_) => StatusCode::NOT_FOUND,
			AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
			AppError::Forbidden(_) => StatusCode::FORBIDDEN,
		};

		error.log();

		Self {
			status: status.as_u16(),
			data: None,
			error: Some(error.to_string()),
		}
	}

	pub fn from_result(result: AppResult<T>, status: StatusCode) -> Self {
		match result {
			Ok(data) => Self::with_status(data, status),
			Err(err) => Self::error(&err),
		}
	}

	pub fn is_success(&self) -> bool {
		StatusCode::from_u16(self.status).unwrap().is_success()
	}

	pub fn get_error(&self) -> Option<AppError> {
		match &self.error {
			Some(err) => Some(AppError::Internal(err.clone())),
			None => None,
		}
	}
}

impl<T> IntoResponse for ApiResponse<T> 
where
	T: Serialize
{
	fn into_response(self) -> axum::response::Response {
		let status = StatusCode::from_u16(self.status)
			.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

		(status, Json(self)).into_response()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_api_response_success() {
		let resp = ApiResponse::success(json!({"key": "value"}));
		assert_eq!(resp.status, 200);
		assert_eq!(resp.data, Some(json!({"key": "value"})));
		assert!(resp.error.is_none());
	}

	#[test]
	fn test_api_response_with_status() {
		let resp = ApiResponse::with_status(
			json!({"key": "value"}), 
			StatusCode::CREATED
		);
		assert_eq!(resp.status, 201);
		assert_eq!(resp.data, Some(json!({"key": "value"})));
		assert!(resp.error.is_none());
	}

	#[test]
	fn test_api_response_error() {
		let resp: ApiResponse<serde_json::Value> = 
			ApiResponse::error(&AppError::Internal("An error occurred".to_string()));
		assert_eq!(resp.status, 500);
		assert!(resp.data.is_none());
		assert_eq!(resp.error, Some("Internal error: An error occurred".to_string()));
	}

	#[test]
	fn test_api_response_not_found() {
		let resp: ApiResponse<serde_json::Value> = 
			ApiResponse::error(&AppError::NotFound("User not found".to_string()));
		assert_eq!(resp.status, 404);
		assert!(resp.data.is_none());
		assert_eq!(resp.error, Some("Not found: User not found".to_string()));
	}
}