use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use qrcode::render::svg;
use qrcode::QrCode;
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{require_role, AuthUser},
    models::table::{CreateTableReq, Table, TableWithStatus, UpdateTableReq},
    models::user::UserRole,
};

use super::auth::AppState;

pub async fn list_tables(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    // Accessible by all authenticated staff
    require_role(
        &claims,
        &[
            UserRole::Admin,
            UserRole::Cashier,
            UserRole::Kitchen,
            UserRole::Waiter,
        ],
    )?;

    let rows = sqlx::query!(
        r#"
        SELECT
            t.id,
            t.table_number,
            t.name,
            t.capacity,
            t.is_active,
            o.id as "active_order_id?",
            o.session_token as "active_session_token?",
            o.status as "order_status?"
        FROM tables t
        LEFT JOIN orders o ON o.table_id = t.id AND o.status IN ('open', 'checkout_pending')
        WHERE t.is_active = true
        ORDER BY t.table_number ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let tables: Vec<TableWithStatus> = rows
        .into_iter()
        .map(|r| TableWithStatus {
            id: r.id,
            table_number: r.table_number,
            name: r.name,
            capacity: r.capacity,
            is_active: r.is_active,
            active_order_id: r.active_order_id,
            active_session_token: r.active_session_token,
            order_status: r.order_status,
        })
        .collect();

    Ok(Json(tables))
}

pub async fn create_table(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateTableReq>,
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    let table = sqlx::query_as::<_, Table>(
        r#"
        INSERT INTO tables (table_number, name, capacity)
        VALUES ($1, $2, $3)
        RETURNING id, table_number, name, capacity, is_active, created_at, updated_at
        "#,
    )
    .bind(payload.table_number)
    .bind(&payload.name)
    .bind(payload.capacity)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.is_unique_violation() {
                return AppError::BadRequest("Table number already exists".to_string());
            }
        }
        AppError::from(e)
    })?;

    Ok((StatusCode::CREATED, Json(table)))
}

pub async fn update_table(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTableReq>,
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    let mut existing = sqlx::query_as::<_, Table>(
        "SELECT id, table_number, name, capacity, is_active, created_at, updated_at FROM tables WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if let Some(table_number) = payload.table_number {
        existing.table_number = table_number;
    }
    if let Some(name) = payload.name {
        existing.name = name;
    }
    if let Some(capacity) = payload.capacity {
        existing.capacity = Some(capacity);
    }
    if let Some(is_active) = payload.is_active {
        existing.is_active = is_active;
    }

    let updated = sqlx::query_as::<_, Table>(
        r#"
        UPDATE tables
        SET table_number = $1, name = $2, capacity = $3, is_active = $4, updated_at = now()
        WHERE id = $5
        RETURNING id, table_number, name, capacity, is_active, created_at, updated_at
        "#,
    )
    .bind(existing.table_number)
    .bind(&existing.name)
    .bind(existing.capacity)
    .bind(existing.is_active)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_table(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin])?;

    sqlx::query("UPDATE tables SET is_active = false, updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "message": "Table deactivated successfully" })))
}

pub async fn open_table(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(table_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    // Check if table is active
    let table = sqlx::query_as::<_, Table>(
        "SELECT id, table_number, name, capacity, is_active, created_at, updated_at FROM tables WHERE id = $1 AND is_active = true",
    )
    .bind(table_id)
    .fetch_one(&state.db)
    .await?;

    // Check if table already has an open order
    let existing_order = sqlx::query!(
        "SELECT id FROM orders WHERE table_id = $1 AND status IN ('open', 'checkout_pending')",
        table_id
    )
    .fetch_optional(&state.db)
    .await?;

    if existing_order.is_some() {
        return Err(AppError::BadRequest(
            "Table already has an open order session".to_string(),
        ));
    }

    // Generate session token (UUID v4 hex string)
    let session_token = Uuid::new_v4().to_string().replace('-', "");

    let order = sqlx::query!(
        r#"
        INSERT INTO orders (table_id, session_token, status, opened_by)
        VALUES ($1, $2, 'open', $3)
        RETURNING id, session_token, status, opened_at
        "#,
        table_id,
        session_token,
        claims.sub
    )
    .fetch_one(&state.db)
    .await?;

    let qr_url = format!("{}/order/{}", state.config.base_url, order.session_token);

    Ok(Json(json!({
        "order_id": order.id,
        "table_id": table.id,
        "table_number": table.table_number,
        "session_token": order.session_token,
        "status": order.status,
        "qr_url": qr_url,
        "opened_at": order.opened_at
    })))
}

pub async fn close_table(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(table_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    let active_order = sqlx::query!(
        "SELECT id, status FROM orders WHERE table_id = $1 AND status IN ('open', 'checkout_pending', 'paid')",
        table_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("No active order session for this table".to_string()))?;

    sqlx::query!(
        "UPDATE orders SET status = 'closed', closed_by = $1, closed_at = now() WHERE id = $2",
        claims.sub,
        active_order.id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "message": "Table closed successfully" })))
}

pub async fn get_table_qr(
    State(state): State<AppState>,
    Path(table_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let order = sqlx::query!(
        "SELECT session_token FROM orders WHERE table_id = $1 AND status IN ('open', 'checkout_pending')",
        table_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("No open session found for this table".to_string()))?;

    let qr_url = format!("{}/order/{}", state.config.base_url, order.session_token);

    let code = QrCode::new(qr_url.as_bytes())
        .map_err(|e| AppError::Internal(format!("Failed to generate QR code: {}", e)))?;

    let svg_content = code
        .render::<svg::Color>()
        .min_dimensions(300, 300)
        .dark_color(svg::Color("#1a1a1a"))
        .light_color(svg::Color("#ffffff"))
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/svg+xml".parse().unwrap());

    Ok((headers, svg_content))
}
