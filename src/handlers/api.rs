// src/handlers/api.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::models::*;
use crate::services::replication::ReplicationService;
use crate::utils::hash::generate_repo_hash;
use crate::AppState;

// Repository endpoints
pub async fn create_repo(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRepoRequest>,
) -> Result<Json<CreateRepoResponse>, StatusCode> {
    let user_id = 1i64; // TODO: Get from authentication
    let repo_hash = generate_repo_hash(&payload.name, user_id);
    
    state.db
        .create_repository(&payload, user_id, &repo_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
   
 state.git_storage
        .init_repo(&repo_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateRepoResponse {
        repo_hash: repo_hash.clone(),
        message: format!("Repository created with hash: {}", repo_hash),
    }))
}

pub async fn get_repo(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Json<RepoMetadata>, StatusCode> {
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let nodes = state.db
        .list_repo_replicas(&repo_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let replica_count = nodes.len() as i64;
    let health_status = if replica_count >= 5 {
        "Excellent"
    } else if replica_count >= state.config.min_replica_count as i64 {
        "Good"
    } else {
        "Needs replication"
    }.to_string();
    
    Ok(Json(RepoMetadata {
        repo_hash: repo.repo_hash,
        name: repo.name,
        description: repo.description,
        size: repo.size,
        replica_count,
        nodes,
        health_status,
    }))
}

pub async fn delete_repo(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Check authentication and ownership
    
    state.db
        .delete_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_repo_nodes(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Json<Vec<NodeInfo>>, StatusCode> {
    let nodes = state.db
        .list_repo_replicas(&repo_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(nodes))
}

pub async fn request_replication(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Json<ReplicationResponse>, StatusCode> {
    let replication_service = ReplicationService::new(state.db.clone());
    
    match replication_service.trigger_replication(&repo_hash).await {
        Ok(target_node) => Ok(Json(ReplicationResponse {
            success: true,
            message: format!("Replication requested to node: {}", target_node),
        })),
        Err(e) => Ok(Json(ReplicationResponse {
            success: false,
            message: format!("Replication failed: {}", e),
        })),
    }
}

// Node endpoints
pub async fn register_node(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterNodeRequest>,
) -> Result<Json<Node>, StatusCode> {
    let node = state.db
        .register_node(&payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(node))
}

pub async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> Result<Json<Node>, StatusCode> {
    let node = state.db
        .get_node(&node_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(node))
}

pub async fn list_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Node>>, StatusCode> {
    let nodes = state.db
        .list_active_nodes(state.config.node_heartbeat_timeout_minutes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(nodes))
}

pub async fn node_heartbeat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NodeHeartbeat>,
) -> Result<StatusCode, StatusCode> {
    state.db
        .update_node_heartbeat(&payload.node_id, payload.storage_used)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Update replicas
    for repo_hash in &payload.hosted_repos {
        let _ = state.db.create_replica(repo_hash, &payload.node_id).await;
    }
    
    Ok(StatusCode::OK)
}

// Stats endpoint
pub async fn network_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NetworkStats>, StatusCode> {
    // TODO: Implement proper statistics aggregation
    let total_repos = 0; // Query count
    let total_nodes = state.db
        .list_active_nodes(state.config.node_heartbeat_timeout_minutes)
        .await
        .map(|n| n.len() as i64)
        .unwrap_or(0);
    
    Ok(Json(NetworkStats {
        total_repos,
        total_nodes,
        total_storage_used: 0,
        active_nodes: total_nodes,
        average_replica_count: 0.0,
    }))
}
