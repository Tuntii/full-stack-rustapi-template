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

use rustapi_macros::Validate;
use rustapi_rs::prelude::*;

/// Form data for creating/updating items
#[derive(Debug, Deserialize, Validate)]
pub struct ItemForm {
    #[validate(
        length(min = 1, message = "Title is required"),
        length(max = 200, message = "Title must be 200 characters or less")
    )]
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
