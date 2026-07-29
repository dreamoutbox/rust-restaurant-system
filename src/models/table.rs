use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Table {
    pub id: Uuid,
    pub table_number: i32,
    pub name: String,
    pub capacity: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableWithStatus {
    pub id: Uuid,
    pub table_number: i32,
    pub name: String,
    pub capacity: Option<i32>,
    pub is_active: bool,
    pub active_order_id: Option<Uuid>,
    pub active_session_token: Option<String>,
    pub order_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTableReq {
    pub table_number: i32,
    pub name: String,
    pub capacity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTableReq {
    pub table_number: Option<i32>,
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub is_active: Option<bool>,
}
