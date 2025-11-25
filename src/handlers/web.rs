// src/handlers/web.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use std::sync::Arc;

use crate::templates;
use crate::AppState;

async fn get_username(state: &Arc<AppState>, jar: &CookieJar) -> Option<String> {
    if let Some(session_id) = jar.get("session_id") {
        if let Some(session) = state.session_store.get_session(session_id.value()).await {
            return Some(session.username);
        }
    }
    None
}

pub async fn index(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Html<String> {
    let username = get_username(&state, &jar).await;
    Html(templates::index::render_with_user(username.as_deref()))
}

pub async fn explore(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, StatusCode> {
    let username = get_username(&state, &jar).await;
    let repos = state.db
        .list_public_repositories(50)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Html(templates::explore::render_with_user(&repos, username.as_deref())))
}

pub async fn view_repo(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(repo_hash): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let username = get_username(&state, &jar).await;
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let replica_count = state.db
        .get_replica_count(&repo_hash)
        .await
        .unwrap_or(0);
    
    let nodes = state.db
        .list_repo_replicas(&repo_hash)
        .await
        .unwrap_or_default();
    
    Ok(Html(templates::repo::render_with_user(&repo, replica_count, &nodes, username.as_deref())))
}

pub async fn dashboard(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, StatusCode> {
    let username = get_username(&state, &jar).await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Get user from database to get ID
    let user = state.db
        .get_user_by_username(&username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let repos = state.db
        .list_user_repositories(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Html(templates::dashboard::render_with_user(&repos, Some(&username))))
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub success: Option<String>,
}

pub async fn login_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(query): Query<LoginQuery>,
) -> Html<String> {
    let username = get_username(&state, &jar).await;
    Html(templates::login::render_with_user(query.success.as_deref(), username.as_deref()))
}

pub async fn signup_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Html<String> {
    let username = get_username(&state, &jar).await;
    Html(templates::signup::render_with_user(username.as_deref()))
}

pub async fn docs(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Html<String> {
    let username = get_username(&state, &jar).await;
    Html(templates::docs::render_with_user(username.as_deref()))
}

pub async fn about(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Html<String> {
    let username = get_username(&state, &jar).await;
    Html(templates::about::render_with_user(username.as_deref()))
}
