use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{require_role, AuthUser},
    models::user::{CreateUserReq, UpdateUserReq, User, UserResponse, UserRole},
};

use super::auth::AppState;

pub async fn list_users(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, display_name, role, is_active, created_at, updated_at FROM users ORDER BY username ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(responses))
}

pub async fn create_user(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateUserReq>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    // Validate role
    let role: UserRole = payload.role.parse().map_err(AppError::BadRequest)?;

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing error: {}", e)))?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, password_hash, display_name, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, password_hash, display_name, role, is_active, created_at, updated_at
        "#,
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(&payload.display_name)
    .bind(role.to_string())
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.is_unique_violation() {
                return AppError::BadRequest("Username already exists".to_string());
            }
        }
        AppError::from(e)
    })?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

pub async fn update_user(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserReq>,
) -> Result<Json<UserResponse>, AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    let mut existing = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, display_name, role, is_active, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if let Some(display_name) = payload.display_name {
        existing.display_name = display_name;
    }

    if let Some(role_str) = payload.role {
        let role: UserRole = role_str.parse().map_err(AppError::BadRequest)?;
        existing.role = role.to_string();
    }

    if let Some(is_active) = payload.is_active {
        existing.is_active = is_active;
    }

    if let Some(new_password) = payload.password {
        if !new_password.is_empty() {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            existing.password_hash = argon2
                .hash_password(new_password.as_bytes(), &salt)
                .map_err(|e| AppError::Internal(format!("Password hashing error: {}", e)))?
                .to_string();
        }
    }

    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET display_name = $1, role = $2, is_active = $3, password_hash = $4, updated_at = now()
        WHERE id = $5
        RETURNING id, username, password_hash, display_name, role, is_active, created_at, updated_at
        "#,
    )
    .bind(&existing.display_name)
    .bind(&existing.role)
    .bind(existing.is_active)
    .bind(&existing.password_hash)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(UserResponse::from(updated)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    // Soft delete (deactivate)
    sqlx::query("UPDATE users SET is_active = false, updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "message": "User deactivated successfully" })))
}
