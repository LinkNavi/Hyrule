// src/templates/admin_enhanced.rs
use super::render_page;
use crate::models::Node;

pub mod nodes_enhanced {
    use super::*;
    
    pub fn render(
        nodes_with_stats: &[crate::handlers::nodes_enhanced::NodeStats],
        total_nodes: usize,
        healthy_nodes: usize,
        total_storage: i64,
        used_storage: i64,
    ) -> String {
        let nodes_html = nodes_with_stats.iter().map(|stats| {
            let health_class = if stats.is_healthy { "node-healthy" } else { "node-warning" };
            
            format!(
                r#"
            <div class="admin-node-card {}">
                <div class="node-header">
                    <div class="node-id-badge"><code>{}</code></div>
                    <div class="node-status">{}</div>
                </div>
                <div class="node-details">
                    <div class="detail-row">
                        <span class="detail-label">Address:</span>
                        <span class="detail-value">{}:{}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Hosted Repos:</span>
                        <span class="detail-value">{}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Storage:</span>
                        <span class="detail-value">{}%</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Reputation:</span>
                        <span class="detail-value">{}/100</span>
                    </div>
                </div>
            </div>
            "#,
                health_class,
                &stats.node.node_id[..12],
                if stats.node.is_anchor != 0 { "⚓ Anchor" } else { "🔗 P2P" },
                stats.node.address,
                stats.node.port,
                stats.repo_count,
                stats.storage_usage_percent,
                stats.node.reputation_score,
            )
        }).collect::<Vec<_>>().join("\n");
        
        let content = format!(
            r#"
        <h1>🖥️ Enhanced Node Management</h1>
        
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Total Nodes</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Healthy Nodes</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Storage</div>
                <div class="stat-value">{} GB</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Used Storage</div>
                <div class="stat-value">{} GB</div>
            </div>
        </div>
        
        <div class="admin-nodes-grid">
            {}
        </div>
        
        <style>
            .admin-nodes-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
                gap: 2rem;
                margin-top: 2rem;
            }}
            
            .node-healthy {{
                border-color: var(--success-color);
            }}
            
            .node-warning {{
                border-color: var(--warning-color);
            }}
        </style>
        "#,
            total_nodes,
            healthy_nodes,
            total_storage / (1024 * 1024 * 1024),
            used_storage / (1024 * 1024 * 1024),
            nodes_html
        );
        
        render_page("Enhanced Node Management", &content)
    }
}

pub mod node_details {
    use super::*;
    use crate::models::NodeInfo;
    
    pub fn render(node: &Node, repo_count: i64, hosted_repos: &[NodeInfo]) -> String {
        let repos_html = if hosted_repos.is_empty() {
            "<p class='empty-state'>No repositories hosted</p>".to_string()
        } else {
            hosted_repos.iter().map(|repo| {
                format!("<div class='repo-item'><code>{}</code></div>", &repo.node_id[..12])
            }).collect::<Vec<_>>().join("\n")
        };
        
        let content = format!(
            r#"
        <h1>Node Details</h1>
        
        <div class="node-info">
            <div class="info-row">
                <span>Node ID:</span>
                <code>{}</code>
            </div>
            <div class="info-row">
                <span>Address:</span>
                <span>{}:{}</span>
            </div>
            <div class="info-row">
                <span>Reputation:</span>
                <span>{}/100</span>
            </div>
            <div class="info-row">
                <span>Hosted Repositories:</span>
                <span>{}</span>
            </div>
        </div>
        
        <h2>Hosted Repositories</h2>
        {}
        "#,
            node.node_id,
            node.address,
            node.port,
            node.reputation_score,
            repo_count,
            repos_html
        );
        
        render_page("Node Details", &content)
    }
}
