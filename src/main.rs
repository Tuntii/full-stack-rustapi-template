mod db;
mod extractors;
mod handlers;
mod middleware;
mod models;
#[cfg(test)]
mod test_utils;

use std::sync::Arc;
use rustapi_rs::prelude::*;
use tera::Tera;

use db::Database;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub tera: Arc<Tera>,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data.db?mode=rwc".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-super-secret-key-change-in-production".to_string());
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    println!("🚀 Starting CRUD App with RustAPI...");
    println!("📦 Connecting to database...");

    // Initialize database
    let db = Database::new(&database_url).await?;
    println!("✅ Database connected and migrations applied");

    // Initialize Tera templates
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => Arc::new(t),
        Err(e) => {
            eprintln!("Template parsing error: {}", e);
            std::process::exit(1);
        }
    };
    println!("✅ Templates loaded");

    // Create app state
    let state = AppState {
        db,
        tera,
        jwt_secret,
    };

    println!("🌐 Server running at http://{}:{}", host, port);
    println!("📝 Visit http://{}:{} to get started", host, port);

    let addr = format!("{}:{}", host, port);

    // Build and run RustAPI server (auto routes)
    RustApi::auto()
        .state(state)
        // Static files
        .status_page()
        .serve_static("/static", "static")
        .run(&addr)
        .await?;

    Ok(())
}
