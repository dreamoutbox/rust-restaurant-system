use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde_json::json;
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    Client, EventType, Webhook,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{require_role, AuthUser},
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
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    let order = sqlx::query!(
        "SELECT id, table_id, status FROM orders WHERE id = $1 AND status IN ('open', 'checkout_pending')",
        order_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Active order not found for checkout".to_string()))?;

    // Sum up order items total
    let total_row = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(quantity * unit_price), 0.00) as "total!: Decimal"
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
        SET status = 'checkout_pending', total_amount = $1
        WHERE id = $2
        RETURNING id, table_id, status, total_amount
        "#,
        total_amount,
        order_id
    )
    .fetch_one(&state.db)
    .await?;

    let table = sqlx::query!("SELECT table_number FROM tables WHERE id = $1", updated_order.table_id)
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
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    let valid_methods = ["cash", "card", "transfer"];
    if !valid_methods.contains(&payload.payment_method.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid payment method. Use cash, card, or transfer".to_string(),
        ));
    }

    let order = sqlx::query!(
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

    let table = sqlx::query!("SELECT table_number FROM tables WHERE id = $1", updated.table_id)
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
pub async fn create_stripe_checkout_session(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_role(&claims, &[UserRole::Admin, UserRole::Cashier])?;

    if state.config.stripe_secret_key.is_empty() || state.config.stripe_secret_key.starts_with("sk_test_mock") {
        return Err(AppError::BadRequest(
            "Stripe secret key not configured in .env".to_string(),
        ));
    }

    let items = sqlx::query!(
        r#"
        SELECT m.name, oi.quantity, oi.unit_price
        FROM order_items oi
        JOIN menu_items m ON m.id = oi.menu_item_id
        WHERE oi.order_id = $1
        "#,
        order_id
    )
    .fetch_all(&state.db)
    .await?;

    if items.is_empty() {
        return Err(AppError::BadRequest("Order has no items".to_string()));
    }

    let client = Client::new(&state.config.stripe_secret_key);

    let line_items: Vec<CreateCheckoutSessionLineItems> = items
        .into_iter()
        .map(|item| {
            // unit_price decimal to cents
            let amount_in_cents = (item.unit_price * Decimal::from(100))
                .to_string()
                .parse::<i64>()
                .unwrap_or(0);

            CreateCheckoutSessionLineItems {
                price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
                    currency: stripe::Currency::USD,
                    product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                        name: item.name,
                        ..Default::default()
                    }),
                    unit_amount: Some(amount_in_cents),
                    ..Default::default()
                }),
                quantity: Some(item.quantity as u64),
                ..Default::default()
            }
        })
        .collect();

    let success_url = format!("{}/payment-success?order_id={}", state.config.base_url, order_id);
    let cancel_url = format!("{}/payment-cancel?order_id={}", state.config.base_url, order_id);

    let mut params = CreateCheckoutSession::new();
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(line_items);
    params.success_url = Some(&success_url);
    params.cancel_url = Some(&cancel_url);
    params.client_reference_id = Some(&order_id.to_string());

    let session = CheckoutSession::create(&client, params)
        .await
        .map_err(|e| AppError::Internal(format!("Stripe session creation failed: {}", e)))?;

    sqlx::query!(
        "UPDATE orders SET stripe_session_id = $1, status = 'checkout_pending' WHERE id = $2",
        session.id.as_str(),
        order_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "checkout_url": session.url,
        "session_id": session.id
    })))
}

// Stripe Webhook Receiver
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    let sig = headers
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Stripe-Signature header".to_string()))?;

    let event = Webhook::construct_event(&body, sig, &state.config.stripe_webhook_secret)
        .map_err(|e| AppError::BadRequest(format!("Webhook signature verification failed: {}", e)))?;

    if event.type_ == EventType::CheckoutSessionCompleted {
        if let stripe::EventObject::CheckoutSession(session) = event.data.object {
            if let Some(ref client_ref) = session.client_reference_id {
                if let Ok(order_id) = Uuid::parse_str(client_ref) {
                    let updated = sqlx::query!(
                        r#"
                        UPDATE orders
                        SET status = 'paid', payment_method = 'stripe'
                        WHERE id = $1
                        RETURNING id, table_id
                        "#,
                        order_id
                    )
                    .fetch_optional(&state.db)
                    .await?;

                    if let Some(ord) = updated {
                        let table = sqlx::query!("SELECT table_number FROM tables WHERE id = $1", ord.table_id)
                            .fetch_one(&state.db)
                            .await?;

                        state.sse.send(SseEvent::PaymentReceived {
                            table_number: table.table_number,
                            order_id: ord.id,
                            method: "stripe".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(Json(json!({ "received": true })))
}
