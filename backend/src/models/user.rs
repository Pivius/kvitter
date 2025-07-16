use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
	pub id: Uuid,
	pub email: String,
	pub password_hash: String,
	pub created_at: NaiveDateTime,
}
#[derive(Serialize)]
pub struct PublicUser {
	pub id: Uuid,
	pub email: String,
	pub created_at: NaiveDateTime,
}

impl From<&User> for PublicUser {
	fn from(user: &User) -> Self {
		PublicUser {
			id: user.id,
			email: user.email.clone(),
			created_at: user.created_at,
		}
	}
}

#[derive(Deserialize)]
pub struct RegisterPayload {
	pub email: String,
	pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserPayload {
	pub email: Option<String>,
	pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePasswordPayload {
	pub old_password: String,
	pub new_password: String,
}