// src/db.rs
use crate::models::*;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn star_repository(&self, repo_hash: &str, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO repo_stars (user_id, repo_hash, starred_at)
             VALUES (?, ?, datetime('now'))
             ON CONFLICT(user_id, repo_hash) DO NOTHING",
        )
        .bind(user_id)
        .bind(repo_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Unstar repository
    pub async fn unstar_repository(
        &self,
        repo_hash: &str,
        user_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM repo_stars WHERE user_id = ? AND repo_hash = ?")
            .bind(user_id)
            .bind(repo_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
 pub async fn has_pinned(&self, repo_hash: &str, user_id: i64) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pins WHERE repo_hash = ? AND user_id = ?",
        )
        .bind(repo_hash)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }
    // Get star count for repository
    pub async fn get_repo_star_count(&self, repo_hash: &str) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM repo_stars WHERE repo_hash = ?")
            .bind(repo_hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    // Check if user starred repo
    pub async fn has_starred(&self, repo_hash: &str, user_id: i64) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM repo_stars WHERE repo_hash = ? AND user_id = ?",
        )
        .bind(repo_hash)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    // Add tag to repository
    pub async fn add_repo_tag(&self, repo_hash: &str, tag: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO repo_tags (repo_hash, tag)
             VALUES (?, ?)
             ON CONFLICT(repo_hash, tag) DO NOTHING",
        )
        .bind(repo_hash)
        .bind(tag.to_lowercase())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Get tags for repository
    pub async fn get_repo_tags(&self, repo_hash: &str) -> Result<Vec<String>, sqlx::Error> {
        let tags = sqlx::query_scalar::<_, String>(
            "SELECT tag FROM repo_tags WHERE repo_hash = ? ORDER BY tag",
        )
        .bind(repo_hash)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    // Get repositories by tag
    pub async fn get_repos_by_tag(&self, tag: &str) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT r.* FROM repositories r
             JOIN repo_tags t ON r.repo_hash = t.repo_hash
             WHERE t.tag = ? AND r.is_private = 0
             ORDER BY r.last_updated DESC",
        )
        .bind(tag.to_lowercase())
        .fetch_all(&self.pool)
        .await
    }

    // Get all tags with counts
    pub async fn get_all_tags(&self) -> Result<Vec<(String, i64)>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT tag, COUNT(*) as count
             FROM repo_tags
             GROUP BY tag
             ORDER BY count DESC, tag ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let tag: String = row.try_get("tag").ok()?;
                let count: i64 = row.try_get("count").ok()?;
                Some((tag, count))
            })
            .collect())
    }

    // Count repos hosted by a node
    pub async fn count_node_repos(&self, node_id: &str) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM replicas WHERE node_id = ?")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    // Get user's starred repositories
    pub async fn get_starred_repositories(
        &self,
        user_id: i64,
    ) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT r.* FROM repositories r
             JOIN repo_stars s ON r.repo_hash = s.repo_hash
             WHERE s.user_id = ?
             ORDER BY s.starred_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    // Track bandwidth usage
    pub async fn log_bandwidth(
        &self,
        repo_hash: &str,
        bytes: i64,
        operation: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO repo_access_log (repo_hash, access_type, bytes_transferred, timestamp)
             VALUES (?, ?, ?, datetime('now'))",
        )
        .bind(repo_hash)
        .bind(operation)
        .bind(bytes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Get bandwidth for last 24 hours
    pub async fn get_bandwidth_24h(&self, repo_hash: Option<&str>) -> Result<i64, sqlx::Error> {
        let query = if let Some(hash) = repo_hash {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(bytes_transferred), 0)
                 FROM repo_access_log
                 WHERE repo_hash = ?
                 AND datetime(timestamp) > datetime('now', '-1 day')",
            )
            .bind(hash)
        } else {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(bytes_transferred), 0)
                 FROM repo_access_log
                 WHERE datetime(timestamp) > datetime('now', '-1 day')",
            )
        };

        query.fetch_one(&self.pool).await
    }

    // Get most popular repositories by clone count
    pub async fn get_popular_repos(&self, limit: i64) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT r.* FROM repositories r
             LEFT JOIN repo_access_log l ON r.repo_hash = l.repo_hash
             WHERE r.is_private = 0
             GROUP BY r.repo_hash
             ORDER BY COUNT(l.id) DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // Search repositories with filters
    pub async fn search_repos_advanced(
        &self,
        query: &str,
        tags: Option<Vec<String>>,
        limit: i64,
    ) -> Result<Vec<Repository>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        if let Some(tag_list) = tags {
            if tag_list.is_empty() {
                return self.search_repositories(query, limit).await;
            }

            // Build IN clause for tags
            let placeholders = tag_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT DISTINCT r.* FROM repositories r
                 JOIN repo_tags t ON r.repo_hash = t.repo_hash
                 WHERE (r.name LIKE ? OR r.description LIKE ?)
                 AND r.is_private = 0
                 AND t.tag IN ({})
                 ORDER BY r.last_updated DESC
                 LIMIT ?",
                placeholders
            );

            let mut query_builder = sqlx::query_as::<_, Repository>(&sql)
                .bind(&search_pattern)
                .bind(&search_pattern);

            for tag in tag_list {
                query_builder = query_builder.bind(tag.to_lowercase());
            }

            query_builder.bind(limit).fetch_all(&self.pool).await
        } else {
            self.search_repositories(query, limit).await
        }
    }

    // Get user activity log
    pub async fn get_user_activity(
        &self,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<ActivityLogEntry>, sqlx::Error> {
        sqlx::query_as::<_, ActivityLogEntry>(
            "SELECT * FROM activity_log
             WHERE user_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // Log user activity
    pub async fn log_activity(
        &self,
        user_id: i64,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        details: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO activity_log (user_id, action, resource_type, resource_id, details, ip_address)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(details)
        .bind(ip_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    // User operations
    pub async fn create_user(&self, user: &CreateUserRequest) -> Result<User, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO users (username, email, password_hash, public_key, storage_quota)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.public_key)
        .bind(user.storage_quota)
        .execute(&self.pool)
        .await?;

        self.get_user_by_id(result.last_insert_rowid()).await
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
    }

    // Repository operations
    pub async fn create_repository(
        &self,
        repo: &CreateRepoRequest,
        owner_id: i64,
        repo_hash: &str,
    ) -> Result<Repository, sqlx::Error> {
        let is_private = if repo.is_private { 1 } else { 0 };

        sqlx::query(
            "INSERT INTO repositories (repo_hash, owner_id, name, description, storage_tier, is_private)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(repo_hash)
        .bind(owner_id)
        .bind(&repo.name)
        .bind(&repo.description)
        .bind(&repo.storage_tier)
        .bind(is_private)
        .execute(&self.pool)
        .await?;

        self.get_repository(repo_hash).await
    }

    pub async fn get_repository(&self, repo_hash: &str) -> Result<Repository, sqlx::Error> {
        sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE repo_hash = ?")
            .bind(repo_hash)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_public_repositories(
        &self,
        limit: i64,
    ) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE is_private = 0 ORDER BY last_updated DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_user_repositories(
        &self,
        user_id: i64,
    ) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE owner_id = ? ORDER BY last_updated DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_repository_size(
        &self,
        repo_hash: &str,
        size: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE repositories SET size = ?, last_updated = datetime('now') WHERE repo_hash = ?",
        )
        .bind(size)
        .bind(repo_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_repository(&self, repo_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM repositories WHERE repo_hash = ?")
            .bind(repo_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Node operations
    pub async fn register_node(&self, node: &RegisterNodeRequest) -> Result<Node, sqlx::Error> {
        let is_anchor = if node.is_anchor { 1 } else { 0 };

        sqlx::query(
            "INSERT INTO nodes (node_id, address, port, storage_capacity, is_anchor)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(node_id) DO UPDATE SET
                address = excluded.address,
                port = excluded.port,
                storage_capacity = excluded.storage_capacity,
                last_seen = datetime('now')",
        )
        .bind(&node.node_id)
        .bind(&node.address)
        .bind(node.port)
        .bind(node.storage_capacity)
        .bind(is_anchor)
        .execute(&self.pool)
        .await?;

        self.get_node(&node.node_id).await
    }

    pub async fn get_node(&self, node_id: &str) -> Result<Node, sqlx::Error> {
        sqlx::query_as::<_, Node>("SELECT * FROM nodes WHERE node_id = ?")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_active_nodes(&self, timeout_minutes: i32) -> Result<Vec<Node>, sqlx::Error> {
        sqlx::query_as::<_, Node>(
            "SELECT * FROM nodes
             WHERE datetime(last_seen) > datetime('now', '-' || ? || ' minutes')
             ORDER BY reputation_score DESC",
        )
        .bind(timeout_minutes)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_node_heartbeat(
        &self,
        node_id: &str,
        storage_used: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE nodes SET last_seen = datetime('now'), storage_used = ? WHERE node_id = ?",
        )
        .bind(storage_used)
        .bind(node_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Replica operations
    pub async fn create_replica(
        &self,
        repo_hash: &str,
        node_id: &str,
    ) -> Result<Replica, sqlx::Error> {
        sqlx::query(
            "INSERT INTO replicas (repo_hash, node_id)
             VALUES (?, ?)
             ON CONFLICT(repo_hash, node_id) DO UPDATE SET
                last_verified = datetime('now')",
        )
        .bind(repo_hash)
        .bind(node_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Replica>("SELECT * FROM replicas WHERE repo_hash = ? AND node_id = ?")
            .bind(repo_hash)
            .bind(node_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_repo_replicas(&self, repo_hash: &str) -> Result<Vec<NodeInfo>, sqlx::Error> {
        let nodes = sqlx::query(
            "SELECT n.node_id, n.address, n.port, n.is_anchor
             FROM replicas r
             JOIN nodes n ON r.node_id = n.node_id
             WHERE r.repo_hash = ?
             AND datetime(n.last_seen) > datetime('now', '-5 minutes')",
        )
        .bind(repo_hash)
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes
            .into_iter()
            .map(|row| NodeInfo {
                node_id: row.try_get("node_id").unwrap_or_default(),
                address: row.try_get("address").unwrap_or_default(),
                port: row.try_get("port").unwrap_or_default(),
                is_anchor: row.try_get::<i64, _>("is_anchor").unwrap_or(0) != 0,
            })
            .collect())
    }

    pub async fn get_replica_count(&self, repo_hash: &str) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM replicas WHERE repo_hash = ?")
            .bind(repo_hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    pub async fn delete_replica(&self, repo_hash: &str, node_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM replicas WHERE repo_hash = ? AND node_id = ?")
            .bind(repo_hash)
            .bind(node_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Enhanced methods from db_enhanced

    // Search repositories by name or description
    pub async fn search_repositories(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Repository>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories 
             WHERE (name LIKE ? OR description LIKE ?)
             AND is_private = 0
             ORDER BY 
                CASE 
                    WHEN name LIKE ? THEN 1
                    ELSE 2
                END,
                last_updated DESC
             LIMIT ?",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(format!("{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // Pin repository
    pub async fn pin_repository(&self, repo_hash: &str, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO pins (user_id, repo_hash, pinned_at)
             VALUES (?, ?, datetime('now'))
             ON CONFLICT(user_id, repo_hash) DO NOTHING",
        )
        .bind(user_id)
        .bind(repo_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Unpin repository
    pub async fn unpin_repository(&self, repo_hash: &str, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pins WHERE user_id = ? AND repo_hash = ?")
            .bind(user_id)
            .bind(repo_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Get pinned repositories
    pub async fn get_pinned_repositories(
        &self,
        user_id: i64,
    ) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT r.* FROM repositories r
             JOIN pins p ON r.repo_hash = p.repo_hash
             WHERE p.user_id = ?
             ORDER BY p.pinned_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    // Get network-wide statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats, sqlx::Error> {
        let total_repos: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM repositories")
            .fetch_one(&self.pool)
            .await?;

        let total_nodes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM nodes 
             WHERE datetime(last_seen) > datetime('now', '-5 minutes')",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_storage: i64 =
            sqlx::query_scalar("SELECT COALESCE(SUM(size), 0) FROM repositories")
                .fetch_one(&self.pool)
                .await?;

        let avg_replicas: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(replica_count), 0) FROM (
                SELECT COUNT(*) as replica_count
                FROM replicas
                GROUP BY repo_hash
            )",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(NetworkStats {
            total_repos,
            total_nodes,
            total_storage_used: total_storage,
            active_nodes: total_nodes,
            average_replica_count: avg_replicas,
        })
    }

    // Get trending repositories
    pub async fn get_trending_repos(&self, limit: i64) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories
             WHERE is_private = 0
             ORDER BY last_updated DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
// Delete repository with all related data
pub async fn delete_repository_complete(&self, repo_hash: &str) -> Result<(), sqlx::Error> {
    // Start a transaction to ensure atomicity
    let mut tx = self.pool.begin().await?;
    
    // Delete in order to avoid foreign key constraint violations
    
    // 1. Delete replicas
    sqlx::query("DELETE FROM replicas WHERE repo_hash = ?")
        .bind(repo_hash)
        .execute(&mut *tx)
        .await?;
    
    // 2. Delete pins
    sqlx::query("DELETE FROM pins WHERE repo_hash = ?")
        .bind(repo_hash)
        .execute(&mut *tx)
        .await?;
    
    // 3. Delete stars
    sqlx::query("DELETE FROM repo_stars WHERE repo_hash = ?")
        .bind(repo_hash)
        .execute(&mut *tx)
        .await?;
    
    // 4. Delete tags
    sqlx::query("DELETE FROM repo_tags WHERE repo_hash = ?")
        .bind(repo_hash)
        .execute(&mut *tx)
        .await?;
    
    // 5. Delete access logs
    sqlx::query("DELETE FROM repo_access_log WHERE repo_hash = ?")
        .bind(repo_hash)
        .execute(&mut *tx)
        .await?;
    
    // 6. Finally delete the repository itself
    sqlx::query("DELETE FROM repositories WHERE repo_hash = ?")
        .bind(repo_hash)
        .execute(&mut *tx)
        .await?;
    
    // Commit the transaction
    tx.commit().await?;
    
    Ok(())
}
    // Get unhealthy repositories (below minimum replica count)
    pub async fn get_unhealthy_repos(&self, min_replicas: i32) -> Result<Vec<String>, sqlx::Error> {
        let repos = sqlx::query(
            "SELECT r.repo_hash
             FROM repositories r
             LEFT JOIN (
                 SELECT repo_hash, COUNT(*) as count
                 FROM replicas
                 GROUP BY repo_hash
             ) rc ON r.repo_hash = rc.repo_hash
             WHERE COALESCE(rc.count, 0) < ?",
        )
        .bind(min_replicas)
        .fetch_all(&self.pool)
        .await?;

        Ok(repos
            .into_iter()
            .filter_map(|row| row.try_get("repo_hash").ok())
            .collect())
    }
}

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
