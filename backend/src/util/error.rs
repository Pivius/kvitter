use axum::{body::Body, http::{Response, StatusCode}, response::IntoResponse, Json};

#[derive(Debug)]
pub struct AppError {
	pub status: StatusCode,
	pub message: &'static str
}

impl AppError {
	pub fn new(status: StatusCode, message: &'static str) -> Self {
		Self { status, message }
	}

	pub fn internal(message: &'static str) -> Self {
		Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
	}

	pub fn not_found(message: &'static str) -> Self {
		Self::new(StatusCode::NOT_FOUND, message)
	}

	pub fn unauthorized(message: &'static str) -> Self {
		Self::new(StatusCode::UNAUTHORIZED, message)
	}

	pub fn bad_request(message: &'static str) -> Self {
		Self::new(StatusCode::BAD_REQUEST, message)
	}
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response<Body> {
		(self.status, Json(self.message)).into_response()
	}
}