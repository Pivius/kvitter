use axum::{
	extract::State, Json, 
	response::IntoResponse, http::StatusCode
};
use sqlx::PgPool;
use crate::{
	models::{
		user::{RegisterPayload, User, PublicUser},
		response::ApiResponse
	},
	auth::{
		hash::{hash_password, verify_password},
		jwt::generate_jwt_token
	},
	util::{
		validation::validate_password,
		error::{AppError, AppResult},
		user_service::{is_email_unique, fetch_user_by_email}
	}
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
	pub token: String,
	pub user: PublicUser,
}

pub async fn signup(
	State(pool): State<PgPool>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	let result: AppResult<()> = async {
		is_email_unique(&pool, &payload.email).await?;
		validate_password(&payload.password)?;

		let password_hash = hash_password(&payload.password)
			.map_err(|err| AppError::Internal(err.to_string()))?;

		sqlx::query_as::<_, User>("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *")
			.bind(&payload.email)
			.bind(&password_hash)
			.fetch_one(&pool)
			.await
			.map_err(|_| AppError::Internal("Failed to create user".into()))?;

		Ok(())
	}.await;

	ApiResponse::from_result(result, StatusCode::CREATED).into_response()
}

pub async fn login(
	State(pool): State<PgPool>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	let result: AppResult<AuthResponse> = async {
		let user = fetch_user_by_email(&pool, &payload.email).await
			.map_err(|err| match err {
				AppError::NotFound(_) => AppError::Auth("Invalid credentials".into()),
				_ => err,
			})?;
		let is_valid = verify_password(&payload.password, &user.password_hash)
			.map_err(|err| AppError::Internal(err.to_string()))?;

		match is_valid {
			true => {
				let token = generate_jwt_token(&user)
					.map_err(|_| AppError::Auth("Failed to generate token".into()))?;

				Ok(AuthResponse {
					token,
					user: user.into(),
				})
			},
			false => Err(AppError::Auth("Invalid credentials".into())),
		}

	}.await;

	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn health_check() -> impl IntoResponse {
	(StatusCode::OK, Json("Service is up and running"))
}