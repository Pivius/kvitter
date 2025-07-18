use axum::{extract::State, Json, response::IntoResponse};
use axum::http::StatusCode;
use sqlx::PgPool;
use crate::models::{
	user::{RegisterPayload, User, PublicUser},
	response::ApiResponse
};
use crate::auth::hash::{hash_password, verify_password};
use crate::auth::jwt::generate_jwt_token;
use crate::routes::user::password_is_valid;
use crate::util::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
	pub token: String,
	pub user: PublicUser,
}

pub async fn is_email_unique(
	State(pool): State<PgPool>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	let result: AppResult<bool> = async {
		let count = sqlx::query_scalar::<_, i64>(
			"SELECT COUNT(*) FROM users WHERE email = $1"
		)
		.bind(&payload.email)
		.fetch_one(&pool)
		.await
		.map_err(|_| AppError::Internal("Failed to check email uniqueness".into()))?;

		Ok(count == 0)
	}.await;

	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn signup(
	State(pool): State<PgPool>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	let result: AppResult<AuthResponse> = async {
		match password_is_valid(&payload.password) {
			Ok(_) => {
				let hashed = hash_password(&payload.password)
					.map_err(|_| AppError::Internal("Failed to hash password".into()))?;
				let user = sqlx::query_as::<_, User>(
					"INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *"
					)
					.bind(&payload.email)
					.bind(&hashed)
					.fetch_one(&pool)
					.await
					.map_err(|_| AppError::Internal("Failed to create user".into()))?;

				let token: AppResult<String> = generate_jwt_token(&user)
					.map_err(|_| AppError::Auth("Failed to generate token".into()));
				let public_user = PublicUser::from(&user);

				match token {
					Ok(token) => Ok(AuthResponse {
						token,
						user: public_user,
					}),
					Err(err) => Err(err),
				}
			},
			Err(err) => return Err(err),
		}
	}.await;

	ApiResponse::from_result(result, StatusCode::CREATED).into_response()
}

pub async fn login(
	State(pool): State<PgPool>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	let result: AppResult<AuthResponse> = async {
		let user = sqlx::query_as::<_, User>
			("SELECT * FROM users WHERE email = $1")
			.bind(&payload.email)
			.fetch_one(&pool)
			.await
			.map_err(|_| AppError::Auth("Invalid credentials".into()))?;

		match verify_password(&payload.password, &user.password_hash) {
			Ok(true) => {
				let token: AppResult<String> = generate_jwt_token(&user)
					.map_err(|_| AppError::Auth("Failed to generate token".into()));
				let public_user = PublicUser::from(&user);

				match token {
					Ok(token) => Ok(AuthResponse {
						token,
						user: public_user,
					}),
					Err(err) => Err(err),
				}
			},
			_ => Err(AppError::Auth("Invalid credentials".into()))
		}
	}.await;

	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn health_check() -> impl IntoResponse {
	(StatusCode::OK, Json("Service is up and running"))
}