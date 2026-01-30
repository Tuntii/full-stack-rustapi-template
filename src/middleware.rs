use jsonwebtoken::{decode, DecodingKey, Validation};
use rustapi_rs::prelude::*;

use crate::{
    models::{Claims, UserInfo},
    AppState,
};

/// Extract JWT token from cookies
fn extract_token_from_cookies(cookies: &Cookies) -> Option<String> {
    cookies.get("token").map(|c| c.value().to_string())
}

/// Get current user from JWT cookie
pub async fn get_current_user(state: &AppState, cookies: &Cookies) -> Option<UserInfo> {
    let token = extract_token_from_cookies(cookies)?;

    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?
    .claims;

    let user = state.db.find_user_by_id(claims.sub).await.ok()??;

    Some(UserInfo::from(user))
}
