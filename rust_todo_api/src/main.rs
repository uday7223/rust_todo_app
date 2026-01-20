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
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::auth_middleware;
use crate::db::connect_db;
use crate::models::{
    CreateTodoReq, CreateTodoResponse, LoginReq, MessageResponse, RegisterReq, TodoResponse,
    TokenResponse,
};
use crate::routes::{create_todo, list_todos, login, register};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let scheme = SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            );
            components.add_security_scheme("bearer_auth", scheme);
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::register,
        routes::login,
        routes::create_todo,
        routes::list_todos
    ),
    components(
        schemas(
            RegisterReq,
            LoginReq,
            CreateTodoReq,
            MessageResponse,
            TokenResponse,
            CreateTodoResponse,
            TodoResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "todos", description = "Todo endpoints")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

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
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(pool)
        .layer(CorsLayer::permissive());

    println!("Server running on http://localhost:3002");
    println!("Swagger UI at http://localhost:3002/docs");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("failed to bind server");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
