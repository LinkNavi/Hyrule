use axum::{
    extract::{State, Form},
    http::StatusCode,
    response::{Html, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use std::sync::Arc;
use time::Duration as TimeDuration;

use crate::AppState;
use crate::auth::session::SessionStore;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_form_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), (StatusCode, Html<String>)> {
    // Find user
    let user = state.db
        .get_user_by_username(&form.username)
        .await
        .map_err(|_| render_login_error("Invalid username or password"))?;
    
    // Verify password
    let is_valid = crate::auth::password::verify_password(&form.password, &user.password_hash)
        .map_err(|_| render_login_error("Invalid username or password"))?;
    
    if !is_valid {
        return Err(render_login_error("Invalid username or password"));
    }
    
    // Create session
    let session_id = state.session_store
        .create_session(user.id, user.username.clone())
        .await;
    
    // Set cookie (HttpOnly, Secure in production)
    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .max_age(TimeDuration::days(7))
        .http_only(true)
        .build();
    
    Ok((jar.add(cookie), Redirect::to("/dashboard")))
}

pub async fn logout_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    // Get session cookie
    if let Some(cookie) = jar.get("session_id") {
        state.session_store.delete_session(cookie.value()).await;
    }
    
    // Remove cookie
    let cookie = Cookie::build(("session_id", ""))
        .path("/")
        .max_age(TimeDuration::seconds(0))
        .build();
    
    (jar.add(cookie), Redirect::to("/"))
}

fn render_login_error(message: &str) -> (StatusCode, Html<String>) {
    let content = format!(
        r#"
        <div class="error-container">
            <h1>Login Failed</h1>
            <div class="error-message">
                <p>{}</p>
            </div>
            <div style="text-align: center; margin-top: 2rem;">
                <a href="/login" class="btn btn-primary">Try Again</a>
            </div>
        </div>
        "#,
        message
    );
    (StatusCode::UNAUTHORIZED, Html(crate::templates::render_page("Login Error", &content)))
}

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
}

pub async fn signup_form_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<SignupForm>,
) -> Result<(CookieJar, Redirect), (StatusCode, Html<String>)> {
    // Validate
    if form.username.len() < 3 || form.username.len() > 32 {
        return Err(render_signup_error("Username must be 3-32 characters"));
    }
    
    if form.password.len() < 8 {
        return Err(render_signup_error("Password must be at least 8 characters"));
    }
    
    // Hash password
    let password_hash = crate::auth::password::hash_password(&form.password)
        .map_err(|_| render_signup_error("Password hashing failed"))?;
    
    // Generate public key
    let public_key = hex::encode(blake3::hash(form.username.as_bytes()).as_bytes());
    let email = format!("{}@hyrule.local", form.username);
    
    let user_req = crate::models::CreateUserRequest {
        username: form.username.clone(),
        email,
        password_hash,
        public_key,
        storage_quota: state.config.default_storage_quota,
    };
    
    // Create user
    let user = state.db
        .create_user(&user_req)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                render_signup_error("Username already taken")
            } else {
                render_signup_error("Failed to create account")
            }
        })?;
    
    // Create session
    let session_id = state.session_store
        .create_session(user.id, user.username.clone())
        .await;
    
    // Set cookie
    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .max_age(TimeDuration::days(7))
        .http_only(true)
        .build();
    
    Ok((jar.add(cookie), Redirect::to("/dashboard")))
}

fn render_signup_error(message: &str) -> (StatusCode, Html<String>) {
    let content = format!(
        r#"
        <div class="error-container">
            <h1>Signup Failed</h1>
            <div class="error-message">
                <p>{}</p>
            </div>
            <div style="text-align: center; margin-top: 2rem;">
                <a href="/signup" class="btn btn-primary">Try Again</a>
            </div>
        </div>
        "#,
        message
    );
    (StatusCode::BAD_REQUEST, Html(crate::templates::render_page("Signup Error", &content)))
}
