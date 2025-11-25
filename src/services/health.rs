// src/services/health.rs
use crate::db::Database;
use crate::services::replication::ReplicationService;
use std::time::Duration;
use tokio::time;
use tracing::{info, warn, error};

pub struct HealthMonitor {
    db: Database,
    min_replica_count: i32,
    check_interval_minutes: u64,
}

impl HealthMonitor {
    pub fn new(db: Database, min_replica_count: i32, check_interval_minutes: u64) -> Self {
        Self {
            db,
            min_replica_count,
            check_interval_minutes,
        }
    }
    
    /// Start the health monitoring loop
    pub async fn start(self) {
        let mut interval = time::interval(Duration::from_secs(self.check_interval_minutes * 60));
        
        info!("Health monitor started, checking every {} minutes", self.check_interval_minutes);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_network_health().await {
                error!("Health check failed: {}", e);
            }
        }
    }
    
    /// Perform a comprehensive health check of the network
    pub async fn check_network_health(&self) -> Result<(), String> {
        info!("Starting network health check...");
        
        // Check for unhealthy repositories
        let unhealthy_repos = self.db
            .get_unhealthy_repos(self.min_replica_count)
            .await
            .map_err(|e| format!("Failed to get unhealthy repos: {}", e))?;
        
        if !unhealthy_repos.is_empty() {
            warn!("Found {} repositories below minimum replica count", unhealthy_repos.len());
            
            // Trigger replication for unhealthy repos
            let replication_service = ReplicationService::new(self.db.clone());
            
            for repo_hash in unhealthy_repos.iter().take(10) {  // Process 10 at a time
                match replication_service.trigger_replication(repo_hash).await {
                    Ok(node_id) => {
                        info!("Triggered replication for {} to node {}", repo_hash, node_id);
                    }
                    Err(e) => {
                        warn!("Failed to replicate {}: {}", repo_hash, e);
                    }
                }
            }
        }
        
        // Clean up stale nodes
        let stale_count = self.cleanup_stale_nodes().await?;
        if stale_count > 0 {
            info!("Cleaned up {} stale nodes", stale_count);
        }
        
        // Log network statistics
        match self.db.get_network_stats().await {
            Ok(stats) => {
                info!(
                    "Network stats - Repos: {}, Nodes: {}, Avg replicas: {:.2}",
                    stats.total_repos, stats.total_nodes, stats.average_replica_count
                );
            }
            Err(e) => {
                warn!("Failed to get network stats: {}", e);
            }
        }
        
        info!("Health check completed");
        Ok(())
    }
    
    /// Remove nodes that haven't been seen recently
    async fn cleanup_stale_nodes(&self) -> Result<usize, String> {
        // This would delete nodes last seen more than 1 hour ago
        // Implementation would depend on your database structure
        Ok(0)  // Placeholder
    }
    
    /// Check if a specific repository needs replication
    pub async fn check_repo_health(&self, repo_hash: &str) -> Result<RepoHealth, String> {
        let replica_count = self.db
            .get_replica_count(repo_hash)
            .await
            .map_err(|e| format!("Failed to get replica count: {}", e))?;
        
        let status = if replica_count >= 5 {
            HealthStatus::Excellent
        } else if replica_count >= self.min_replica_count as i64 {
            HealthStatus::Good
        } else if replica_count > 0 {
            HealthStatus::NeedsReplication
        } else {
            HealthStatus::Critical
        };
        
        Ok(RepoHealth {
            repo_hash: repo_hash.to_string(),
            replica_count,
            min_replicas: self.min_replica_count,
            status,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RepoHealth {
    pub repo_hash: String,
    pub replica_count: i64,
    pub min_replicas: i32,
    pub status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Excellent,      // 5+ replicas
    Good,           // >= min_replicas
    NeedsReplication, // 1+ but < min_replicas
    Critical,       // 0 replicas
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HealthStatus::Excellent => write!(f, "Excellent"),
            HealthStatus::Good => write!(f, "Good"),
            HealthStatus::NeedsReplication => write!(f, "Needs Replication"),
            HealthStatus::Critical => write!(f, "Critical"),
        }
    }
}
