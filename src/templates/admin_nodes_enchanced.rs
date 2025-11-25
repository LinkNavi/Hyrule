// src/handlers/admin_nodes_enhanced.rs
use axum::{
    extract::{Path, State, Form},
    http::StatusCode,
    response::{Html, Redirect},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::templates;
use crate::AppState;

// Enhanced node view with detailed statistics
pub async fn admin_nodes_enhanced(
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let nodes = state.db
        .list_active_nodes(state.config.node_heartbeat_timeout_minutes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut nodes_with_stats = Vec::new();
    
    for node in nodes {
        let repo_count = state.db
            .count_node_repos(&node.node_id)
            .await
            .unwrap_or(0);
        
        // Calculate health metrics
        let storage_usage_percent = if node.storage_capacity > 0 {
            (node.storage_used as f64 / node.storage_capacity as f64 * 100.0) as i32
        } else {
            0
        };
        
        let is_healthy = node.reputation_score >= 80 && storage_usage_percent < 90;
        
        nodes_with_stats.push(NodeStats {
            node,
            repo_count,
            storage_usage_percent,
            is_healthy,
        });
    }
    
    // Sort by health status, then reputation
    nodes_with_stats.sort_by(|a, b| {
        b.is_healthy.cmp(&a.is_healthy)
            .then(b.node.reputation_score.cmp(&a.node.reputation_score))
    });
    
    // Get network-wide stats
    let total_nodes = nodes_with_stats.len();
    let healthy_nodes = nodes_with_stats.iter().filter(|n| n.is_healthy).count();
    let total_storage: i64 = nodes_with_stats.iter().map(|n| n.node.storage_capacity).sum();
    let used_storage: i64 = nodes_with_stats.iter().map(|n| n.node.storage_used).sum();
    
    Ok(Html(templates::admin::nodes_enhanced::render(
        &nodes_with_stats,
        total_nodes,
        healthy_nodes,
        total_storage,
        used_storage,
    )))
}

#[derive(Debug)]
pub struct NodeStats {
    pub node: crate::models::Node,
    pub repo_count: i64,
    pub storage_usage_percent: i32,
    pub is_healthy: bool,
}

// Node actions
#[derive(Debug, Deserialize)]
pub struct NodeAction {
    pub action: String,
    pub node_id: String,
}

pub async fn node_action(
    State(state): State<Arc<AppState>>,
    Form(form): Form<NodeAction>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    match form.action.as_str() {
        "ping" => {
            // Try to ping the node
            let node = state.db.get_node(&form.node_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, Html(error_page("Node not found"))))?;
            
            // Simple HTTP ping
            let url = format!("http://{}:{}/api/health", node.address, node.port);
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Html(error_page("Failed to create HTTP client"))))?;
            
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    // Update last_seen
                    let _ = state.db.update_node_heartbeat(&form.node_id, node.storage_used).await;
                }
                _ => {
                    // Node is unreachable
                }
            }
        }
        "remove" => {
            // Remove node and all its replicas
            // First get all replicas
            let replicas = state.db.list_repo_replicas(&form.node_id)
                .await
                .unwrap_or_default();
            
            // Delete all replicas
            for replica_node in replicas {
                let _ = state.db.delete_replica(&replica_node.node_id, &form.node_id).await;
            }
            
            // Delete node (would need to add this method to db)
            // let _ = state.db.delete_node(&form.node_id).await;
        }
        "ban" => {
            // Set reputation to 0 (would need to add this method)
            // let _ = state.db.update_node_reputation(&form.node_id, 0).await;
        }
        "trust" => {
            // Set reputation to 100
            // let _ = state.db.update_node_reputation(&form.node_id, 100).await;
        }
        _ => return Err((StatusCode::BAD_REQUEST, Html(error_page("Invalid action")))),
    }
    
    Ok(Redirect::to("/admin/nodes"))
}

fn error_page(message: &str) -> String {
    crate::templates::render_page(
        "Error",
        &format!(
            r#"<div class="section">
                <h1>Error</h1>
                <p>{}</p>
                <a href="/admin/nodes" class="btn btn-secondary">Back to Nodes</a>
            </div>"#,
            message
        ),
    )
}

// Add node statistics endpoint
pub async fn node_details(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let node = state.db
        .get_node(&node_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let repos = state.db
        .count_node_repos(&node_id)
        .await
        .unwrap_or(0);
    
    // Get list of repositories hosted by this node
    let hosted_repos = state.db
        .list_repo_replicas(&node_id)
        .await
        .unwrap_or_default();
    
    Ok(Html(templates::admin::node_details::render(&node, repos, &hosted_repos)))
}
