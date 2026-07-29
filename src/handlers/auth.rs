use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;

use crate::{
    config::Config,
    error::AppError,
    middleware::auth::{generate_jwt, AuthUser, COOKIE_NAME},
    models::user::{LoginReq, User, UserResponse},
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub sse: crate::sse::SseBroadcaster,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginReq>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, display_name, role, is_active, created_at, updated_at FROM users WHERE username = $1 AND is_active = true",
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Auth("Invalid username or password".to_string()))?;

    // Verify password with argon2
    let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Auth("Invalid username or password".to_string()))?;

    argon2::Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Auth("Invalid username or password".to_string()))?;

    let token = generate_jwt(user.id, &user.username, &user.role, &state.config)?;

    let cookie_header = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        COOKIE_NAME,
        token,
        state.config.jwt_expiry_hours * 3600
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie_header.parse().unwrap());

    let user_resp = UserResponse::from(user);

    Ok((
        StatusCode::OK,
        headers,
        Json(json!({
            "user": user_resp,
            "token": token
        })),
    ))
}

pub async fn logout() -> impl IntoResponse {
    let cookie_header = format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        COOKIE_NAME
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie_header.parse().unwrap());

    (headers, Json(json!({ "message": "Logged out successfully" })))
}

pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, display_name, role, is_active, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(UserResponse::from(user)))
}
