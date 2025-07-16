use axum::{Router, routing::{post, get, put, delete}, extract::State};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use jsonwebtoken::TokenData;
use serde_json::json;
use sqlx::{PgPool};
use tower::ServiceExt;
use dotenvy::from_filename;
use crate::{routes::auth::{signup, login, health_check}, models::user::RegisterPayload, routes::user};

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
		"password": "securepassword"
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
async fn test_login(pool: PgPool) {
	let app = build_app(pool.clone());

	let payload = json!({
		"email": "test@example.com",
		"password": "securepassword"
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

	//let user_data = axum::body::to_bytes(login_response.into_body(), 1024 * 1024).await.unwrap();
	//let user_json: serde_json::Value = serde_json::from_slice(&user_data).unwrap();
	let token_json: serde_json::Value = serde_json::from_slice(
		&axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
		.await.unwrap()
	).unwrap();
	let token = token_json.get("token").expect("token in login response");
	
	assert!(token.is_string(), "Token should not be empty");

}

#[sqlx::test]
async fn test_login_invalid_password(pool: PgPool) {
	let app = build_app(pool.clone());

	let correct = json!({
		"email": "test@example.com",
		"password": "correctpassword"
	});

	let wrong = json!({
		"email": "test@example.com",
		"password": "wrongpassword"
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
		"password": "securepassword"
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

	let token_json: serde_json::Value = serde_json::from_slice(
		&axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
		.await.unwrap()
	).unwrap();
	let token = token_json.get("token").expect("token in login response").as_str().unwrap();

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

	let me_data = axum::body::to_bytes(me_response.into_body(), 1024 * 1024).await.unwrap();
	let me_json: serde_json::Value = serde_json::from_slice(&me_data).unwrap();

	assert_eq!(me_json.get("email").unwrap().as_str().unwrap(), payload["email"].as_str().unwrap());
	assert!(me_json.get("id").is_some());
	assert!(me_json.get("created_at").is_some());
}

#[sqlx::test]
async fn test_change_password(pool: PgPool) {
	let app = build_app(pool.clone());

	let payload = json!({
		"email": "test@example.com",
		"password": "securepassword"
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

	let token_json: serde_json::Value = serde_json::from_slice(
		&axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
		.await.unwrap()
	).unwrap();
	let token = token_json.get("token").expect("token in login response").as_str().unwrap();
	let change_password_payload = json!({
		"old_password": "securepassword",
		"new_password": "newsecurepassword"
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

	assert_eq!(change_password_response.status(), StatusCode::OK);

	let login_with_new_password_response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/login")
				.header("Content-Type", "application/json")
				.body(Body::from(json!({
					"email": "test@example.com",
					"password": "newsecurepassword"
				}).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(login_with_new_password_response.status(), StatusCode::OK);

	let new_token_json: serde_json::Value = serde_json::from_slice(
		&axum::body::to_bytes(login_with_new_password_response.into_body(), 1024 * 1024)
		.await.unwrap()
	).unwrap();
	let new_token = new_token_json.get("token").expect("token in login response").as_str().unwrap();

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
					"password": "securepassword"
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
					"old_password": "wrongpassword",
					"new_password": "anothernewpassword"
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
					"old_password": "newsecurepassword",
					"new_password": ""
				}).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(change_password_error_response.status(), StatusCode::BAD_REQUEST);

}