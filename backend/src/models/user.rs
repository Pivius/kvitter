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

/// # PublicUser
/// A public representation of a user, excluding sensitive information like password hash.
/// This is used for responses that do not require sensitive data.
#[derive(Serialize, Deserialize)]
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

/// # UpdateUserPayload
/// Contains fields that the database will try to update.
/// Fields that are `None` will not be updated.
/// the `email` field is optional, but if provided, it must be unique.
/// The `password` field is optional, but if provided, adhere to the password policy.
/// If the `password` is provided, it will not be hashed here, but should be hashed in the handler before saving to the database.
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