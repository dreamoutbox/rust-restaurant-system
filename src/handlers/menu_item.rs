use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, require_role},
    models::menu_item::{CreateMenuItemReq, MenuItem, MenuItemWithCategory, UpdateMenuItemReq},
    models::user::UserRole,
};

use super::auth::AppState;

pub async fn list_menu(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            m.id,
            m.category_id,
            c.name as category_name,
            m.name,
            m.description,
            m.price as "price!: i64",
            m.image_path,
            m.is_available,
            m.sort_order
        FROM menu_items m
        JOIN categories c ON c.id = m.category_id
        WHERE m.is_available = true AND m.deleted_at IS NULL AND c.is_active = true
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
            m.price as "price!: i64",
            m.image_path,
            m.is_available,
            m.sort_order
        FROM menu_items m
        JOIN categories c ON c.id = m.category_id
        WHERE m.deleted_at IS NULL
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
        RETURNING id, category_id, name, description, price, image_path, is_available, sort_order, deleted_at, created_at, updated_at
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

    let item = sqlx::query_as::<_, MenuItem>(
        r#"
        UPDATE menu_items
        SET
            category_id = COALESCE($1, category_id),
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            price = COALESCE($4, price),
            image_path = COALESCE($5, image_path),
            is_available = COALESCE($6, is_available),
            sort_order = COALESCE($7, sort_order),
            updated_at = now()
        WHERE id = $8 AND deleted_at IS NULL
        RETURNING id, category_id, name, description, price, image_path, is_available, sort_order, deleted_at, created_at, updated_at
        "#,
    )
    .bind(payload.category_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(&payload.image_path)
    .bind(payload.is_available)
    .bind(payload.sort_order)
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Menu item not found".to_string()))?;

    Ok(Json(item))
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

    sqlx::query("UPDATE menu_items SET deleted_at = now(), updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(
        json!({ "message": "Menu item deleted successfully" }),
    ))
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
        let ext = file_name.rsplit('.').next().unwrap_or("png").to_lowercase();

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

    let image_path =
        relative_file_path.ok_or_else(|| AppError::BadRequest("No file uploaded".to_string()))?;

    sqlx::query("UPDATE menu_items SET image_path = $1, updated_at = now() WHERE id = $2")
        .bind(&image_path)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "image_path": image_path })))
}
