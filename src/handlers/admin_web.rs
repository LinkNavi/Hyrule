// Hyrule/src/handlers/admin_web.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Redirect},
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;

use crate::templates;
use crate::AppState;

// Helper to check admin access - MUST be used by all admin endpoints
async fn check_admin_access(
    state: &Arc<AppState>,
    jar: &CookieJar,
) -> Result<(i64, String), (StatusCode, Html<String>)> {
    let session_id = jar
        .get("session_id")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    let session = state
        .session_store
        .get_session(session_id.value())
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    // Check if user is admin or Link
    let user = state
        .db
        .get_user_by_id(session.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(error_page("Failed to verify admin")),
            )
        })?;

    if user.is_admin == 0 {
        return Err((StatusCode::FORBIDDEN, Html(access_denied())));
    }

    Ok((session.user_id, session.username))
}

fn redirect_to_login() -> String {
    crate::templates::render_page(
        "Login Required",
        r#"<div class="section">
            <h1>🔒 Login Required</h1>
            <p>You must be logged in to access the admin panel.</p>
            <a href="/login" class="btn btn-primary">Go to Login</a>
        </div>"#,
    )
}

fn access_denied() -> String {
    crate::templates::render_page(
        "Access Denied",
        r#"<div class="section">
            <h1>⛔ Access Denied</h1>
            <p>You do not have permission to access the admin panel.</p>
            <p>Only administrators (admin, Link) can access this area.</p>
            <a href="/dashboard" class="btn btn-secondary">Back to Dashboard</a>
        </div>"#,
    )
}

pub async fn admin_dashboard(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (_user_id, _username) = check_admin_access(&state, &jar).await?;

    let stats = state.db.get_network_stats().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(error_page("Failed to load stats")),
        )
    })?;

    let unhealthy_repos = state
        .db
        .get_unhealthy_repos(state.config.min_replica_count)
        .await
        .map(|repos| repos.len() as i64)
        .unwrap_or(0);

    let bandwidth_24h = state.db.get_bandwidth_24h(None).await.unwrap_or(0);

    Ok(Html(templates::admin::render_dashboard(
        stats.total_repos,
        stats.total_nodes,
        unhealthy_repos,
        stats.total_storage_used,
        bandwidth_24h,
    )))
}

pub async fn admin_nodes_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (_user_id, _username) = check_admin_access(&state, &jar).await?;

    let nodes = state
        .db
        .list_active_nodes(state.config.node_heartbeat_timeout_minutes)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(error_page("Failed to load nodes")),
            )
        })?;

    let mut nodes_with_counts = Vec::new();

    for node in nodes {
        let repo_count = state.db.count_node_repos(&node.node_id).await.unwrap_or(0);

        nodes_with_counts.push((node, repo_count));
    }

    Ok(Html(templates::admin::render_nodes(&nodes_with_counts)))
}

pub async fn admin_repos_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (_user_id, _username) = check_admin_access(&state, &jar).await?;

    let repos = state.db.list_public_repositories(100).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(error_page("Failed to load repositories")),
        )
    })?;

    Ok(Html(templates::admin::render_repos(&repos)))
}

pub async fn admin_users_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (_user_id, _username) = check_admin_access(&state, &jar).await?;

    Ok(Html(templates::render_page(
        "User Management",
        "<h1>User Management</h1><p>Coming soon...</p>",
    )))
}

fn error_page(message: &str) -> String {
    crate::templates::render_page(
        "Error",
        &format!(
            r#"<div class="section">
                <h1>Error</h1>
                <p>{}</p>
                <a href="/admin" class="btn btn-secondary">Back to Admin</a>
            </div>"#,
            message
        ),
    )
}
