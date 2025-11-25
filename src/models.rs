
// src/models.rs
use serde::{Deserialize, Serialize};

pub const MAX_USERNAME_LEN: usize = 32;
pub const MIN_USERNAME_LEN: usize = 3;
pub const MAX_REPO_NAME_LEN: usize = 64;
pub const MIN_REPO_NAME_LEN: usize = 3;
pub const MAX_DESCRIPTION_LEN: usize = 500;
pub const MAX_SEARCH_QUERY_LEN: usize = 100;

// Database models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub is_admin: i32,
    pub password_hash: String,
    pub public_key: String,
    pub storage_quota: i64,
    pub storage_used: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Repository {
    pub repo_hash: String,
    pub owner_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub size: i64,
    pub storage_tier: String,
    pub is_private: i64,
    pub created_at: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Node {
    pub node_id: String,
    pub address: String,
    pub port: i32,
    pub last_seen: String,
    pub reputation_score: i32,
    pub storage_capacity: i64,
    pub storage_used: i64,
    pub is_anchor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Replica {
    pub repo_hash: String,
    pub node_id: String,
    pub created_at: String,
    pub last_verified: Option<String>,
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub public_key: String,
    pub storage_quota: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub storage_tier: String,
    pub is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateRepoResponse {
    pub repo_hash: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterNodeRequest {
    pub node_id: String,
    pub address: String,
    pub port: i32,
    pub storage_capacity: i64,
    pub is_anchor: bool,
}

#[derive(Debug, Deserialize)]
pub struct NodeHeartbeat {
    pub node_id: String,
    pub storage_used: i64,
    pub hosted_repos: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RepoMetadata {
    pub repo_hash: String,
    pub name: String,
    pub description: Option<String>,
    pub size: i64,
    pub replica_count: i64,
    pub nodes: Vec<NodeInfo>,
    pub health_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub port: i32,
    pub is_anchor: bool,
}

#[derive(Debug, Serialize)]
pub struct ReplicationRequest {
    pub repo_hash: String,
    pub target_node_id: String,
}

#[derive(Debug, Serialize)]
pub struct ReplicationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct NetworkStats {
    pub total_repos: i64,
    pub total_nodes: i64,
    pub total_storage_used: i64,
    pub active_nodes: i64,
    pub average_replica_count: f64,
}

// At the end of the file
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ActivityLogEntry {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}
