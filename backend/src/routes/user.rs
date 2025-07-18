use axum::{
	extract::{Path, State},
	Json,
	response::IntoResponse,
	http::StatusCode,
};
use sqlx::PgPool;
use crate::{auth::hash::verify_password, models::{
	response::ApiResponse, user::{ChangePasswordPayload, PublicUser, UpdateUserPayload, User}
}, routes::auth::AuthResponse};
use crate::auth::hash::hash_password;
use crate::auth::jwt::AuthUser;
use crate::util::error::{AppError, AppResult};

pub async fn get_user_by_uuid(
	Path(user_id): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result: AppResult<PublicUser> = async {
		let user = sqlx::query_as::<_, User>
			("SELECT * FROM users WHERE id = $1")
			.bind(&user_id)
			.fetch_optional(&pool)
			.await
			.map_err(|_| AppError::Internal("Error fetching user".into()))
			.and_then(|user| 
				user.ok_or(AppError::NotFound("User not found".into()))
			)?;

		Ok(PublicUser::from(&user))
	}.await;

	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn get_user_by_email(
	Path(email): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result: AppResult<PublicUser> = async {
		let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
			.bind(&email)
			.fetch_optional(&pool)
			.await
			.map_err(|_| AppError::Internal("Error fetching user".into()))?
			.ok_or(AppError::NotFound("User not found".into()))?;

		Ok(PublicUser::from(&user))
	}.await;

	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn delete_user(
	Path(user_id): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result: AppResult<()> = async {
		sqlx::query("DELETE FROM users WHERE id = $1")
			.bind(&user_id)
			.execute(&pool)
			.await
			.map_err(|_| AppError::Internal("Failed to delete user".into()))?;

		Ok(())
	}.await;

	ApiResponse::from_result(result, StatusCode::NO_CONTENT).into_response()
}

pub async fn get_me(
	AuthUser(user_id): AuthUser,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result: AppResult<PublicUser> = async {
		let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
			.bind(&user_id)
			.fetch_optional(&pool)
			.await
			.map_err(|_| AppError::Internal("Error fetching user".into()))?
			.ok_or(AppError::NotFound("User not found".into()))?;

		Ok(PublicUser::from(&user))
	}.await;

	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

/// Validates the password according to your application's requirements.
/// Prerequisites include
/// - Minimum length
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
pub fn password_is_valid(password: &str) -> AppResult<()> {
	if password.len() < 8 {
		return Err(AppError::BadRequest("Password must be at least 8 characters long".into()));
	}
	if !password.chars().any(|c| c.is_uppercase()) {
		return Err(AppError::BadRequest("Password must contain at least one uppercase letter".into()));
	}
	if !password.chars().any(|c| c.is_lowercase()) {
		return Err(AppError::BadRequest("Password must contain at least one lowercase letter".into()));
	}
	if !password.chars().any(|c| c.is_digit(10)) {
		return Err(AppError::BadRequest("Password must contain at least one digit".into()));
	}
	Ok(())
}

pub async fn change_password(
	AuthUser(user_id): AuthUser,
	State(pool): State<PgPool>,
	Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
	let result: AppResult<()> = async {
		password_is_valid(&payload.new_password)?;

		let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
			.bind(&user_id)
			.fetch_optional(&pool)
			.await
			.map_err(|_| AppError::Internal("Error fetching user".into()))?
			.ok_or(AppError::NotFound("User not found".into()))?;

		if !verify_password(&payload.old_password, &user.password_hash).unwrap() {
			return Err(AppError::Auth("Current password is incorrect".into()));
		}

		let hashed = hash_password(&payload.new_password)
			.map_err(|_| AppError::Internal("Failed to hash new password".into()))?;

		sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
			.bind(&hashed)
			.bind(&user_id)
			.execute(&pool)
			.await
			.map_err(|_| AppError::Internal("Failed to update password".into()))?;

		Ok(())
	}.await;

	ApiResponse::from_result(result, StatusCode::NO_CONTENT).into_response()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_password_is_valid() {
		assert!(password_is_valid("Valid1").is_err());
		assert!(password_is_valid("validpassword").is_err());
		assert!(password_is_valid("VALIDPASSWORD").is_err());
		assert!(password_is_valid("ValidPassword").is_err());
		assert!(password_is_valid("Valid1Password").is_ok());
		assert!(password_is_valid("Valid1Password!").is_ok());
	}
}