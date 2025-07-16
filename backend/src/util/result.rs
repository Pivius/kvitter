use axum::{body::Body, http::{Response, StatusCode}, response::IntoResponse, Json};
use crate::util::error::AppError;

pub type AppResult<T> = Result<T, AppError>;

pub struct AppResponse<T>(pub AppResult<T>);

impl<T> From<AppResult<T>> for AppResponse<T> {
    fn from(result: AppResult<T>) -> Self {
        AppResponse(result)
    }
}

impl<T> IntoResponse for AppResponse<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        match self.0 {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}