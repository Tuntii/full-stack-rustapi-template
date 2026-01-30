use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rustapi_rs::prelude::*;
use rustapi_rs::ResponseBody;
use jsonwebtoken::{encode, Header, EncodingKey};
use tera::Context;

use crate::{
    extractors::Form,
    models::{Claims, LoginForm, RegisterForm, UserInfo},
    AppState,
};

/// Show login page
#[rustapi_rs::get("/login")]
pub async fn show_login(State(state): State<AppState>) -> Response {
    let mut context = Context::new();
    context.insert("user", &None::<UserInfo>);
    
    match state.tera.render("auth/login.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

/// Handle login form submission
#[rustapi_rs::post("/login")]
pub async fn handle_login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let mut context = Context::new();
    context.insert("user", &None::<UserInfo>);
    context.insert("username", &form.username);

    // Find user
    let user = match state.db.find_user_by_username(&form.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            context.insert("error", "Invalid username or password");
            return render_login(&state.tera, &context);
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            context.insert("error", "An error occurred. Please try again.");
            return render_login(&state.tera, &context);
        }
    };

    // Verify password
    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(_) => {
            context.insert("error", "An error occurred. Please try again.");
            return render_login(&state.tera, &context);
        }
    };

    if Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        context.insert("error", "Invalid username or password");
        return render_login(&state.tera, &context);
    }

    // Create JWT token
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        exp: now + 86400, // 24 hours
        iat: now,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {}", e);
            context.insert("error", "An error occurred. Please try again.");
            return render_login(&state.tera, &context);
        }
    };

    // Set cookie and redirect
    let cookie = format!(
        "token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400",
        token
    );

    redirect_with_cookie("/items", &cookie)
}

/// Show registration page
#[rustapi_rs::get("/register")]
pub async fn show_register(State(state): State<AppState>) -> Response {
    let mut context = Context::new();
    context.insert("user", &None::<UserInfo>);
    
    match state.tera.render("auth/register.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

/// Handle registration form submission
#[rustapi_rs::post("/register")]
pub async fn handle_register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Response {
    let mut context = Context::new();
    context.insert("user", &None::<UserInfo>);
    context.insert("username", &form.username);
    context.insert("email", &form.email);

    // Validate form
    if form.username.len() < 3 {
        context.insert("error", "Username must be at least 3 characters");
        return render_register(&state.tera, &context);
    }

    if form.password.len() < 6 {
        context.insert("error", "Password must be at least 6 characters");
        return render_register(&state.tera, &context);
    }

    if form.password != form.confirm_password {
        context.insert("error", "Passwords do not match");
        return render_register(&state.tera, &context);
    }

    // Check if username exists
    match state.db.username_exists(&form.username).await {
        Ok(true) => {
            context.insert("error", "Username is already taken");
            return render_register(&state.tera, &context);
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            context.insert("error", "An error occurred. Please try again.");
            return render_register(&state.tera, &context);
        }
        _ => {}
    }

    // Check if email exists
    match state.db.email_exists(&form.email).await {
        Ok(true) => {
            context.insert("error", "Email is already registered");
            return render_register(&state.tera, &context);
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            context.insert("error", "An error occurred. Please try again.");
            return render_register(&state.tera, &context);
        }
        _ => {}
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match Argon2::default().hash_password(form.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            eprintln!("Password hash error: {}", e);
            context.insert("error", "An error occurred. Please try again.");
            return render_register(&state.tera, &context);
        }
    };

    // Create user
    if let Err(e) = state
        .db
        .create_user(&form.username, &form.email, &password_hash)
        .await
    {
        eprintln!("Database error: {}", e);
        context.insert("error", "An error occurred. Please try again.");
        return render_register(&state.tera, &context);
    }

    // Redirect to login with success message
    Redirect::to("/login?registered=true").into_response()
}

/// Handle logout
#[rustapi_rs::post("/logout")]
pub async fn handle_logout() -> Response {
    let cookie = "token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0";
    redirect_with_cookie("/", cookie)
}

// Helper function to redirect with a Set-Cookie header
fn redirect_with_cookie(location: &str, cookie: &str) -> Response {
    let mut response = Response::new(ResponseBody::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;

    if let Ok(value) = location.parse() {
        response.headers_mut().insert("Location", value);
    }

    if let Ok(value) = cookie.parse() {
        response.headers_mut().insert("Set-Cookie", value);
    }

    response
}

// Helper functions
fn render_login(tera: &tera::Tera, context: &Context) -> Response {
    match tera.render("auth/login.html", context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

fn render_register(tera: &tera::Tera, context: &Context) -> Response {
    match tera.render("auth/register.html", context) {
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
    use crate::{extractors::Form, models::{LoginForm, RegisterForm}};
    use crate::test_utils::{cleanup_db, header_value, setup_test_state};
    use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};

    fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("hash password")
            .to_string()
    }

    #[tokio::test]
    async fn show_login_returns_ok() {
        let (state, path) = setup_test_state().await;
        let response = show_login(State(state.clone())).await;
        assert_eq!(response.status(), StatusCode::OK);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn show_register_returns_ok() {
        let (state, path) = setup_test_state().await;
        let response = show_register(State(state.clone())).await;
        assert_eq!(response.status(), StatusCode::OK);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn handle_register_rejects_invalid_form() {
        let (state, path) = setup_test_state().await;
        let form = RegisterForm {
            username: "ab".to_string(),
            email: "bad@example.com".to_string(),
            password: "short".to_string(),
            confirm_password: "mismatch".to_string(),
        };

        let response = handle_register(State(state.clone()), Form(form)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let exists = state.db.username_exists("ab").await.expect("username exists");
        assert!(!exists);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn handle_register_success_redirects() {
        let (state, path) = setup_test_state().await;
        let form = RegisterForm {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "password123".to_string(),
            confirm_password: "password123".to_string(),
        };

        let response = handle_register(State(state.clone()), Form(form)).await;
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(header_value(&response, "Location"), Some("/login?registered=true".to_string()));

        let exists = state.db.username_exists("alice").await.expect("username exists");
        assert!(exists);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn handle_login_invalid_password_renders_form() {
        let (state, path) = setup_test_state().await;
        let hash = hash_password("correct-password");
        state
            .db
            .create_user("bob", "bob@example.com", &hash)
            .await
            .expect("create user");

        let response = handle_login(
            State(state.clone()),
            Form(LoginForm {
                username: "bob".to_string(),
                password: "wrong".to_string(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn handle_login_sets_cookie_and_redirects() {
        let (state, path) = setup_test_state().await;
        let hash = hash_password("secret");
        state
            .db
            .create_user("carol", "carol@example.com", &hash)
            .await
            .expect("create user");

        let response = handle_login(
            State(state.clone()),
            Form(LoginForm {
                username: "carol".to_string(),
                password: "secret".to_string(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(header_value(&response, "Location"), Some("/items".to_string()));
        let set_cookie = header_value(&response, "Set-Cookie").unwrap_or_default();
        assert!(set_cookie.contains("token="));
        cleanup_db(path);
    }

    #[tokio::test]
    async fn handle_logout_clears_cookie() {
        let (_state, path) = setup_test_state().await;
        let response = handle_logout().await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(header_value(&response, "Location"), Some("/".to_string()));
        let set_cookie = header_value(&response, "Set-Cookie").unwrap_or_default();
        assert!(set_cookie.contains("Max-Age=0"));
        cleanup_db(path);
    }
}
