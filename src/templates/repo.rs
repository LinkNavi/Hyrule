// src/templates/repo.rs
use super::render_page_with_user;
use crate::models::{Repository, NodeInfo};

pub fn render(repo: &Repository, replica_count: i64, nodes: &[NodeInfo]) -> String {
    render_with_user(repo, replica_count, nodes, None)
}

pub fn render_with_user(repo: &Repository, replica_count: i64, nodes: &[NodeInfo], username: Option<&str>) -> String {
    let health_status = if replica_count >= 5 {
        ("Excellent", "status-excellent")
    } else if replica_count >= 3 {
        ("Good", "status-good")
    } else {
        ("Needs Replication", "status-warning")
    };
    
    let nodes_html = if nodes.is_empty() {
        "<p class='empty-state'>No active nodes currently hosting this repository.</p>".to_string()
    } else {
        nodes.iter().map(render_node_info).collect::<Vec<_>>().join("\n")
    };
    
    let content = format!(
        r#"
    <div class="breadcrumb">
        <a href="/explore">← Back to Explore</a>
    </div>
    
    <div class="repo-header">
        <h1>{}</h1>
        <p class="repo-description">{}</p>
    </div>
    
    <div class="clone-box">
        <h3>Clone this repository:</h3>
        <div class="code-block">
            <pre><code>$ triforge clone hyrule://{}</code></pre>
        </div>
        <p class="hint">💡 Use <code>--anonymous</code> flag for Tor routing</p>
    </div>
    
    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-label">Repository Hash</div>
            <div class="stat-value"><code>{}</code></div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Size</div>
            <div class="stat-value">{} KB</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Replicas</div>
            <div class="stat-value">{}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Health</div>
            <div class="stat-value {}">
                <span class="status-indicator"></span>
                {}
            </div>
        </div>
    </div>
    
    <div class="section">
        <h2>Network Status</h2>
        <p>This repository is currently hosted on {} node(s) across the Hyrule network.</p>
        
        <div class="nodes-list">
            {}
        </div>
    </div>
    
    <div class="section">
        <h2>Files</h2>
        <p class="empty-state">File browser coming soon...</p>
    </div>
    
    <div class="section">
        <h2>Commits</h2>
        <p class="empty-state">Commit history coming soon...</p>
    </div>
    "#,
        repo.name,
        repo.description.as_deref().unwrap_or("No description"),
        repo.repo_hash,
        &repo.repo_hash[..16.min(repo.repo_hash.len())],
        repo.size / 1024,
        replica_count,
        health_status.1,
        health_status.0,
        replica_count,
        nodes_html
    );
    
    render_page_with_user(&repo.name, &content, username)
}

fn render_node_info(node: &NodeInfo) -> String {
    let node_type = if node.is_anchor { "⚓ Anchor Node" } else { "🔗 P2P Node" };
    let short_id = &node.node_id[..12.min(node.node_id.len())];
    
    format!(
        r#"
    <div class="node-item">
        <div class="node-icon">{}</div>
        <div class="node-details">
            <div class="node-id"><code>{}</code></div>
            <div class="node-address">{}:{}</div>
        </div>
        <div class="node-type">{}</div>
    </div>
    "#,
        if node.is_anchor { "⚓" } else { "🔗" },
        short_id,
        node.address,
        node.port,
        node_type
    )
}
