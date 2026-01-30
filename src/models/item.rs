use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Item model representing the items table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Item {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Form data for creating/updating items
#[derive(Debug, Deserialize)]
pub struct ItemForm {
    pub title: String,
    pub description: Option<String>,
}

/// Item creation data (includes user_id)
#[derive(Debug)]
pub struct CreateItem {
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
}
