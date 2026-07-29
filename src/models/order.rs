#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::order_item::OrderItemDetail;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub table_id: Uuid,
    pub session_token: String,
    pub status: String,
    pub total_amount: i64,
    pub payment_method: Option<String>,
    pub stripe_session_id: Option<String>,
    pub opened_by: Option<Uuid>,
    pub closed_by: Option<Uuid>,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct OrderDetail {
    pub id: Uuid,
    pub table_id: Uuid,
    pub table_number: i32,
    pub table_name: String,
    pub session_token: String,
    pub status: String,
    pub total_amount: i64,
    pub payment_method: Option<String>,
    pub stripe_session_id: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub items: Vec<OrderItemDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ManualPaymentReq {
    pub payment_method: String, // cash, card, transfer
}
