use rustapi_rs::prelude::*;
use tera::Context;

use crate::{extractors::AppCookies, models::UserInfo, middleware::get_current_user, AppState};

/// Home page handler
#[rustapi_rs::get("/")]
pub async fn home(
    State(state): State<AppState>,
    cookies: AppCookies,
) -> Response {
    let mut context = Context::new();
    
    // Try to get current user (optional)
    if let Some(user) = get_current_user(&state, &cookies).await {
        context.insert("user", &Some(&user));
    } else {
        context.insert("user", &None::<UserInfo>);
    }

    match state.tera.render("index.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{cleanup_db, cookies_for_user, empty_cookies, setup_test_state};

    #[tokio::test]
    async fn home_returns_ok_for_anonymous() {
        let (state, path) = setup_test_state().await;
        let response = home(State(state.clone()), empty_cookies()).await;
        assert_eq!(response.status(), StatusCode::OK);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn home_returns_ok_for_authenticated_user() {
        let (state, path) = setup_test_state().await;
        let user = state
            .db
            .create_user("viewer", "viewer@example.com", "hash")
            .await
            .expect("create user");
        let cookies = cookies_for_user(&state.jwt_secret, user.id, &user.username);
        let response = home(State(state.clone()), cookies).await;
        assert_eq!(response.status(), StatusCode::OK);
        cleanup_db(path);
    }
}
