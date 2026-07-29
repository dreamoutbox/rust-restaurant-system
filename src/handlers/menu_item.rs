use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{require_role, AuthUser},
    models::menu_item::{CreateMenuItemReq, MenuItem, MenuItemWithCategory, UpdateMenuItemReq},
    models::user::UserRole,
};

use super::auth::AppState;

pub async fn list_menu(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            m.id,
            m.category_id,
            c.name as category_name,
            m.name,
            m.description,
            m.price,
            m.image_path,
            m.is_available,
            m.sort_order
        FROM menu_items m
        JOIN categories c ON c.id = m.category_id
        WHERE m.is_available = true AND c.is_active = true
        ORDER BY c.sort_order ASC, m.sort_order ASC, m.name ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let items: Vec<MenuItemWithCategory> = rows
        .into_iter()
        .map(|r| MenuItemWithCategory {
            id: r.id,
            category_id: r.category_id,
            category_name: r.category_name,
            name: r.name,
            description: r.description,
            price: r.price,
            image_path: r.image_path,
            is_available: r.is_available,
            sort_order: r.sort_order,
        })
        .collect();

    Ok(Json(items))
}

pub async fn list_all_menu_items(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    let rows = sqlx::query!(
        r#"
        SELECT
            m.id,
            m.category_id,
            c.name as category_name,
            m.name,
            m.description,
            m.price,
            m.image_path,
            m.is_available,
            m.sort_order
        FROM menu_items m
        JOIN categories c ON c.id = m.category_id
        ORDER BY c.sort_order ASC, m.sort_order ASC, m.name ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let items: Vec<MenuItemWithCategory> = rows
        .into_iter()
        .map(|r| MenuItemWithCategory {
            id: r.id,
            category_id: r.category_id,
            category_name: r.category_name,
            name: r.name,
            description: r.description,
            price: r.price,
            image_path: r.image_path,
            is_available: r.is_available,
            sort_order: r.sort_order,
        })
        .collect();

    Ok(Json(items))
}

pub async fn create_menu_item(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateMenuItemReq>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    let sort_order = payload.sort_order.unwrap_or(0);

    let item = sqlx::query_as::<_, MenuItem>(
        r#"
        INSERT INTO menu_items (category_id, name, description, price, image_path, sort_order)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, category_id, name, description, price, image_path, is_available, sort_order, created_at, updated_at
        "#,
    )
    .bind(payload.category_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(&payload.image_path)
    .bind(sort_order)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update_menu_item(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMenuItemReq>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    let mut existing = sqlx::query_as::<_, MenuItem>(
        "SELECT id, category_id, name, description, price, image_path, is_available, sort_order, created_at, updated_at FROM menu_items WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if let Some(category_id) = payload.category_id {
        existing.category_id = category_id;
    }
    if let Some(name) = payload.name {
        existing.name = name;
    }
    if let Some(description) = payload.description {
        existing.description = Some(description);
    }
    if let Some(price) = payload.price {
        existing.price = price;
    }
    if let Some(image_path) = payload.image_path {
        existing.image_path = Some(image_path);
    }
    if let Some(is_available) = payload.is_available {
        existing.is_available = is_available;
    }
    if let Some(sort_order) = payload.sort_order {
        existing.sort_order = sort_order;
    }

    let updated = sqlx::query_as::<_, MenuItem>(
        r#"
        UPDATE menu_items
        SET category_id = $1, name = $2, description = $3, price = $4, image_path = $5, is_available = $6, sort_order = $7, updated_at = now()
        WHERE id = $8
        RETURNING id, category_id, name, description, price, image_path, is_available, sort_order, created_at, updated_at
        "#,
    )
    .bind(existing.category_id)
    .bind(&existing.name)
    .bind(&existing.description)
    .bind(existing.price)
    .bind(&existing.image_path)
    .bind(existing.is_available)
    .bind(existing.sort_order)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_menu_item(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    sqlx::query("UPDATE menu_items SET is_available = false, updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "message": "Menu item deactivated successfully" })))
}

pub async fn upload_menu_item_image(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Cashier, UserRole::Kitchen],
    )?;

    let upload_dir = PathBuf::from(&state.config.upload_dir);
    if !upload_dir.exists() {
        fs::create_dir_all(&upload_dir)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;
    }

    let mut relative_file_path: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart parse error: {}", e)))?
    {
        let file_name = field.file_name().unwrap_or("image.png").to_string();
        let ext = file_name
            .rsplit('.')
            .next()
            .unwrap_or("png")
            .to_lowercase();

        let new_file_name = format!("{}_{}.{}", id, Uuid::new_v4().simple(), ext);
        let dest_path = upload_dir.join(&new_file_name);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read field bytes: {}", e)))?;

        fs::write(&dest_path, data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to save image: {}", e)))?;

        relative_file_path = Some(format!("/uploads/{}", new_file_name));
        break;
    }

    let image_path = relative_file_path.ok_or_else(|| AppError::BadRequest("No file uploaded".to_string()))?;

    sqlx::query("UPDATE menu_items SET image_path = $1, updated_at = now() WHERE id = $2")
        .bind(&image_path)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "image_path": image_path })))
}
