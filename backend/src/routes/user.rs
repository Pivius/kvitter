use axum::{
	extract::{Path, State},
	Json,
	response::IntoResponse,
	http::StatusCode,
};
use sqlx::PgPool;
use crate::models::user::{User, UpdateUserPayload, ChangePasswordPayload, PublicUser};
use crate::auth::hash::hash_password;
use crate::auth::jwt::AuthUser;
use crate::util::result::{AppResult, AppResponse};
use crate::util::error::AppError;

pub async fn get_user_by_uuid(
	Path(user_id): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
		.bind(user_id)
		.fetch_optional(&pool)
		.await;

	match user {
		Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
		Ok(None) => (StatusCode::NOT_FOUND, Json("User not found")).into_response(),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json("Error fetching user")).into_response(),
	}
}

pub async fn get_user_by_email(
	Path(email): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
		.bind(email)
		.fetch_optional(&pool)
		.await;

	match user {
		Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
		Ok(None) => (StatusCode::NOT_FOUND, Json("User not found")).into_response(),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json("Error fetching user")).into_response(),
	}
}

pub async fn update_user(
	Path(user_id): Path<String>,
	State(pool): State<PgPool>,
	Json(payload): Json<UpdateUserPayload>,
) -> impl IntoResponse {
	let mut tx = match pool.begin().await {
		Ok(t) => t,
		Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json("Error starting transaction")).into_response(),
	};

	if let Some(email) = &payload.email {
		if sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
			.bind(email)
			.bind(&user_id)
			.execute(&mut *tx)
			.await
			.is_err()
		{
			return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to update email")).into_response();
		}
	}

	if let Some(password) = &payload.password {
		let hashed = match hash_password(password) {
			Ok(h) => h,
			Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to hash password")).into_response(),
		};

		if sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
			.bind(&hashed)
			.bind(&user_id)
			.execute(&mut *tx)
			.await
			.is_err()
		{
			return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to update password")).into_response();
		}
	}

	if tx.commit().await.is_err() {
		return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to commit changes")).into_response();
	}

	(StatusCode::OK, Json("User updated")).into_response()
}

pub async fn delete_user(
	Path(user_id): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result = sqlx::query("DELETE FROM users WHERE id = $1")
		.bind(user_id)
		.execute(&pool)
		.await;

	match result {
		Ok(_) => (StatusCode::OK, Json("User deleted")).into_response(),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to delete user")).into_response(),
	}
}

pub async fn get_me(
	AuthUser(user_id): AuthUser,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
		.bind(&user_id)
		.fetch_optional(&pool)
		.await;

	match user {
		Ok(Some(user)) => (StatusCode::OK, Json(PublicUser::from(&user))).into_response(),
		Ok(None) => (StatusCode::NOT_FOUND, Json("User not found")).into_response(),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json("Error fetching user")).into_response(),
	}
}

/// Validates the password according to your application's requirements.
/// Prerequisites include
/// - Minimum length
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
pub fn password_is_valid(password: &str) -> AppResult<()> {
	if password.len() < 8 {
		return Err(AppError::bad_request("Password must be at least 8 characters long"));
	}
	if !password.chars().any(|c| c.is_uppercase()) {
		return Err(AppError::bad_request("Password must contain at least one uppercase letter"));
	}
	if !password.chars().any(|c| c.is_lowercase()) {
		return Err(AppError::bad_request("Password must contain at least one lowercase letter"));
	}
	if !password.chars().any(|c| c.is_digit(10)) {
		return Err(AppError::bad_request("Password must contain at least one digit"));
	}
	Ok(())
}

pub async fn change_password(
	AuthUser(user_id): AuthUser,
	State(pool): State<PgPool>,
	Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
	match password_is_valid(&payload.new_password) {
		Ok(_) => {
			let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
				.bind(&user_id)
				.fetch_one(&pool)
				.await;

			let user = match user {
				Ok(u) => u,
				Err(_) => return (StatusCode::NOT_FOUND, Json("User not found")).into_response(),
			};

			if !crate::auth::hash::verify_password(&payload.old_password, &user.password_hash).unwrap_or(false) {
				return (StatusCode::UNAUTHORIZED, Json("Invalid old password")).into_response();
			}

			let hashed = match hash_password(&payload.new_password) {
				Ok(h) => h,
				Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to hash password")).into_response(),
			};

			let res = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
				.bind(&hashed)
				.bind(&user_id)
				.execute(&pool)
				.await;

			match res {
				Ok(_) => (StatusCode::OK, Json("Password updated")).into_response(),
				Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to update password")).into_response(),
			}
		},
		Err(err) => return err.into_response(),
	}
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