use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTodoReq {
    pub title: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTodoReq {
    pub title: String,
    pub completed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateTodoResponse {
    pub id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct TodoResponse {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: NaiveDateTime,
}
