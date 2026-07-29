use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{require_role, AuthUser},
    models::category::{Category, CreateCategoryReq, UpdateCategoryReq},
    models::user::UserRole,
};

use super::auth::AppState;

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT id, name, sort_order, is_active, created_at, updated_at FROM categories WHERE is_active = true ORDER BY sort_order ASC, name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(categories))
}

pub async fn create_category(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateCategoryReq>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    let sort_order = payload.sort_order.unwrap_or(0);

    let category = sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (name, sort_order)
        VALUES ($1, $2)
        RETURNING id, name, sort_order, is_active, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(sort_order)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.is_unique_violation() {
                return AppError::BadRequest("Category name already exists".to_string());
            }
        }
        AppError::from(e)
    })?;

    Ok((StatusCode::CREATED, Json(category)))
}

pub async fn update_category(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryReq>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    let mut existing = sqlx::query_as::<_, Category>(
        "SELECT id, name, sort_order, is_active, created_at, updated_at FROM categories WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if let Some(name) = payload.name {
        existing.name = name;
    }
    if let Some(sort_order) = payload.sort_order {
        existing.sort_order = sort_order;
    }
    if let Some(is_active) = payload.is_active {
        existing.is_active = is_active;
    }

    let updated = sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories
        SET name = $1, sort_order = $2, is_active = $3, updated_at = now()
        WHERE id = $4
        RETURNING id, name, sort_order, is_active, created_at, updated_at
        "#,
    )
    .bind(&existing.name)
    .bind(existing.sort_order)
    .bind(existing.is_active)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_category(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    sqlx::query("UPDATE categories SET is_active = false, updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "message": "Category deactivated successfully" })))
}
