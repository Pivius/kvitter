use axum::{
	extract::FromRequestParts, 
	http::request::Parts
};
use jsonwebtoken::{
	decode, encode, Algorithm, 
	DecodingKey, EncodingKey, Header, Validation
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::{
	models::user::{User, PublicUser},
	util::error::{AppError, AppResult}
};

const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	pub sub: Uuid,
	pub exp: usize,
}
pub struct AuthUser(pub Uuid);

impl From<User> for AuthUser {
	fn from(user: User) -> Self {
		AuthUser(user.id)
	}
}

impl From<PublicUser> for AuthUser {
	fn from(user: PublicUser) -> Self {
		AuthUser(user.id)
	}
}

impl<S> FromRequestParts<S> for AuthUser
where
	S: Send + Sync,
{
	type Rejection = AppError;

	async fn from_request_parts(
		parts: &mut Parts,
		_state: &S,
	) -> AppResult<Self> {
		let auth_result = parts.headers.get("authorization")
			.and_then(|h| h.to_str().ok())
			.ok_or(AppError::Auth("Missing authorization header".into()))
			.and_then(|header| {
				header.strip_prefix("Bearer ")
					.ok_or(AppError::Auth("Invalid authorization header".into()))
			});
		let token = match auth_result {
			Ok(t) => t,
			Err(e) => return Err(e),
		};
		let secret = std::env::var("JWT_SECRET")
			.map_err(|_| AppError::Internal("JWT secret not configured".into()))?;
		let token_data = decode::<Claims>(
			token,
			&DecodingKey::from_secret(secret.as_bytes()),
			&Validation::new(Algorithm::HS256),
		).map_err(|_| AppError::Auth("Invalid or expired token".into()))?;

		Ok(AuthUser(token_data.claims.sub))
	}
}

pub fn generate_jwt_token(user: &User) -> AppResult<String> {
	let secret = std::env::var("JWT_SECRET")
		.map_err(|_| AppError::Internal("JWT secret not set".into()))?;
	let exp = (chrono::Utc::now() + chrono::Duration::hours(JWT_EXPIRATION_HOURS))
		.timestamp() as usize;
	let claims = Claims {
		sub: user.id,
		exp,
	};

	encode(
		&Header::default(),
		&claims,
		&EncodingKey::from_secret(secret.as_bytes()),
	).map_err(|_| AppError::Internal("Failed to generate token".into()))
}

pub fn validate_jwt(token: &str) -> AppResult<Claims> {
	let secret = std::env::var("JWT_SECRET")
		.map_err(|_| AppError::Internal("JWT secret not configured".into()))?;

	decode::<Claims>(
		token,
		&DecodingKey::from_secret(secret.as_bytes()),
		&Validation::new(Algorithm::HS256),
	)
	.map(|token_data| token_data.claims)
	.map_err(|_| AppError::Auth("Invalid or expired token".into()))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::user::User;

	#[tokio::test]
	async fn test_generate_and_validate_jwt() {
		let user = User {
			id: Uuid::new_v4(),
			email: "<Email>".into(),
			password_hash: "<PasswordHash>".into(),
			created_at: chrono::Utc::now().naive_utc(),
		};
		let token = generate_jwt_token(&user).unwrap();
		let claims = validate_jwt(&token).unwrap();

		assert_eq!(claims.sub, user.id);
	}
}