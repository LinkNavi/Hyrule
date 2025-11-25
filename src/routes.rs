// Hyrule/src/routes_complete.rs
// Complete routing with all Git HTTP endpoints

use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::handlers::{
    admin_web, api, api_complete, api_enhanced, git, git_http_complete, repo_browser, web,
    web_enhanced,
};
use crate::models::{NetworkStats, Repository};
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // ==================
        // GIT SMART HTTP PROTOCOL (RFC)
        // ==================
        // Info/refs endpoint for smart HTTP
        .route(
            "/git/:hash/info/refs",
            get(git_http_complete::git_info_refs),
        )
        // Upload pack (clone/fetch)
        .route(
            "/git/:hash/git-upload-pack",
            post(git_http_complete::git_upload_pack),
        )
        // Receive pack (push)
        .route(
            "/git/:hash/git-receive-pack",
            post(git_http_complete::git_receive_pack),
        )
        // Dumb HTTP fallback
        .route("/git/:hash/download", get(git_http_complete::dumb_clone))
        // ==================
        // WEB UI ROUTES
        // ==================
        .route("/", get(web::index))
        .route("/explore", get(web::explore))
        .route("/docs", get(web::docs))
        .route("/about", get(web::about))
        .route("/dashboard", get(web_enhanced::dashboard_enhanced))
        .route("/profile", get(web_enhanced::profile_page))
        .route("/search", get(web_enhanced::search_page))
        .route("/tags", get(web_enhanced::tags_page))
        .route("/starred", get(web_enhanced::starred_page))
        .route("/pinned", get(web_enhanced::pinned_page))
        .route("/repos/new", get(web_enhanced::create_repo_page))
        .route("/repos/new", post(web_enhanced::create_repo_submit))
        .route("/repos/action", post(web_enhanced::repo_action))
        .route("/repos/fork", post(web_enhanced::fork_repo_form))
        // Repository views
        .route("/r/:hash", get(web_enhanced::repo_enhanced))
        .route("/r/:hash/files", get(repo_browser::browse_files))
        .route("/r/:hash/files/*path", get(repo_browser::browse_directory))
        .route("/r/:hash/file/*path", get(repo_browser::view_file))
        .route("/r/:hash/commits", get(repo_browser::list_commits))
        .route(
            "/r/:hash/commit/:commit_hash",
            get(repo_browser::view_commit),
        )
        .route("/r/:hash/branches", get(repo_browser::list_branches))
        .route(
            "/r/:hash/clone",
            get(crate::handlers::clone_page::show_clone_page),
        )
        // Auth
        .route("/login", get(web::login_page))
        .route("/signup", get(web::signup_page))
        .route(
            "/login",
            post(crate::auth::auth_session::login_form_handler),
        )
        .route(
            "/signup",
            post(crate::auth::auth_session::signup_form_handler),
        )
        .route("/logout", post(crate::auth::auth_session::logout_handler))
        // Admin
        // ==================
        // API ROUTES
        // ==================
        // Auth API
        .route("/api/auth/login", post(api_enhanced::login))
        .route("/api/auth/signup", post(api_enhanced::signup))
        .route("/api/auth/profile", get(api_enhanced::get_profile))
        // Repository API
        .route("/api/repos", get(list_public_repos))
        .route("/api/repos/user", get(api_complete::list_user_repos))
        .route("/api/repos/search", get(api_enhanced::search_repos))
        .route("/api/repos/trending", get(get_trending_repos))
        .route("/api/repos/popular", get(get_popular_repos))
        .route("/api/repos", post(api_enhanced::create_repo_authenticated))
        .route("/api/repos/:hash", get(api_complete::get_repo_detailed))
        .route(
            "/api/repos/:hash",
            delete(api_complete::delete_repo_complete),
        )
        .route("/api/repos/:hash/fork", post(api_complete::fork_repo))
        .route("/api/repos/:hash/stats", get(api_enhanced::get_repo_stats))
        .route("/api/repos/:hash/nodes", get(api::get_repo_nodes))
        .route(
            "/api/repos/:hash/readme",
            get(api_complete::get_repo_readme),
        )
        .route("/api/repos/:hash/replicate", post(api::request_replication))
        // User interactions
        .route("/api/repos/:hash/star", post(api_complete::star_repo))
        .route("/api/repos/:hash/star", delete(api_complete::unstar_repo))
        .route("/api/repos/:hash/pin", post(api_enhanced::pin_repo))
        .route("/api/repos/:hash/unpin", delete(api_enhanced::unpin_repo))
        .route("/api/repos/pinned", get(api_enhanced::get_pinned_repos))
        .route("/api/repos/starred", get(get_starred_repos))
        // Tags
        .route("/api/repos/:hash/tags", post(api_complete::add_tags))
        .route("/api/repos/:hash/tags", get(get_repo_tags))
        .route("/api/tags", get(get_all_tags))
        .route("/api/tags/:tag/repos", get(api_complete::get_repos_by_tag))
        // Git operations (custom protocol)
        .route("/api/repos/:hash/objects/:id", get(git::download_object))
        .route("/api/repos/:hash/objects", post(git::upload_object))
        .route(
            "/api/repos/:hash/objects/batch",
            post(git::batch_upload_objects),
        )
        .route("/api/repos/:hash/objects", get(git::list_objects))
        .route("/api/repos/:hash/pack", get(git::get_packfile))
        .route("/api/repos/:hash/refs", post(git::update_ref))
        .route("/api/repos/:hash/refs/*ref_name", get(git::get_ref))
        // Node API
        .route("/api/nodes", get(api::list_nodes))
        .route("/api/nodes", post(api::register_node))
        .route("/api/nodes/:id", get(api::get_node))
        .route("/api/nodes/heartbeat", post(api::node_heartbeat))
        // Admin API
        .route("/admin", get(admin_web::admin_dashboard))
        .route("/admin/nodes", get(admin_web::admin_nodes_page))
        .route("/admin/repos", get(admin_web::admin_repos_page))
        .route("/admin/users", get(admin_web::admin_users_page))
        // Admin API routes
        .route("/api/admin/nodes", get(api_complete::admin_list_nodes))
        .route("/api/admin/health", get(api_complete::admin_system_health))
        .route(
            "/api/admin/replicate",
            post(api_complete::admin_trigger_replication),
        )
        .route("/api/admin/health-check", post(admin_trigger_health_check))
        // Stats
        .route("/api/stats", get(network_stats))
        .route("/api/health", get(health_check))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(axum::middleware::from_fn(
            crate::middleware::security::security_headers,
        ))
        .layer(DefaultBodyLimit::disable())
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

// Handler functions

pub async fn admin_trigger_health_check(
    State(state): State<Arc<AppState>>,
    _admin: crate::auth::AdminUser, // Add admin check
) -> Result<StatusCode, StatusCode> {
    let db = state.db.clone();
    let config = state.config.clone();

    tokio::spawn(async move {
        let health_monitor =
            crate::services::health::HealthMonitor::new(db, config.min_replica_count, 1);
        let _ = health_monitor.check_network_health().await;
    });

    Ok(StatusCode::ACCEPTED)
}
pub async fn list_public_repos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state
        .db
        .list_public_repositories(50)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(repos))
}

pub async fn get_trending_repos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state
        .db
        .get_trending_repos(20)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(repos))
}

pub async fn get_popular_repos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state
        .db
        .get_popular_repos(20)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(repos))
}

pub async fn get_starred_repos(
    State(state): State<Arc<AppState>>,
    user: crate::auth::AuthUser,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state
        .db
        .get_starred_repositories(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(repos))
}

pub async fn get_repo_tags(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let tags = state
        .db
        .get_repo_tags(&repo_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tags))
}

#[derive(Debug, Serialize)]
pub struct TagWithCount {
    pub tag: String,
    pub count: i64,
}

pub async fn get_all_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TagWithCount>>, StatusCode> {
    let tags = state
        .db
        .get_all_tags()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = tags
        .into_iter()
        .map(|(tag, count)| TagWithCount { tag, count })
        .collect();
    Ok(Json(result))
}

pub async fn network_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NetworkStats>, StatusCode> {
    let stats = state
        .db
        .get_network_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stats))
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthCheckResponse>, StatusCode> {
    let stats = state
        .db
        .get_network_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        total_repos: stats.total_repos,
        active_nodes: stats.active_nodes,
    }))
}

#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub total_repos: i64,
    pub active_nodes: i64,
}
