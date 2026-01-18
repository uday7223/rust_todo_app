mod auth;
mod db;
mod models;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::db::connect_db;
use crate::routes::{create_todo, list_todos, login, register};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = connect_db().await;

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/todos", post(create_todo).get(list_todos))
        .with_state(pool)
        .layer(CorsLayer::permissive());

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind server");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
