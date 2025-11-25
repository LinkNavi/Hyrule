// src/handlers/api_complete.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::auth::AdminUser;
use crate::auth::AuthUser;
use crate::models::*;
use crate::AppState;
// src/handlers/api_complete.rs
use pulldown_cmark::{Parser, html};

async fn check_repo_access(
    db: &crate::db::Database,
    repo_hash: &str,
    user_id: i64,
    require_ownership: bool,
) -> Result<crate::models::Repository, StatusCode> {
    let repo = db
        .get_repository(repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Private repos require authentication
    if repo.is_private != 0 {
        if require_ownership && repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
        // Even for non-ownership operations on private repos, verify user has access
        // For now, only owner can access private repos
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Public repos with ownership requirement
    if require_ownership && repo.owner_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(repo)
}

pub async fn get_repo_readme(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<String, StatusCode> {
    let repo_path = state.git_storage.repo_path(&repo_hash);
    
    // Try to find README file
    let readme_names = ["README.md", "README.txt", "README", "Readme.md", "readme.md"];
    
    for name in &readme_names {
        match read_file_from_git(&repo_path, "HEAD", name) {
            Ok(content) => {
                // If it's markdown, convert to HTML
                if name.ends_with(".md") {
                    let parser = Parser::new(&content);
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    return Ok(html_output);
                } else {
                    // Plain text - wrap in <pre>
                    return Ok(format!("<pre>{}</pre>", html_escape(&content)));
                }
            }
            Err(_) => continue,
        }
    }
    
    Err(StatusCode::NOT_FOUND)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
// Fork repository
#[derive(Debug, Deserialize)]
pub struct ForkRepoRequest {
    pub new_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ForkRepoResponse {
    pub original_hash: String,
    pub forked_hash: String,
    pub message: String,
}

pub async fn fork_repo(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
    Json(payload): Json<ForkRepoRequest>,
) -> Result<Json<ForkRepoResponse>, StatusCode> {
    // Get original repository
    let original_repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Generate new hash for fork
    let fork_name = payload.new_name.unwrap_or(format!("{}-fork", original_repo.name));
    let fork_hash = crate::utils::hash::generate_repo_hash(&fork_name, user.id);
    
    // Copy repository data
    let original_path = state.git_storage.repo_path(&repo_hash);
    let fork_path = state.git_storage.repo_path(&fork_hash);
    
    std::fs::create_dir_all(&fork_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Copy all git objects
    copy_dir_recursive(&original_path, &fork_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create database entry
    let fork_request = CreateRepoRequest {
        name: fork_name.clone(),
        description: payload.description.or(Some(format!("Fork of {}", original_repo.name))),
        storage_tier: "free".to_string(),
        is_private: original_repo.is_private != 0,
    };
    
    state.db
        .create_repository(&fork_request, user.id, &fork_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(ForkRepoResponse {
        original_hash: repo_hash,
        forked_hash: fork_hash.clone(),
        message: format!("Repository forked successfully as {}", fork_name),
    }))
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    
    Ok(())
}

// Delete repository (proper implementation)
pub async fn delete_repo_complete(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    if repo.owner_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Delete from storage first (ignore errors as files might not exist)
    let _ = state.git_storage.delete_repo(&repo_hash);
    
    // Delete from database using the complete method
    state.db
        .delete_repository_complete(&repo_hash)
        .await
        .map_err(|e| {
            eprintln!("Error deleting repository: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

// List user's repositories
pub async fn list_user_repos(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state.db
        .list_user_repositories(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(repos))
}

// Get detailed repository info
#[derive(Debug, Serialize)]
pub struct DetailedRepoInfo {
    pub repo: Repository,
    pub owner_username: String,
    pub replica_count: i64,
    pub nodes: Vec<NodeInfo>,
    pub health_status: String,
    pub total_commits: i64,
    pub total_bandwidth: i64,
    pub stars: i64,
}

pub async fn get_repo_detailed(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Json<DetailedRepoInfo>, StatusCode> {
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let owner = state.db
        .get_user_by_id(repo.owner_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let nodes = state.db
        .list_repo_replicas(&repo_hash)
        .await
        .unwrap_or_default();
    
    let replica_count = nodes.len() as i64;
    
    let health_status = if replica_count >= 5 {
        "Excellent"
    } else if replica_count >= state.config.min_replica_count as i64 {
        "Good"
    } else {
        "Needs replication"
    }.to_string();
    
    // Get star count
    let stars = state.db
        .get_repo_star_count(&repo_hash)
        .await
        .unwrap_or(0);
    
    Ok(Json(DetailedRepoInfo {
        repo,
        owner_username: owner.username,
        replica_count,
        nodes,
        health_status,
        total_commits: 0, // Would need to parse git log
        total_bandwidth: 0, // Would need bandwidth tracking
        stars,
    }))
}

// Star/unstar repository
pub async fn star_repo(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.db
        .star_repository(&repo_hash, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::OK)
}

pub async fn unstar_repo(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.db
        .unstar_repository(&repo_hash, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::OK)
}

// Add tags to repository
#[derive(Debug, Deserialize)]
pub struct AddTagsRequest {
    pub tags: Vec<String>,
}

pub async fn add_tags(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
    Json(payload): Json<AddTagsRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    if repo.owner_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    for tag in payload.tags {
        let _ = state.db.add_repo_tag(&repo_hash, &tag).await;
    }
    
    Ok(StatusCode::OK)
}

// Get repositories by tag
pub async fn get_repos_by_tag(
    State(state): State<Arc<AppState>>,
    Path(tag): Path<String>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let repos = state.db
        .get_repos_by_tag(&tag)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(repos))
}

// Admin: Get all nodes with details
#[derive(Debug, Serialize)]
pub struct AdminNodeInfo {
    pub node: Node,
    pub hosted_repos: i64,
    pub total_bandwidth: i64,
}


pub async fn admin_list_nodes(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser, // Changed from user: AuthUser
) -> Result<Json<Vec<AdminNodeInfo>>, StatusCode> {
    // No need to check admin status - extractor handles it
    
    let nodes = state.db
        .list_active_nodes(state.config.node_heartbeat_timeout_minutes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut admin_nodes = Vec::new();
    
    for node in nodes {
        let hosted_repos = state.db
            .count_node_repos(&node.node_id)
            .await
            .unwrap_or(0);
        
        admin_nodes.push(AdminNodeInfo {
            node,
            hosted_repos,
            total_bandwidth: 0,
        });
    }
    
    Ok(Json(admin_nodes))
}

pub async fn admin_system_health(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser, // Changed
) -> Result<Json<SystemHealth>, StatusCode> {
    let stats = state.db
        .get_network_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let unhealthy_repos = state.db
        .get_unhealthy_repos(state.config.min_replica_count)
        .await
        .map(|repos| repos.len() as i64)
        .unwrap_or(0);
    
    Ok(Json(SystemHealth {
        total_repos: stats.total_repos,
        total_nodes: stats.total_nodes,
        active_nodes: stats.active_nodes,
        total_storage_used: stats.total_storage_used,
        unhealthy_repos,
        average_replica_count: stats.average_replica_count,
        total_bandwidth_24h: 0,
    }))
}

pub async fn admin_trigger_replication(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser, // Changed
    Json(payload): Json<ManualReplicationRequest>,
) -> Result<Json<ReplicationResponse>, StatusCode> {
    let replication_service = crate::services::replication::ReplicationService::new(state.db.clone());
    
    match replication_service.trigger_replication(&payload.repo_hash).await {
        Ok(target_node) => Ok(Json(ReplicationResponse {
            success: true,
            message: format!("Replication triggered to node: {}", target_node),
        })),
        Err(e) => Ok(Json(ReplicationResponse {
            success: false,
            message: format!("Replication failed: {}", e),
        })),
    }
}

// Admin: System health overview
#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub total_repos: i64,
    pub total_nodes: i64,
    pub active_nodes: i64,
    pub total_storage_used: i64,
    pub unhealthy_repos: i64,
    pub average_replica_count: f64,
    pub total_bandwidth_24h: i64,
}



// Admin: Manually trigger replication
#[derive(Debug, Deserialize)]
pub struct ManualReplicationRequest {
    pub repo_hash: String,
    pub target_node_id: Option<String>,
}





fn read_file_from_git(
    repo_path: &std::path::Path,
    ref_name: &str,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let path_arg = format!("{}:{}", ref_name, file_path);
    
    let output = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("show")
        .arg(&path_arg)
        .output()?;
    
    if !output.status.success() {
        return Err("File not found".into());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
