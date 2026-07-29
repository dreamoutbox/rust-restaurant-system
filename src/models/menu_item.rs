use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub image_path: Option<String>,
    pub is_available: bool,
    pub sort_order: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemWithCategory {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub image_path: Option<String>,
    pub is_available: bool,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateMenuItemReq {
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub image_path: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMenuItemReq {
    pub category_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub image_path: Option<String>,
    pub is_available: Option<bool>,
    pub sort_order: Option<i32>,
}
