use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, require_role},
    models::menu_item::MenuItemWithCategory,
    models::order::OrderDetail,
    models::order_item::{OrderItemDetail, SubmitOrderItemsReq},
    models::user::UserRole,
    sse::SseEvent,
};

use super::auth::AppState;

#[derive(Debug, Deserialize)]
pub struct OrderFilterQuery {
    pub status: Option<String>,
    pub table_id: Option<Uuid>,
}

// Public: Customer gets menu for session token
pub async fn get_customer_session_menu(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let order = sqlx::query!(
        r#"
        SELECT o.id, o.status, t.table_number, t.name as table_name
        FROM orders o
        JOIN tables t ON t.id = o.table_id
        WHERE o.session_token = $1 AND o.status IN ('open', 'checkout_pending')
        "#,
        token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Invalid or expired table session token".to_string()))?;

    let menu_rows = sqlx::query!(
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

    let items: Vec<MenuItemWithCategory> = menu_rows
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

    Ok(Json(json!({
        "order_id": order.id,
        "table_number": order.table_number,
        "table_name": order.table_name,
        "status": order.status,
        "menu": items
    })))
}

// Public: Customer submits items to their session order
pub async fn submit_customer_order_items(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<SubmitOrderItemsReq>,
) -> Result<impl IntoResponse, AppError> {
    if payload.items.is_empty() {
        return Err(AppError::BadRequest(
            "Cannot submit empty order".to_string(),
        ));
    }

    let order = sqlx::query!(
        r#"
        SELECT o.id, o.status, t.table_number
        FROM orders o
        JOIN tables t ON t.id = o.table_id
        WHERE o.session_token = $1 AND o.status = 'open'
        "#,
        token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        AppError::NotFound("Active order session not found for this token".to_string())
    })?;

    let mut created_item_ids = Vec::new();

    for input in &payload.items {
        if input.quantity <= 0 {
            continue;
        }

        // Fetch current price of item
        let menu_item = sqlx::query!(
            "SELECT price FROM menu_items WHERE id = $1 AND is_available = true",
            input.menu_item_id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest(format!("Menu item {} unavailable", input.menu_item_id))
        })?;

        let created_item = sqlx::query!(
            r#"
            INSERT INTO order_items (order_id, menu_item_id, quantity, unit_price, note, status)
            VALUES ($1, $2, $3, $4, $5, 'pending')
            RETURNING id
            "#,
            order.id,
            input.menu_item_id,
            input.quantity,
            menu_item.price,
            input.note
        )
        .fetch_one(&state.db)
        .await?;

        created_item_ids.push(created_item.id);
    }

    // Emit SSE event to kitchen & cashier
    state.sse.send(SseEvent::NewOrderItems {
        table_number: order.table_number,
        order_id: order.id,
        items_count: created_item_ids.len(),
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Order submitted successfully",
            "item_count": created_item_ids.len()
        })),
    ))
}

// Public: Customer checks status of their table order
pub async fn get_customer_order_status(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let order = sqlx::query!(
        r#"
        SELECT o.id, o.table_id, o.status, o.total_amount, t.table_number, t.name as table_name
        FROM orders o
        JOIN tables t ON t.id = o.table_id
        WHERE o.session_token = $1
        "#,
        token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Session order not found".to_string()))?;

    let item_rows = sqlx::query!(
        r#"
        SELECT
            oi.id,
            oi.order_id,
            t.table_number,
            oi.menu_item_id,
            m.name as menu_item_name,
            oi.quantity,
            oi.unit_price,
            oi.note,
            oi.status,
            oi.created_at
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        JOIN tables t ON t.id = o.table_id
        JOIN menu_items m ON m.id = oi.menu_item_id
        WHERE oi.order_id = $1
        ORDER BY oi.created_at ASC
        "#,
        order.id
    )
    .fetch_all(&state.db)
    .await?;

    let items: Vec<OrderItemDetail> = item_rows
        .into_iter()
        .map(|r| OrderItemDetail {
            id: r.id,
            order_id: r.order_id,
            table_number: r.table_number,
            menu_item_id: r.menu_item_id,
            menu_item_name: r.menu_item_name,
            quantity: r.quantity,
            unit_price: r.unit_price,
            note: r.note,
            status: r.status,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(json!({
        "order_id": order.id,
        "table_number": order.table_number,
        "table_name": order.table_name,
        "status": order.status,
        "total_amount": order.total_amount,
        "items": items
    })))
}

// Staff: List orders
pub async fn list_orders(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<OrderFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[
            UserRole::Admin,
            UserRole::Cashier,
            UserRole::Kitchen,
            UserRole::Waiter,
        ],
    )?;

    let mut sql = String::from(
        r#"
        SELECT
            o.id,
            o.table_id,
            t.table_number,
            t.name as table_name,
            o.session_token,
            o.status,
            o.total_amount,
            o.payment_method,
            o.stripe_session_id,
            o.opened_at,
            o.closed_at
        FROM orders o
        JOIN tables t ON t.id = o.table_id
        WHERE 1=1
        "#,
    );

    if let Some(ref status) = query.status {
        sql.push_str(&format!(" AND o.status = '{}'", status.replace('\'', "")));
    } else {
        // Default list active orders
        sql.push_str(" AND o.status IN ('open', 'checkout_pending', 'paid')");
    }

    if let Some(table_id) = query.table_id {
        sql.push_str(&format!(" AND o.table_id = '{}'", table_id));
    }

    sql.push_str(" ORDER BY o.opened_at DESC");

    let rows = sqlx::query(&sql).fetch_all(&state.db).await?;

    use sqlx::Row;
    let mut orders = Vec::new();

    for row in rows {
        let order_id: Uuid = row.get("id");
        let table_id: Uuid = row.get("table_id");
        let table_number: i32 = row.get("table_number");
        let table_name: String = row.get("table_name");
        let session_token: String = row.get("session_token");
        let status: String = row.get("status");
        let total_amount: rust_decimal::Decimal = row.get("total_amount");
        let payment_method: Option<String> = row.get("payment_method");
        let stripe_session_id: Option<String> = row.get("stripe_session_id");
        let opened_at: chrono::DateTime<chrono::Utc> = row.get("opened_at");
        let closed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("closed_at");

        // Fetch items for each order
        let item_rows = sqlx::query!(
            r#"
            SELECT
                oi.id,
                oi.order_id,
                t.table_number,
                oi.menu_item_id,
                m.name as menu_item_name,
                oi.quantity,
                oi.unit_price,
                oi.note,
                oi.status,
                oi.created_at
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            JOIN tables t ON t.id = o.table_id
            JOIN menu_items m ON m.id = oi.menu_item_id
            WHERE oi.order_id = $1
            ORDER BY oi.created_at ASC
            "#,
            order_id
        )
        .fetch_all(&state.db)
        .await?;

        let items: Vec<OrderItemDetail> = item_rows
            .into_iter()
            .map(|r| OrderItemDetail {
                id: r.id,
                order_id: r.order_id,
                table_number: r.table_number,
                menu_item_id: r.menu_item_id,
                menu_item_name: r.menu_item_name,
                quantity: r.quantity,
                unit_price: r.unit_price,
                note: r.note,
                status: r.status,
                created_at: r.created_at,
            })
            .collect();

        orders.push(OrderDetail {
            id: order_id,
            table_id,
            table_number,
            table_name,
            session_token,
            status,
            total_amount,
            payment_method,
            stripe_session_id,
            opened_at,
            closed_at,
            items,
        });
    }

    Ok(Json(orders))
}

// Staff: Get specific order detail
pub async fn get_order_detail(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        &claims,
        &[
            UserRole::Admin,
            UserRole::Cashier,
            UserRole::Kitchen,
            UserRole::Waiter,
        ],
    )?;

    let o = sqlx::query!(
        r#"
        SELECT
            o.id,
            o.table_id,
            t.table_number,
            t.name as table_name,
            o.session_token,
            o.status,
            o.total_amount,
            o.payment_method,
            o.stripe_session_id,
            o.opened_at,
            o.closed_at
        FROM orders o
        JOIN tables t ON t.id = o.table_id
        WHERE o.id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;

    let item_rows = sqlx::query!(
        r#"
        SELECT
            oi.id,
            oi.order_id,
            t.table_number,
            oi.menu_item_id,
            m.name as menu_item_name,
            oi.quantity,
            oi.unit_price,
            oi.note,
            oi.status,
            oi.created_at
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        JOIN tables t ON t.id = o.table_id
        JOIN menu_items m ON m.id = oi.menu_item_id
        WHERE oi.order_id = $1
        ORDER BY oi.created_at ASC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    let items: Vec<OrderItemDetail> = item_rows
        .into_iter()
        .map(|r| OrderItemDetail {
            id: r.id,
            order_id: r.order_id,
            table_number: r.table_number,
            menu_item_id: r.menu_item_id,
            menu_item_name: r.menu_item_name,
            quantity: r.quantity,
            unit_price: r.unit_price,
            note: r.note,
            status: r.status,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(OrderDetail {
        id: o.id,
        table_id: o.table_id,
        table_number: o.table_number,
        table_name: o.table_name,
        session_token: o.session_token,
        status: o.status,
        total_amount: o.total_amount,
        payment_method: o.payment_method,
        stripe_session_id: o.stripe_session_id,
        opened_at: o.opened_at,
        closed_at: o.closed_at,
        items,
    }))
}
