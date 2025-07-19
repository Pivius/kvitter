use sqlx::PgPool;
use uuid::Uuid;
use crate::{
	util::error::{AppError, AppResult},
	models::user::{PublicUser, User}
};

pub async fn is_email_unique(pool: &PgPool, email: &str) -> AppResult<()> {
	let count = sqlx::query_scalar::<_, i64>
		("SELECT COUNT(*) FROM users WHERE email = $1")
		.bind(email)
		.fetch_one(pool)
		.await
		.map_err(|_| AppError::Internal("Failed to check email uniqueness".into()))?;

	match count {
		0 => Ok(()),
		_ => Err(AppError::Auth("Email is already taken".into())),
	}
}

pub async fn fetch_user_by_uuid(pool: &PgPool, user_id: &Uuid) -> AppResult<User> {
	sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
		.bind(user_id)
		.fetch_optional(pool)
		.await
		.map_err(|_| AppError::Internal("Error fetching user".into()))?
		.ok_or(AppError::NotFound("User not found".into()))
}

pub async fn fetch_user_by_email(pool: &PgPool, email: &str) -> AppResult<User> {
	sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
		.bind(email)
		.fetch_optional(pool)
		.await
		.map_err(|_| AppError::Internal("Error fetching user".into()))?
		.ok_or(AppError::NotFound("User not found".into()))
}

pub async fn delete_user_by_uuid(pool: &PgPool, user_id: &Uuid) -> AppResult<()> {
	sqlx::query("DELETE FROM users WHERE id = $1")
		.bind(user_id)
		.execute(pool)
		.await
		.map_err(|_| AppError::Internal("Failed to delete user".into()))?;
	
	Ok(())
}

pub async fn update_user_password(pool: &PgPool, user_id: &Uuid, password_hash: &str) -> AppResult<()> {
	sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
		.bind(password_hash)
		.bind(user_id)
		.execute(pool)
		.await
		.map_err(|_| AppError::Internal("Failed to update password".into()))?;
	
	Ok(())
}

pub async fn fetch_and_map_by_uuid(pool: &PgPool, user_id: &Uuid) -> AppResult<PublicUser> {
	fetch_user_by_uuid(pool, user_id)
		.await
		.map(|user| user.into())
}

pub async fn fetch_and_map_by_email(pool: &PgPool, email: &str) -> AppResult<PublicUser> {
	fetch_user_by_email(pool, email)
		.await
		.map(|user| user.into())
}
