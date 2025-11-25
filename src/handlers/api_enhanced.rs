// src/handlers/api_enhanced.rs
use axum::{
    extract::{Path, Query, State, Form},
    http::StatusCode,
    Json,
    response::{Html, Redirect},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::AuthUser;
use crate::models::*;
use crate::AppState;

// Authentication endpoints
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub storage_used: i64,
    pub storage_quota: i64,
}

// Form-based login (no JS required)
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_form(
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let user = state.db
        .get_user_by_username(&form.username)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, Html(render_error("Invalid username or password"))))?;
    
    let is_valid = crate::auth::password::verify_password(&form.password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Html(render_error("Authentication error"))))?;
    
    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, Html(render_error("Invalid username or password"))));
    }
    
    // In a real app, you'd set a session cookie here
    // For now, just redirect to dashboard
    Ok(Redirect::to("/dashboard"))
}

// API login (JSON)
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = state.db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|e| {
            eprintln!("User not found: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string())
        })?;
    
    let is_valid = crate::auth::password::verify_password(&payload.password, &user.password_hash)
        .map_err(|e| {
            eprintln!("Password verification error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error".to_string())
        })?;
    
    if !is_valid {
        eprintln!("Invalid password for user: {}", payload.username);
        return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()));
    }
    
    let token = crate::auth::jwt::generate_token(user.id, &user.username)
        .map_err(|e| {
            eprintln!("Token generation error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string())
        })?;
    
    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            storage_used: user.storage_used,
            storage_quota: user.storage_quota,
        },
    }))
}

// Form-based signup (no JS required, no email required)
#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
}

pub async fn signup_form(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SignupForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    // Validate username
    if form.username.len() < 3 || form.username.len() > 32 {
        return Err((StatusCode::BAD_REQUEST, Html(render_error("Username must be 3-32 characters"))));
    }
    
    if form.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, Html(render_error("Password must be at least 8 characters"))));
    }
    
    let password_hash = crate::auth::password::hash_password(&form.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Html(render_error("Password hashing failed"))))?;
    
    // Generate a dummy public key
    let public_key = hex::encode(blake3::hash(form.username.as_bytes()).as_bytes());
    
    // Use username@hyrule.local as email (not displayed to users)
    let email = format!("{}@hyrule.local", form.username);
    
    let user_req = CreateUserRequest {
        username: form.username.clone(),
        email,
        password_hash,
        public_key,
        storage_quota: state.config.default_storage_quota,
    };
    
    state.db
        .create_user(&user_req)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                (StatusCode::CONFLICT, Html(render_error("Username already taken")))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Html(render_error("Failed to create account")))
            }
        })?;
    
    // Redirect to login page on success
    Ok(Redirect::to("/login?success=Account created successfully"))
}

// API signup (JSON) - also no email required
#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    if payload.username.len() < 3 || payload.username.len() > 32 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if payload.password.len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let password_hash = crate::auth::password::hash_password(&payload.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let public_key = hex::encode(blake3::hash(payload.username.as_bytes()).as_bytes());
    let email = format!("{}@hyrule.local", payload.username);
    
    let user_req = CreateUserRequest {
        username: payload.username,
        email,
        password_hash,
        public_key,
        storage_quota: state.config.default_storage_quota,
    };
    
    let user = state.db
        .create_user(&user_req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let token = crate::auth::jwt::generate_token(user.id, &user.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            storage_used: user.storage_used,
            storage_quota: user.storage_quota,
        },
    }))
}

fn render_error(message: &str) -> String {
    use crate::templates::render_page;
    let content = format!(
        r#"
        <div class="error-container">
            <h1>Error</h1>
            <div class="error-message">
                <p>{}</p>
            </div>
            <div style="text-align: center; margin-top: 2rem;">
                <a href="javascript:history.back()" class="btn btn-secondary">Go Back</a>
            </div>
        </div>
        "#,
        message
    );
    render_page("Error", &content)
}

// Enhanced repository endpoints with authentication
pub async fn create_repo_authenticated(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(payload): Json<CreateRepoRequest>,
) -> Result<Json<CreateRepoResponse>, StatusCode> {
    let repo_hash = crate::utils::hash::generate_repo_hash(&payload.name, user.id);
    
    state.db
        .create_repository(&payload, user.id, &repo_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Initialize Git storage for this repository
    state.git_storage
        .init_repo(&repo_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(CreateRepoResponse {
        repo_hash: repo_hash.clone(),
        message: format!("Repository created with hash: {}", repo_hash),
    }))
}

// Search repositories
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub results: Vec<Repository>,
    pub count: usize,
}

pub async fn search_repos(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResults>, StatusCode> {
    let limit = query.limit.unwrap_or(20).min(100);
    
    let results = state.db
        .search_repositories(&query.q, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let count = results.len();
    
    Ok(Json(SearchResults { results, count }))
}

// Get user profile
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<UserInfo>, StatusCode> {
    let db_user = state.db
        .get_user_by_id(user.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(UserInfo {
        id: db_user.id,
        username: db_user.username,
        email: db_user.email,
        storage_used: db_user.storage_used,
        storage_quota: db_user.storage_quota,
    }))
}

// Repository statistics
#[derive(Debug, Serialize)]
pub struct RepoStats {
    pub total_size: i64,
    pub total_clones: i64,
    pub total_bandwidth: i64,
    pub replica_health: f64,
}

pub async fn get_repo_stats(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Json<RepoStats>, StatusCode> {
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let replica_count = state.db
        .get_replica_count(&repo_hash)
        .await
        .unwrap_or(0);
    
    let replica_health = (replica_count as f64 / state.config.min_replica_count as f64).min(1.0);
    
    Ok(Json(RepoStats {
        total_size: repo.size,
        total_clones: 0,
        total_bandwidth: 0,
        replica_health,
    }))
}

// Pin/unpin repositories
pub async fn pin_repo(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.db
        .pin_repository(&repo_hash, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::OK)
}

pub async fn unpin_repo(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.db
        .unpin_repository(&repo_hash, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::OK)
}

pub async fn get_pinned_repos(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state.db
        .get_pinned_repositories(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(repos))
}
