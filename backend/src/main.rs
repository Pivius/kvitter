mod util;
mod tests;
mod models;
mod routes;
mod auth;

use std::env;
use tracing::{info, Level};
use tracing_subscriber;
use axum::{Router, routing::post, routing::get, routing::put, routing::delete};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
	tracing_subscriber::fmt().with_max_level(Level::INFO).init();
	dotenv().ok();

	let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let pool = PgPoolOptions::new()
		//.max_connections(5)
		.connect(&db_url)
		.await?;
	let app = Router::new()
		.without_v07_checks()
		.route("/auth/signup", post(routes::auth::signup))
		.route("/health", get(routes::auth::health_check))
		.route("/user/:id", get(routes::user::get_user_by_uuid).put(routes::user::update_user).delete(routes::user::delete_user))
		.layer(CorsLayer::permissive())
		.with_state(pool);
	let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();

	info!("Server is running on http://0.0.0.0:5000");

	axum::serve(listener, app)
		.await
		.unwrap();

	Ok(())
}