use rustapi_rs::prelude::*;
use tera::Context;

use crate::{
    extractors::{AppCookies, Form},
    middleware::get_current_user,
    models::{CreateItem, ItemForm},
    AppState,
};

/// List all items for the current user
#[rustapi_rs::get("/items")]
pub async fn list_items(State(state): State<AppState>, cookies: AppCookies) -> Response {
    let mut context = Context::new();

    // Get current user from JWT
    let user = match get_current_user(&state, &cookies).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    context.insert("user", &Some(&user));

    let items = match state.db.get_user_items(user.id).await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Database error: {}", e);
            context.insert("error", "Failed to load items");
            vec![]
        }
    };

    context.insert("items", &items);

    render_template(&state, "items/list.html", &context)
}

/// Show form to create a new item
#[rustapi_rs::get("/items/new")]
pub async fn new_item_form(State(state): State<AppState>, cookies: AppCookies) -> Response {
    let user = match get_current_user(&state, &cookies).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let mut context = Context::new();
    context.insert("user", &Some(&user));
    context.insert("item", &None::<()>);

    render_template(&state, "items/form.html", &context)
}

/// Create a new item
#[rustapi_rs::post("/items")]
pub async fn create_item(
    State(state): State<AppState>,
    cookies: AppCookies,
    Form(form): Form<ItemForm>,
) -> Response {
    let user = match get_current_user(&state, &cookies).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let mut context = Context::new();
    context.insert("user", &Some(&user));

    // Validate
    if let Err(validation_errors) = form.validate() {
        let error_msg = format!("Validation error: {:?}", validation_errors);

        context.insert("error", &error_msg);
        context.insert("item", &None::<()>);
        return render_template(&state, "items/form.html", &context);
    }

    let create_item = CreateItem {
        user_id: user.id,
        title: form.title.trim().to_string(),
        description: form
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
    };

    match state.db.create_item(create_item).await {
        Ok(_) => Redirect::to("/items?success=created").into_response(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            context.insert("error", "Failed to create item");
            context.insert("item", &None::<()>);
            render_template(&state, "items/form.html", &context)
        }
    }
}

/// Show form to edit an item
#[rustapi_rs::get("/items/{id}/edit")]
pub async fn edit_item_form(
    State(state): State<AppState>,
    cookies: AppCookies,
    Path(id): Path<i64>,
) -> Response {
    let user = match get_current_user(&state, &cookies).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let mut context = Context::new();
    context.insert("user", &Some(&user));

    let item = match state.db.get_item(id, user.id).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            return Redirect::to("/items?error=not_found").into_response();
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Redirect::to("/items?error=database").into_response();
        }
    };

    context.insert("item", &Some(&item));

    render_template(&state, "items/form.html", &context)
}

/// Update an item
#[rustapi_rs::post("/items/{id}")]
pub async fn update_item(
    State(state): State<AppState>,
    cookies: AppCookies,
    Path(id): Path<i64>,
    Form(form): Form<ItemForm>,
) -> Response {
    let user = match get_current_user(&state, &cookies).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let mut context = Context::new();
    context.insert("user", &Some(&user));

    // Validate
    if let Err(validation_errors) = form.validate() {
        if let Ok(Some(item)) = state.db.get_item(id, user.id).await {
            context.insert("item", &Some(&item));
        }

        let error_msg = format!("Validation error: {:?}", validation_errors);

        context.insert("error", &error_msg);
        return render_template(&state, "items/form.html", &context);
    }

    let description = form
        .description
        .as_deref()
        .map(|d| d.trim())
        .filter(|d| !d.is_empty());

    match state
        .db
        .update_item(id, user.id, form.title.trim(), description)
        .await
    {
        Ok(Some(_)) => Redirect::to("/items?success=updated").into_response(),
        Ok(None) => Redirect::to("/items?error=not_found").into_response(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            if let Ok(Some(item)) = state.db.get_item(id, user.id).await {
                context.insert("item", &Some(&item));
            }
            context.insert("error", "Failed to update item");
            render_template(&state, "items/form.html", &context)
        }
    }
}

/// Delete an item
#[rustapi_rs::post("/items/{id}/delete")]
pub async fn delete_item(
    State(state): State<AppState>,
    cookies: AppCookies,
    Path(id): Path<i64>,
) -> Response {
    let user = match get_current_user(&state, &cookies).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    match state.db.delete_item(id, user.id).await {
        Ok(true) => Redirect::to("/items?success=deleted").into_response(),
        Ok(false) => Redirect::to("/items?error=not_found").into_response(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Redirect::to("/items?error=database").into_response()
        }
    }
}

// Helper function to render templates
fn render_template(state: &AppState, template: &str, context: &Context) -> Response {
    match state.tera.render(template, context) {
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
    use crate::test_utils::{
        cleanup_db, cookies_for_user, empty_cookies, header_value, setup_test_state,
    };
    use rustapi_rs::Path;

    async fn setup_user(state: &AppState) -> (i64, AppCookies) {
        let user = state
            .db
            .create_user("user", "user@example.com", "hash")
            .await
            .expect("create user");
        let cookies = cookies_for_user(&state.jwt_secret, user.id, &user.username);
        (user.id, cookies)
    }

    #[tokio::test]
    async fn list_items_requires_auth() {
        let (state, path) = setup_test_state().await;
        let response = list_items(State(state.clone()), empty_cookies()).await;
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            header_value(&response, "Location"),
            Some("/login".to_string())
        );
        cleanup_db(path);
    }

    #[tokio::test]
    async fn list_items_returns_ok_for_authenticated_user() {
        let (state, path) = setup_test_state().await;
        let (user_id, cookies) = setup_user(&state).await;
        state
            .db
            .create_item(CreateItem {
                user_id,
                title: "Item".to_string(),
                description: None,
            })
            .await
            .expect("create item");

        let response = list_items(State(state.clone()), cookies).await;
        assert_eq!(response.status(), StatusCode::OK);
        cleanup_db(path);
    }

    #[tokio::test]
    async fn create_item_validates_title() {
        let (state, path) = setup_test_state().await;
        let (user_id, cookies) = setup_user(&state).await;

        let response = create_item(
            State(state.clone()),
            cookies,
            Form(ItemForm {
                title: "".to_string(),
                description: None,
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let items = state.db.get_user_items(user_id).await.expect("items");
        assert!(items.is_empty());
        cleanup_db(path);
    }

    #[tokio::test]
    async fn create_item_redirects_on_success() {
        let (state, path) = setup_test_state().await;
        let (_user_id, cookies) = setup_user(&state).await;

        let response = create_item(
            State(state.clone()),
            cookies,
            Form(ItemForm {
                title: "New".to_string(),
                description: Some("Desc".to_string()),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            header_value(&response, "Location"),
            Some("/items?success=created".to_string())
        );
        cleanup_db(path);
    }

    #[tokio::test]
    async fn edit_item_form_redirects_when_missing() {
        let (state, path) = setup_test_state().await;
        let (_user_id, cookies) = setup_user(&state).await;

        let response = edit_item_form(State(state.clone()), cookies, Path(999)).await;
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            header_value(&response, "Location"),
            Some("/items?error=not_found".to_string())
        );
        cleanup_db(path);
    }

    #[tokio::test]
    async fn update_item_redirects_when_missing() {
        let (state, path) = setup_test_state().await;
        let (_user_id, cookies) = setup_user(&state).await;

        let response = update_item(
            State(state.clone()),
            cookies,
            Path(999),
            Form(ItemForm {
                title: "Title".to_string(),
                description: None,
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            header_value(&response, "Location"),
            Some("/items?error=not_found".to_string())
        );
        cleanup_db(path);
    }

    #[tokio::test]
    async fn delete_item_redirects_on_success() {
        let (state, path) = setup_test_state().await;
        let (user_id, cookies) = setup_user(&state).await;

        let item = state
            .db
            .create_item(CreateItem {
                user_id,
                title: "Delete".to_string(),
                description: None,
            })
            .await
            .expect("create item");

        let response = delete_item(State(state.clone()), cookies, Path(item.id)).await;
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            header_value(&response, "Location"),
            Some("/items?success=deleted".to_string())
        );
        cleanup_db(path);
    }
}
