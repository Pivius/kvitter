use axum::{extract::State, Json, response::IntoResponse};
use axum::http::StatusCode;
use sqlx::PgPool;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::Utc;
use crate::models::user::{RegisterPayload, User};
use crate::auth::hash::{hash_password, verify_password};
use crate::auth::jwt::Claims;
use crate::util::error::AppError;
use crate::util::result::{AppResult, AppResponse};
use serde_json::json;

const JWT_EXPIRATION_HOURS: i64 = 24;

pub async fn signup(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    match try_signup(&pool, payload).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(err) => err.into_response(),
    }
}

async fn try_signup(pool: &PgPool, payload: RegisterPayload) -> AppResult<User> {
    let hashed = hash_password(&payload.password)
        .map_err(|_| AppError::internal("Failed to hash password"))?;

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *"
    )
        .bind(&payload.email)
        .bind(&hashed)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::internal("Failed to create user"))?;

    Ok(user)
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    match try_login(&pool, payload).await {
        Ok(token) => (StatusCode::OK, Json(json!({ "token": token }))).into_response(),
        Err(err) => err.into_response(),
    }
}

async fn try_login(pool: &PgPool, payload: RegisterPayload) -> AppResult<String> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
        .bind(&payload.email)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::not_found("User not found"))?;

    let valid = verify_password(&payload.password, &user.password_hash)
        .unwrap_or(false);

    if !valid {
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::internal("JWT secret not set"))?;

    let exp = (Utc::now() + chrono::Duration::hours(JWT_EXPIRATION_HOURS)).timestamp() as usize;
    let claims = Claims { sub: user.id, exp };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
        .map_err(|_| AppError::internal("Failed to create token"))?;

    Ok(token)
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json("Service is up and running"))
}