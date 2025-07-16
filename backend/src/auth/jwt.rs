use axum::{
	extract::{FromRequestParts}, 
	http::{request::Parts, StatusCode}
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	pub sub: Uuid,
	pub exp: usize,
}

pub struct AuthUser(pub Uuid);

impl<S> FromRequestParts<S> for AuthUser
where
	S: Send + Sync,
{
	type Rejection = (StatusCode, &'static str);

	async fn from_request_parts(
		parts: &mut Parts,
		_state: &S,
	) -> Result<Self, Self::Rejection> {
		let auth_header = parts.headers.get("authorization")
			.and_then(|h| h.to_str().ok())
			.ok_or((StatusCode::UNAUTHORIZED, "Missing auth header"))?;

		let token = auth_header.strip_prefix("Bearer ")
			.ok_or((StatusCode::UNAUTHORIZED, "Invalid auth header"))?;

		let secret = std::env::var("JWT_SECRET").unwrap();
		let token_data = decode::<Claims>(
			token,
			&DecodingKey::from_secret(secret.as_bytes()),
			&Validation::new(Algorithm::HS256),
		).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

		Ok(AuthUser(token_data.claims.sub))
	}
}