use rustapi_openapi::{Operation, OperationModifier};
use rustapi_rs::{ApiError, Cookies, FromRequest, Request, Result};
use serde::de::DeserializeOwned;

/// Custom Form extractor for URL-encoded form data
/// Similar to Axum's Form extractor but works with RustAPI
pub struct Form<T>(pub T);

impl<T> std::ops::Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DeserializeOwned + Send + 'static> FromRequest for Form<T> {
    async fn from_request(req: &mut Request) -> Result<Self> {
        // Ensure the body is loaded
        req.load_body().await?;

        // Get the body bytes
        let body_bytes = req
            .take_body()
            .ok_or_else(|| ApiError::internal("Body already consumed"))?;

        // Parse as URL-encoded form data
        let form: T = serde_urlencoded::from_bytes(&body_bytes)
            .map_err(|e| ApiError::bad_request(format!("Invalid form data: {}", e)))?;

        Ok(Form(form))
    }
}

impl<T> OperationModifier for Form<T> {
    fn update_operation(_op: &mut Operation) {}
}

/// Wrapper around Cookies to satisfy OperationModifier bound
pub struct AppCookies(pub Cookies);

impl std::ops::Deref for AppCookies {
    type Target = Cookies;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for AppCookies {
    async fn from_request(req: &mut Request) -> Result<Self> {
        let cookies = Cookies::from_request(req).await?;
        Ok(AppCookies(cookies))
    }
}

impl OperationModifier for AppCookies {
    fn update_operation(_op: &mut Operation) {}
}
