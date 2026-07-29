use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, require_role},
    models::order_item::UpdateOrderItemStatusReq,
    models::user::UserRole,
    sse::SseEvent,
};

use super::auth::AppState;

pub async fn update_order_item_status(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrderItemStatusReq>,
) -> Result<impl IntoResponse, AppError> {
    // Authorized: Admin, Kitchen, Waiter
    require_role(
        &claims,
        &[UserRole::Admin, UserRole::Kitchen, UserRole::Waiter],
    )?;

    let valid_statuses = ["pending", "preparing", "finished", "served"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: pending, preparing, finished, served",
            payload.status
        )));
    }

    let item = sqlx::query!(
        r#"
        SELECT
            oi.id,
            oi.order_id,
            oi.status as current_status,
            m.name as item_name,
            t.table_number
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        JOIN tables t ON t.id = o.table_id
        JOIN menu_items m ON m.id = oi.menu_item_id
        WHERE oi.id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Order item not found".to_string()))?;

    sqlx::query!(
        "UPDATE order_items SET status = $1, updated_at = now() WHERE id = $2",
        payload.status,
        id
    )
    .execute(&state.db)
    .await?;

    // Broadcast SSE update to all listening clients (Kitchen, Waiter, Cashier)
    state.sse.send(SseEvent::ItemStatusChanged {
        table_number: item.table_number,
        order_id: item.order_id,
        item_id: item.id,
        item_name: item.item_name,
        status: payload.status.clone(),
    });

    Ok(Json(json!({
        "id": id,
        "status": payload.status,
        "message": "Order item status updated successfully"
    })))
}
