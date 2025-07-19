use axum::{
	extract::{Path, State},
	Json,
	response::IntoResponse,
	http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
	auth::{
		hash::{verify_password, hash_password}, 
		jwt::AuthUser
	},
	models::{
		response::ApiResponse, 
		user::ChangePasswordPayload
	},
	util::{
		validation::validate_password,
		error::{AppError, AppResult},
		user_service::{
			fetch_user_by_uuid, 
			delete_user_by_uuid, 
			update_user_password, 
			fetch_and_map_by_uuid, 
			fetch_and_map_by_email
		}
	},
};

pub async fn get_user_by_uuid(
	Path(user_id): Path<Uuid>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result = fetch_and_map_by_uuid(&pool, &user_id).await;
	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn get_user_by_email(
	Path(email): Path<String>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result = fetch_and_map_by_email(&pool, &email).await;
	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn delete_user(
	Path(user_id): Path<Uuid>,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result = delete_user_by_uuid(&pool, &user_id).await;
	ApiResponse::from_result(result, StatusCode::NO_CONTENT).into_response()
}

pub async fn get_me(
	AuthUser(user_id): AuthUser,
	State(pool): State<PgPool>,
) -> impl IntoResponse {
	let result = fetch_and_map_by_uuid(&pool, &user_id).await;
	ApiResponse::from_result(result, StatusCode::OK).into_response()
}

pub async fn change_password(
	AuthUser(user_id): AuthUser,
	State(pool): State<PgPool>,
	Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
	let result: AppResult<()> = async {
		validate_password(&payload.new_password)?;

		let user = fetch_user_by_uuid(&pool, &user_id).await?;
		let is_valid = verify_password(&payload.old_password, &user.password_hash)
			.map_err(|err| AppError::Auth(err.to_string()))?;

		match is_valid {
			true => {
				let hashed = hash_password(&payload.new_password)
					.map_err(|_| AppError::Internal("Failed to hash new password".into()))?;

				update_user_password(&pool, &user_id, &hashed).await?;
				Ok(())
			}
			false => Err(AppError::Auth("Current password is incorrect".into())),
		}
	}.await;

	ApiResponse::from_result(result, StatusCode::NO_CONTENT).into_response()
}