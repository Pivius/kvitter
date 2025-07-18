use axum::{Router, routing::{post, get, put, delete}, extract::State};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use jsonwebtoken::TokenData;
use serde_json::json;
use sqlx::{PgPool};
use tower::ServiceExt;
use dotenvy::from_filename;
use crate::{models::{response::ApiResponse, user::{PublicUser, RegisterPayload}}, routes::{auth::{health_check, login, signup, AuthResponse}, user}, util::error::AppResult};

#[ctor::ctor]
fn init() {
	let _ = from_filename(".env.test");
}

fn build_app(pool: PgPool) -> Router {
	Router::new()
		.without_v07_checks()
		.route("/signup", post(signup))
		.route("/login", post(login))
		.route("/health", get(health_check))
		.route("/me", get(user::get_me))
		.route("/me/password", put(user::change_password))
		.with_state(pool)
}

#[sqlx::test]
async fn test_health_check(pool: PgPool) {
	let app = build_app(pool);
	let response = app
		.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
		.await
		.unwrap();
	println!("Response: {:?}", response.status());
	assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn test_signup(pool: PgPool) {
	let app = build_app(pool);

	let payload = json!({
		"email": "test@example.com",
		"password": "SecurePassword123"
	});

	let response = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test]
async fn test_signup_used_email(pool: PgPool) {
	let app = build_app(pool.clone());

	let payload = json!({
		"email": "test@example.com",
		"password": "SecurePassword123"
	});

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::CREATED);

	let second_response = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(second_response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn test_signup_invalid_password(pool: PgPool) {
	let app = build_app(pool);

	let payload = json!({
		"email": "test@example.com",
		"password": "short"
	});

	let response = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn test_login(pool: PgPool) {
	let app = build_app(pool.clone());

	let payload = json!({
		"email": "test@example.com",
		"password": "SecurePassword123"
	});

	let signup_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(signup_response.status(), StatusCode::CREATED);

	let login_response = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(login_response.status(), StatusCode::OK);

	let response_body = axum::body::to_bytes(login_response.into_body(), 8 * 1024).await.unwrap();
	let api_response: ApiResponse<AuthResponse> = serde_json::from_slice(&response_body).unwrap();
	let auth_response = api_response.data.unwrap();

	assert!(!&auth_response.token.is_empty(), "Token should not be empty");
	let public_user = &auth_response.user;
	assert_eq!(&public_user.email, payload["email"].as_str().unwrap());
	assert_eq!(&public_user.id, &auth_response.user.id);
}

#[sqlx::test]
async fn test_login_invalid_password(pool: PgPool) {
	let app = build_app(pool.clone());

	let correct = json!({
		"email": "test@example.com",
		"password": "CorrectPassword123"
	});

	let wrong = json!({
		"email": "test@example.com",
		"password": "WrongPassword123"
	});

	let _ = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(correct.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	let login_response = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(wrong.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn test_get_me(pool: PgPool) {
	let app = build_app(pool.clone());

	let payload = json!({
		"email": "test@example.com",
		"password": "SecurePassword123"
	});

	let signup_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(signup_response.status(), StatusCode::CREATED);

	let login_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(login_response.status(), StatusCode::OK);

	let response_body = axum::body::to_bytes(login_response.into_body(), 8 * 1024).await.unwrap();
	let api_response: ApiResponse<AuthResponse> = serde_json::from_slice(&response_body).unwrap();
	let auth_response = api_response.data.unwrap();
	let token = auth_response.token;
	let me_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("GET")
				.uri("/me")
				.header("Authorization", format!("Bearer {}", token))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(me_response.status(), StatusCode::OK);

	let user_data = axum::body::to_bytes(me_response.into_body(), 8 * 1024).await.unwrap();
	let user_response: ApiResponse<PublicUser> = serde_json::from_slice(&user_data).unwrap();
	let user = user_response.data.unwrap();

	assert_eq!(&user.email, payload["email"].as_str().unwrap());
	assert_eq!(&user.id, &auth_response.user.id);
}

#[sqlx::test]
async fn test_change_password(pool: PgPool) {
	let app = build_app(pool.clone());

	let payload = json!({
		"email": "test@example.com",
		"password": "SecurePassword123"
	});

	let signup_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/signup")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(signup_response.status(), StatusCode::CREATED);

	let login_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(login_response.status(), StatusCode::OK);

	let response_body = axum::body::to_bytes(login_response.into_body(), 8 * 1024).await.unwrap();
	let api_response: ApiResponse<AuthResponse> = serde_json::from_slice(&response_body).unwrap();
	let auth_response = api_response.data.unwrap();
	let token = auth_response.token;
	let change_password_payload = json!({
		"old_password": "SecurePassword123",
		"new_password": "NewSecurePassword123"
	});
	let change_password_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("PUT")
				.uri("/me/password")
				.header("Authorization", format!("Bearer {}", token))
				.header("Content-Type", "application/json")
				.body(Body::from(change_password_payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();
	
	assert_eq!(change_password_response.status(), StatusCode::NO_CONTENT);

	let login_with_new_password_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(json!({
					"email": "test@example.com",
					"password": "NewSecurePassword123"
				}).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(login_with_new_password_response.status(), StatusCode::OK);

	let new_response_body = axum::body::to_bytes(login_with_new_password_response.into_body(), 8 * 1024).await.unwrap();
	let new_api_response: ApiResponse<AuthResponse> = serde_json::from_slice(&new_response_body).unwrap();
	let new_auth_response = new_api_response.data.unwrap();
	let new_token = new_auth_response.token;
	assert!(!new_token.is_empty(), "New token should not be empty");

	let login_with_old_password_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(json!({
					"email": "test@example.com",
					"password": "SecurePassword123"
				}).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(login_with_old_password_response.status(), StatusCode::UNAUTHORIZED);

	let change_password_invalid_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("PUT")
				.uri("/me/password")
				.header("Authorization", format!("Bearer {}", new_token))
				.header("Content-Type", "application/json")
				.body(Body::from(json!({
					"old_password": "WrongPassword123",
					"new_password": "AnotherNewPassword123"
				}).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(change_password_invalid_response.status(), StatusCode::UNAUTHORIZED);

	let change_password_error_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("PUT")
				.uri("/me/password")
				.header("Authorization", format!("Bearer {}", new_token))
				.header("Content-Type", "application/json")
				.body(Body::from(json!({
					"old_password": "WrongPassword123",
					"new_password": ""
				}).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(change_password_error_response.status(), StatusCode::BAD_REQUEST);

}