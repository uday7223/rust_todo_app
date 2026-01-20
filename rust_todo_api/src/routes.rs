use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::auth::{generate_jwt, hash_password, verify_password};
use crate::models::{
    CreateTodoReq, CreateTodoResponse, LoginReq, MessageResponse, RegisterReq, TodoResponse,
    TokenResponse, UpdateTodoReq,
};

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterReq,
    tag = "auth",
    responses(
        (status = 200, description = "User registered", body = MessageResponse),
        (status = 400, description = "User exists")
    )
)]
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterReq>,
) -> impl IntoResponse {
    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&payload.password);

    let res = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(payload.email)
        .bind(password_hash)
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

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginReq,
    tag = "auth",
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginReq>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(payload.email)
        .fetch_optional(&pool)
        .await
        .unwrap();

    if let Some(user) = row {
        let user_id: Uuid = user.try_get("id").unwrap();
        let password_hash: String = user.try_get("password_hash").unwrap();
        if verify_password(&password_hash, &payload.password) {
            let token = generate_jwt(user_id);
            return Json(TokenResponse { token }).into_response();
        }
    }

    (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodoReq,
    tag = "todos",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Todo created", body = CreateTodoResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTodoReq>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();

    sqlx::query("INSERT INTO todos (id, user_id, title) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(user_id)
        .bind(payload.title)
        .execute(&pool)
        .await
        .unwrap();

    Json(CreateTodoResponse { id })
}

#[utoipa::path(
    get,
    path = "/todos",
    tag = "todos",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of todos", body = [TodoResponse]),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_todos(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> impl IntoResponse {
    let todos = sqlx::query("SELECT id, title, completed, created_at FROM todos WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap();

    let response: Vec<TodoResponse> = todos
        .into_iter()
        .map(|todo| TodoResponse {
            id: todo.try_get("id").unwrap(),
            title: todo.try_get("title").unwrap(),
            completed: todo.try_get("completed").unwrap(),
            created_at: todo.try_get("created_at").unwrap(),
        })
        .collect();

    Json(response)
}

#[utoipa::path(
    put,
    path = "/todos/{id}",
    request_body = UpdateTodoReq,
    params(
        ("id" = Uuid, Path, description = "Todo id")
    ),
    tag = "todos",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Todo updated", body = TodoResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Todo not found")
    )
)]
pub async fn update_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(todo_id): Path<Uuid>,
    Json(payload): Json<UpdateTodoReq>,
) -> impl IntoResponse {
    let row = sqlx::query(
        "UPDATE todos SET title = $1, completed = $2 WHERE id = $3 AND user_id = $4 RETURNING id, title, completed, created_at",
    )
    .bind(payload.title)
    .bind(payload.completed)
    .bind(todo_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    let todo = match row {
        Some(row) => TodoResponse {
            id: row.try_get("id").unwrap(),
            title: row.try_get("title").unwrap(),
            completed: row.try_get("completed").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
        },
        None => return (StatusCode::NOT_FOUND, "Todo not found").into_response(),
    };

    Json(todo).into_response()
}

#[utoipa::path(
    delete,
    path = "/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo id")
    ),
    tag = "todos",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Todo deleted", body = MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Todo not found")
    )
)]
pub async fn delete_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(todo_id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1 AND user_id = $2")
        .bind(todo_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    if result.rows_affected() == 0 {
        return (StatusCode::NOT_FOUND, "Todo not found").into_response();
    }

    Json(MessageResponse {
        message: "Todo deleted".to_string(),
    })
    .into_response()
}
