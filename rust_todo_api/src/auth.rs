use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::encode_b64(b"randomsalt").expect("invalid salt");
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn generate_jwt(user_id: Uuid) -> String {
    let exp = chrono::Utc::now().timestamp() as usize + 60 * 60 * 24; // 1 day
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            std::env::var("JWT_SECRET")
                .expect("JWT_SECRET not set")
                .as_bytes(),
        ),
    )
    .expect("failed to generate jwt")
}

pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Response {
    let token = match extract_bearer_token(req.headers().get("authorization")) {
        Some(token) => token,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let validation = Validation::default();
    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id = match Uuid::parse_str(&token_data.claims.sub) {
        Ok(id) => id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    req.extensions_mut().insert(user_id);
    next.run(req).await
}

fn extract_bearer_token(header_value: Option<&axum::http::HeaderValue>) -> Option<String> {
    let value = header_value?.to_str().ok()?;
    let mut parts = value.splitn(2, ' ');
    let scheme = parts.next()?;
    let token = parts.next()?;
    if scheme.eq_ignore_ascii_case("bearer") {
        Some(token.to_string())
    } else {
        None
    }
}
