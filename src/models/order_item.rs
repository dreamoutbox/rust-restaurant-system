#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub unit_price: i64,
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemDetail {
    pub id: Uuid,
    pub order_id: Uuid,
    pub table_number: i32,
    pub menu_item_id: Uuid,
    pub menu_item_name: String,
    pub quantity: i32,
    pub unit_price: i64,
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitOrderItemInput {
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitOrderItemsReq {
    pub items: Vec<SubmitOrderItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderItemStatusReq {
    pub status: String, // pending, preparing, finished, served, cancelled
}
