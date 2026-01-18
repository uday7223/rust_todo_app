use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}
