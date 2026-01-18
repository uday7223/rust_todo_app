mod auth;
mod db;
mod models;
mod routes;

use axum::{
    middleware,
    routing::post,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::auth::auth_middleware;
use crate::db::connect_db;
use crate::routes::{create_todo, list_todos, login, register};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = connect_db().await;

    let protected_routes = Router::new()
        .route("/", post(create_todo).get(list_todos))
        .route_layer(middleware::from_fn(auth_middleware));

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .nest("/todos", protected_routes)
        .with_state(pool)
        .layer(CorsLayer::permissive());

    println!("Server running on http://localhost:3002");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("failed to bind server");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
