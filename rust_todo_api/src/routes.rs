use axum::{
    extract::{Path, State},
    Extension,
    Json,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::auth::{generate_jwt, hash_password, verify_password};
use crate::error::AppError;
use crate::models::{
    CreateTodoReq, CreateTodoResponse, ErrorResponse, LoginReq, MessageResponse, RegisterReq,
    TodoResponse, TokenResponse, UpdateTodoReq,
};

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterReq,
    tag = "auth",
    responses(
        (status = 200, description = "User registered", body = MessageResponse),
        (status = 400, description = "User exists", body = ErrorResponse)
    )
)]
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterReq>,
) -> Result<Json<MessageResponse>, AppError> {
    validate_email(&payload.email)?;
    validate_password(&payload.password)?;

    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&payload.password);

    let res = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(payload.email)
        .bind(password_hash)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => Ok(Json(MessageResponse {
            message: "User registered".to_string(),
        })),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(AppError::BadRequest("User exists".to_string()))
            } else {
                Err(AppError::from(err))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginReq,
    tag = "auth",
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<TokenResponse>, AppError> {
    validate_email(&payload.email)?;
    validate_password(&payload.password)?;

    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(payload.email)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::from)?;

    if let Some(user) = row {
        let user_id: Uuid = user.try_get("id").map_err(|_| AppError::Internal("Invalid user id".to_string()))?;
        let password_hash: String =
            user.try_get("password_hash").map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;
        if verify_password(&password_hash, &payload.password) {
            let token = generate_jwt(user_id);
            return Ok(Json(TokenResponse { token }));
        }
    }

    Err(AppError::Unauthorized("Invalid credentials".to_string()))
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodoReq,
    tag = "todos",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Todo created", body = CreateTodoResponse),
        (status = 400, description = "Invalid title", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn create_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTodoReq>,
) -> Result<Json<CreateTodoResponse>, AppError> {
    validate_title(&payload.title)?;

    let id = Uuid::new_v4();

    sqlx::query("INSERT INTO todos (id, user_id, title) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(user_id)
        .bind(payload.title)
        .execute(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(CreateTodoResponse { id }))
}

#[utoipa::path(
    get,
    path = "/todos",
    tag = "todos",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of todos", body = [TodoResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn list_todos(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<TodoResponse>>, AppError> {
    let todos = sqlx::query("SELECT id, title, completed, created_at FROM todos WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    let mut response = Vec::with_capacity(todos.len());
    for todo in todos {
        response.push(TodoResponse {
            id: todo
                .try_get("id")
                .map_err(|_| AppError::Internal("Invalid todo id".to_string()))?,
            title: todo
                .try_get("title")
                .map_err(|_| AppError::Internal("Invalid title".to_string()))?,
            completed: todo
                .try_get("completed")
                .map_err(|_| AppError::Internal("Invalid completed flag".to_string()))?,
            created_at: todo
                .try_get("created_at")
                .map_err(|_| AppError::Internal("Invalid timestamp".to_string()))?,
        });
    }

    Ok(Json(response))
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
        (status = 400, description = "Invalid title", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Todo not found", body = ErrorResponse)
    )
)]
pub async fn update_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(todo_id): Path<Uuid>,
    Json(payload): Json<UpdateTodoReq>,
) -> Result<Json<TodoResponse>, AppError> {
    validate_title(&payload.title)?;

    let row = sqlx::query(
        "UPDATE todos SET title = $1, completed = $2 WHERE id = $3 AND user_id = $4 RETURNING id, title, completed, created_at",
    )
    .bind(payload.title)
    .bind(payload.completed)
    .bind(todo_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?;

    let todo = match row {
        Some(row) => TodoResponse {
            id: row.try_get("id").map_err(|_| AppError::Internal("Invalid todo id".to_string()))?,
            title: row.try_get("title").map_err(|_| AppError::Internal("Invalid title".to_string()))?,
            completed: row.try_get("completed").map_err(|_| AppError::Internal("Invalid completed flag".to_string()))?,
            created_at: row.try_get("created_at").map_err(|_| AppError::Internal("Invalid timestamp".to_string()))?,
        },
        None => return Err(AppError::NotFound("Todo not found".to_string())),
    };

    Ok(Json(todo))
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Todo not found", body = ErrorResponse)
    )
)]
pub async fn delete_todo(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(todo_id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1 AND user_id = $2")
        .bind(todo_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(AppError::from)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Todo not found".to_string()));
    }

    Ok(Json(MessageResponse {
        message: "Todo deleted".to_string(),
    }))
}

fn validate_email(email: &str) -> Result<(), AppError> {
    let email = email.trim();
    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }
    match email.split_once('@') {
        Some((_, domain)) if domain.contains('.') => Ok(()),
        _ => Err(AppError::BadRequest("Invalid email format".to_string())),
    }
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.trim().len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<(), AppError> {
    if title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }
    Ok(())
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map(|code| code == "23505").unwrap_or(false),
        _ => false,
    }
}
