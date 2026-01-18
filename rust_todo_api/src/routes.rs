use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{generate_jwt, hash_password, verify_password};
use crate::models::{CreateTodoReq, LoginReq, MessageResponse, RegisterReq, TodoResponse};

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterReq>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&payload.password);

    let res = sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        payload.email,
        password_hash
    )
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Json(MessageResponse {
            message: "User registered".to_string(),
        })
        .into_response(),
        Err(_) => (StatusCode::BAD_REQUEST, "User exists").into_response(),
    }
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginReq>,
) -> impl IntoResponse {
    let row = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    if let Some(user) = row {
        if verify_password(&user.password_hash, &payload.password) {
            let token = generate_jwt(user.id);
            return Json(json!({ "token": token })).into_response();
        }
    }

    (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
}

pub async fn create_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTodoReq>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO todos (id, user_id, title) VALUES ($1, $2, $3)",
        id,
        user_id,
        payload.title
    )
    .execute(&pool)
    .await
    .unwrap();

    Json(json!({ "id": id }))
}

pub async fn list_todos(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> impl IntoResponse {
    let todos = sqlx::query!(
        "SELECT id, title, completed, created_at FROM todos WHERE user_id = $1",
        user_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let response: Vec<TodoResponse> = todos
        .into_iter()
        .map(|todo| TodoResponse {
            id: todo.id,
            title: todo.title,
            completed: todo.completed,
            created_at: todo.created_at,
        })
        .collect();

    Json(response)
}
