use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User model representing the users table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
}

/// User data for templates (without sensitive fields)
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

/// Form data for user registration
#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

/// Form data for user login
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user id
    pub username: String,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
}
