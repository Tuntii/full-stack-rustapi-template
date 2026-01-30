use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;

use crate::models::{User, Item, CreateItem};

/// Database connection pool wrapper
#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    /// Create a new database connection and run migrations
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Ensure database file exists
        let db_path = database_url.replace("sqlite:", "").replace("?mode=rwc", "");
        if !Path::new(&db_path).exists() {
            std::fs::File::create(&db_path).ok();
        }

        let pool = SqlitePool::connect(database_url).await?;
        
        let db = Self { pool };
        db.run_migrations().await?;
        
        Ok(db)
    }

    /// Run SQL migrations
    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        // Create tables directly
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    // ==================== User Operations ====================

    /// Create a new user
    pub async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES (?, ?, ?)
            RETURNING id, username, email, password_hash, created_at
            "#
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }

    /// Find user by username
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    /// Find user by ID
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    /// Check if username exists
    pub async fn username_exists(&self, username: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result.0 > 0)
    }

    /// Check if email exists
    pub async fn email_exists(&self, email: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result.0 > 0)
    }

    // ==================== Item Operations ====================

    /// Create a new item
    pub async fn create_item(&self, item: CreateItem) -> Result<Item, sqlx::Error> {
        let created = sqlx::query_as::<_, Item>(
            r#"
            INSERT INTO items (user_id, title, description)
            VALUES (?, ?, ?)
            RETURNING id, user_id, title, description, created_at, updated_at
            "#
        )
        .bind(item.user_id)
        .bind(&item.title)
        .bind(&item.description)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(created)
    }

    /// Get all items for a user
    pub async fn get_user_items(&self, user_id: i64) -> Result<Vec<Item>, sqlx::Error> {
        let items = sqlx::query_as::<_, Item>(
            r#"
            SELECT id, user_id, title, description, created_at, updated_at
            FROM items
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(items)
    }

    /// Get a single item by ID (must belong to user)
    pub async fn get_item(&self, id: i64, user_id: i64) -> Result<Option<Item>, sqlx::Error> {
        let item = sqlx::query_as::<_, Item>(
            r#"
            SELECT id, user_id, title, description, created_at, updated_at
            FROM items
            WHERE id = ? AND user_id = ?
            "#
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(item)
    }

    /// Update an item
    pub async fn update_item(&self, id: i64, user_id: i64, title: &str, description: Option<&str>) -> Result<Option<Item>, sqlx::Error> {
        let item = sqlx::query_as::<_, Item>(
            r#"
            UPDATE items
            SET title = ?, description = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND user_id = ?
            RETURNING id, user_id, title, description, created_at, updated_at
            "#
        )
        .bind(title)
        .bind(description)
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(item)
    }

    /// Delete an item
    pub async fn delete_item(&self, id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM items WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::models::CreateItem;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn setup_test_db() -> (Database, PathBuf) {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("basic_crud_ops_test_{}.db", nanos));

        let url = format!("sqlite:{}?mode=rwc", path.display());
        let db = Database::new(&url).await.expect("create test db");
        (db, path)
    }

    fn cleanup_db(path: PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn user_queries_work() {
        let (db, path) = setup_test_db().await;

        let user = db
            .create_user("alice", "alice@example.com", "hash")
            .await
            .expect("create user");

        let by_username = db
            .find_user_by_username("alice")
            .await
            .expect("find by username")
            .expect("user exists");

        assert_eq!(user.id, by_username.id);

        let by_id = db
            .find_user_by_id(user.id)
            .await
            .expect("find by id")
            .expect("user exists");

        assert_eq!(by_id.username, "alice");
        assert!(db.username_exists("alice").await.expect("username exists"));
        assert!(db.email_exists("alice@example.com").await.expect("email exists"));

        cleanup_db(path);
    }

    #[tokio::test]
    async fn item_crud_works() {
        let (db, path) = setup_test_db().await;

        let user = db
            .create_user("bob", "bob@example.com", "hash")
            .await
            .expect("create user");

        let created = db
            .create_item(CreateItem {
                user_id: user.id,
                title: "First".to_string(),
                description: Some("Desc".to_string()),
            })
            .await
            .expect("create item");

        let items = db
            .get_user_items(user.id)
            .await
            .expect("list items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "First");

        let fetched = db
            .get_item(created.id, user.id)
            .await
            .expect("get item")
            .expect("item exists");
        assert_eq!(fetched.description.as_deref(), Some("Desc"));

        let updated = db
            .update_item(created.id, user.id, "Updated", Some("New"))
            .await
            .expect("update item")
            .expect("updated item");
        assert_eq!(updated.title, "Updated");

        let deleted = db
            .delete_item(created.id, user.id)
            .await
            .expect("delete item");
        assert!(deleted);

        cleanup_db(path);
    }
}
