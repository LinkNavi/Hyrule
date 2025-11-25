// src/services/replication.rs
use crate::db::Database;

pub struct ReplicationService {
    db: Database,
}

impl ReplicationService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    /// Trigger replication for a repository to a new node
    pub async fn trigger_replication(&self, repo_hash: &str) -> Result<String, String> {
        // Get current replicas
        let current_replicas = self.db
            .list_repo_replicas(repo_hash)
            .await
            .map_err(|e| format!("Failed to list replicas: {}", e))?;
        
        // Get available nodes
        let available_nodes = self.db
            .list_active_nodes(10)
            .await
            .map_err(|e| format!("Failed to list nodes: {}", e))?;
        
        // Find node not already hosting this repo
        let current_node_ids: Vec<String> = current_replicas.iter()
            .map(|n| n.node_id.clone())
            .collect();
        
        let target_node = available_nodes
            .into_iter()
            .find(|node| !current_node_ids.contains(&node.node_id))
            .ok_or_else(|| "No available nodes for replication".to_string())?;
        
        // Create replica record
        self.db
            .create_replica(repo_hash, &target_node.node_id)
            .await
            .map_err(|e| format!("Failed to create replica: {}", e))?;
        
        // TODO: Actually trigger data transfer to node
        // This would involve:
        // 1. Packaging the git objects
        // 2. Sending to target node via HTTP/gRPC
        // 3. Verifying successful transfer
        
        Ok(target_node.node_id)
    }
    
    /// Check health of all repositories and trigger replication if needed
    pub async fn health_check(&self, _min_replicas: i32) -> Result<Vec<String>, String> {
        // TODO: Implement health check that:
        // 1. Queries all repos with replica count < min_replicas
        // 2. Triggers replication for each
        // 3. Returns list of repos that needed replication
        
        Ok(vec![])
    }
}