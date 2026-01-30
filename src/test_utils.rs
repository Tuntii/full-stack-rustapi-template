#[cfg(test)]
use std::{path::PathBuf, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

#[cfg(test)]
use cookie::{Cookie, CookieJar};
#[cfg(test)]
use jsonwebtoken::{encode, EncodingKey, Header};
#[cfg(test)]
use rustapi_rs::{Cookies, Response};
#[cfg(test)]
use tera::Tera;

#[cfg(test)]
use crate::{
    db::Database,
    extractors::AppCookies,
    models::Claims,
    AppState,
};

#[cfg(test)]
pub async fn setup_test_state() -> (AppState, PathBuf) {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("basic_crud_ops_api_test_{}.db", nanos));

    let url = format!("sqlite:{}?mode=rwc", path.display());
    let db = Database::new(&url).await.expect("create test db");

    let mut tera = Tera::default();
    add_test_templates(&mut tera);

    let state = AppState {
        db,
        tera: Arc::new(tera),
        jwt_secret: "test-secret".to_string(),
    };

    (state, path)
}

#[cfg(test)]
pub fn cleanup_db(path: PathBuf) {
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
pub fn empty_cookies() -> AppCookies {
    AppCookies(Cookies(CookieJar::new()))
}

#[cfg(test)]
pub fn cookies_for_user(secret: &str, user_id: i64, username: &str) -> AppCookies {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: now + 3600,
        iat: now,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .expect("encode token");

    let mut jar = CookieJar::new();
    jar.add(Cookie::new("token", token));

    AppCookies(Cookies(jar))
}

#[cfg(test)]
pub fn header_value(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
fn add_test_templates(tera: &mut Tera) {
    tera.add_raw_template("index.html", "HOME")
        .expect("add index template");
    tera.add_raw_template("auth/login.html", "LOGIN")
        .expect("add login template");
    tera.add_raw_template("auth/register.html", "REGISTER")
        .expect("add register template");
    tera.add_raw_template("items/list.html", "ITEMS LIST")
        .expect("add items list template");
    tera.add_raw_template("items/form.html", "ITEMS FORM")
        .expect("add items form template");
}
