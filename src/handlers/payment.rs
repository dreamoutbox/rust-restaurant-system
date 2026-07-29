use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, require_role},
    models::order::ManualPaymentReq,
    models::user::UserRole,
    sse::SseEvent,
};

use super::auth::AppState;

// Calculate order total and transition order to checkout_pending status
pub async fn checkout_order(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    let _order = sqlx::query!(
        "SELECT id, table_id, status FROM orders WHERE id = $1 AND status IN ('open', 'checkout_pending')",
        order_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Active order not found for checkout".to_string()))?;

    // Sum up order items total
    let total_row = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(quantity * unit_price), 0)::bigint as "total!: i64"
        FROM order_items
        WHERE order_id = $1
        "#,
        order_id
    )
    .fetch_one(&state.db)
    .await?;

    let total_amount = total_row.total;

    let updated_order = sqlx::query!(
        r#"
        UPDATE orders
        SET status = 'checkout_pending', total_amount = $1::bigint
        WHERE id = $2
        RETURNING id, table_id, status, total_amount as "total_amount!: i64"
        "#,
        total_amount,
        order_id
    )
    .fetch_one(&state.db)
    .await?;

    let table = sqlx::query!(
        "SELECT table_number FROM tables WHERE id = $1",
        updated_order.table_id
    )
    .fetch_one(&state.db)
    .await?;

    state.sse.send(SseEvent::OrderCheckout {
        table_number: table.table_number,
        order_id: updated_order.id,
        total: total_amount.to_string(),
    });

    Ok(Json(json!({
        "order_id": updated_order.id,
        "status": updated_order.status,
        "total_amount": updated_order.total_amount
    })))
}

// Cashier records manual payment (Cash, Card, Transfer)
pub async fn record_manual_payment(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<ManualPaymentReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    let valid_methods = ["cash", "card", "transfer"];
    if !valid_methods.contains(&payload.payment_method.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid payment method. Use cash, card, or transfer".to_string(),
        ));
    }

    let _order = sqlx::query!(
        "SELECT id, table_id, status, total_amount FROM orders WHERE id = $1 AND status IN ('open', 'checkout_pending')",
        order_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Order not available for payment".to_string()))?;

    let updated = sqlx::query!(
        r#"
        UPDATE orders
        SET status = 'paid', payment_method = $1, closed_by = $2
        WHERE id = $3
        RETURNING id, table_id, status, total_amount, payment_method
        "#,
        payload.payment_method,
        claims.sub,
        order_id
    )
    .fetch_one(&state.db)
    .await?;

    let table = sqlx::query!(
        "SELECT table_number FROM tables WHERE id = $1",
        updated.table_id
    )
    .fetch_one(&state.db)
    .await?;

    state.sse.send(SseEvent::PaymentReceived {
        table_number: table.table_number,
        order_id: updated.id,
        method: payload.payment_method.clone(),
    });

    Ok(Json(json!({
        "order_id": updated.id,
        "status": updated.status,
        "total_amount": updated.total_amount,
        "payment_method": updated.payment_method,
        "message": "Payment recorded successfully"
    })))
}

// Create Stripe Checkout Session
// NOTE: Stripe integration is stubbed pending async-stripe API alignment.
pub async fn create_stripe_checkout_session(
    State(_state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(_order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;
    Err(AppError::Internal(
        "Stripe checkout integration not yet configured. Use manual payment instead.".to_string(),
    ))
}

// Stripe Webhook Receiver (stub)
pub async fn handle_stripe_webhook(
    State(_state): State<AppState>,
    _headers: HeaderMap,
    _body: String,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({ "received": true })))
}
