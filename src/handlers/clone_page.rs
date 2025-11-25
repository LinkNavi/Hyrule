// Hyrule/src/handlers/clone_page.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;

use crate::AppState;
use crate::templates;

async fn get_session_user_id(
    state: &Arc<AppState>,
    jar: &CookieJar,
) -> Option<i64> {
    if let Some(session_id) = jar.get("session_id") {
        if let Some(session) = state.session_store.get_session(session_id.value()).await {
            return Some(session.user_id);
        }
    }
    None
}

pub async fn show_clone_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(repo_hash): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar).await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    let server_url = format!("http://{}:{}", 
        state.config.host, 
        state.config.port
    );
    
    Ok(Html(templates::clone_page::render(&repo, &server_url)))
}
