// src/templates/admin.rs
use super::render_page;
use crate::models::{Node, Repository};

pub fn render_dashboard(
    total_repos: i64,
    total_nodes: i64,
    unhealthy_repos: i64,
    total_storage: i64,
    bandwidth_24h: i64,
) -> String {
    let content = format!(
        r#"
    <h1> Admin Dashboard</h1>
    
    <div class="admin-stats">
        <div class="admin-stat-card">
            <div class="admin-stat-icon"></div>
            <div class="admin-stat-content">
                <div class="admin-stat-value">{}</div>
                <div class="admin-stat-label">Total Repositories</div>
            </div>
        </div>
        
        <div class="admin-stat-card">
            <div class="admin-stat-icon"></div>
            <div class="admin-stat-content">
                <div class="admin-stat-value">{}</div>
                <div class="admin-stat-label">Active Nodes</div>
            </div>
        </div>
        
        <div class="admin-stat-card warning">
            <div class="admin-stat-icon"></div>
            <div class="admin-stat-content">
                <div class="admin-stat-value">{}</div>
                <div class="admin-stat-label">Unhealthy Repos</div>
            </div>
        </div>
        
        <div class="admin-stat-card">
            <div class="admin-stat-icon"></div>
            <div class="admin-stat-content">
                <div class="admin-stat-value">{} GB</div>
                <div class="admin-stat-label">Total Storage</div>
            </div>
        </div>
        
        <div class="admin-stat-card">
            <div class="admin-stat-icon"></div>
            <div class="admin-stat-content">
                <div class="admin-stat-value">{} GB</div>
                <div class="admin-stat-label">Bandwidth (24h)</div>
            </div>
        </div>
    </div>
    
    <div class="admin-actions">
        <h2>Quick Actions</h2>
        <div class="action-buttons">
            <a href="/admin/nodes" class="btn btn-primary">Manage Nodes</a>
            <a href="/admin/repos" class="btn btn-secondary">View All Repos</a>
            <a href="/admin/users" class="btn btn-secondary">Manage Users</a>
            <button class="btn btn-warning" onclick="triggerHealthCheck()">Run Health Check</button>
        </div>
    </div>
    
    <div class="admin-section">
        <h2>Recent Activity</h2>
        <p class="empty-state">Activity log coming soon...</p>
    </div>
    
    <style>
        .admin-stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 2rem;
            margin: 2rem 0;
        }}
        
        .admin-stat-card {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 2rem;
            display: flex;
            gap: 1.5rem;
            align-items: center;
            transition: all 0.3s ease;
        }}
        
        .admin-stat-card:hover {{
            transform: translateY(-5px);
            box-shadow: var(--shadow-md);
        }}
        
        .admin-stat-card.warning {{
            border-color: var(--warning-color);
        }}
        
        .admin-stat-icon {{
            font-size: 3rem;
            filter: drop-shadow(0 0 10px var(--primary-glow));
        }}
        
        .admin-stat-value {{
            font-size: 2.5rem;
            font-weight: 800;
            color: var(--primary-color);
            text-shadow: 0 0 20px var(--text-glow);
        }}
        
        .admin-stat-label {{
            color: var(--text-secondary);
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        .admin-actions {{
            background: var(--bg-glass);
            padding: 2rem;
            border-radius: var(--border-radius);
            margin: 3rem 0;
        }}
        
        .admin-section {{
            background: var(--bg-glass);
            padding: 2rem;
            border-radius: var(--border-radius);
            margin: 2rem 0;
        }}
    </style>
    
    <script>
        async function triggerHealthCheck() {{
            const btn = event.target;
            btn.disabled = true;
            btn.textContent = 'Running...';
            
            try {{
                const response = await fetch('/api/admin/health-check', {{
                    method: 'POST',
                    headers: {{
                        'Authorization': 'Bearer ' + localStorage.getItem('token')
                    }}
                }});
                
                if (response.ok) {{
                    alert('Health check completed successfully!');
                }} else {{
                    alert('Health check failed');
                }}
            }} catch (e) {{
                alert('Error: ' + e.message);
            }} finally {{
                btn.disabled = false;
                btn.textContent = 'Run Health Check';
            }}
        }}
    </script>
    "#,
        total_repos,
        total_nodes,
        unhealthy_repos,
        total_storage / (1024 * 1024 * 1024),
        bandwidth_24h / (1024 * 1024 * 1024),
    );
    
    render_page("Admin Dashboard", &content)
}

pub fn render_nodes(nodes: &[(Node, i64)]) -> String {
    let nodes_html = nodes.iter().map(|(node, repo_count)| {
        let status = if node.is_anchor != 0 { " Anchor" } else { " P2P" };
        let health = if node.reputation_score >= 80 { "Healthy" } else { "Degraded" };
        
        format!(
            r#"
        <div class="admin-node-card">
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
                    <span class="detail-value">{} / {} GB</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Reputation:</span>
                    <span class="detail-value">{}/100</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Last Seen:</span>
                    <span class="detail-value">{}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Health:</span>
                    <span class="detail-value">{}</span>
                </div>
            </div>
            <div class="node-actions">
                <button class="btn btn-secondary" onclick="pingNode('{}')">Ping</button>
                <button class="btn btn-warning">Ban</button>
            </div>
        </div>
        "#,
            &node.node_id[..12],
            status,
            node.address,
            node.port,
            repo_count,
            node.storage_used / (1024 * 1024 * 1024),
            node.storage_capacity / (1024 * 1024 * 1024),
            node.reputation_score,
            &node.last_seen[..19],
            health,
            node.node_id,
        )
    }).collect::<Vec<_>>().join("\n");
    
    let content = format!(
        r#"
    <h1> Node Management</h1>
    
    <div class="admin-toolbar">
        <input type="text" id="node-search" placeholder="Search nodes..." class="search-input">
        <button class="btn btn-primary">Refresh</button>
    </div>
    
    <div class="admin-nodes-grid">
        {}
    </div>
    
    <style>
        .admin-toolbar {{
            display: flex;
            gap: 1rem;
            margin: 2rem 0;
        }}
        
        .search-input {{
            flex: 1;
            padding: 1rem 1.5rem;
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: 14px;
            color: var(--text-color);
            font-size: 1rem;
        }}
        
        .search-input:focus {{
            outline: none;
            border-color: var(--primary-color);
            box-shadow: 0 0 20px rgba(0, 255, 136, 0.3);
        }}
        
        .admin-nodes-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
            gap: 2rem;
        }}
        
        .admin-node-card {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 2rem;
            transition: all 0.3s ease;
        }}
        
        .admin-node-card:hover {{
            transform: translateY(-5px);
            box-shadow: var(--shadow-md);
            border-color: var(--border-glow);
        }}
        
        .node-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border-color);
        }}
        
        .node-id-badge {{
            font-weight: 700;
            color: var(--primary-color);
        }}
        
        .node-status {{
            padding: 0.5rem 1rem;
            background: rgba(0, 255, 136, 0.2);
            border-radius: 20px;
            font-size: 0.9rem;
            font-weight: 600;
        }}
        
        .node-details {{
            margin-bottom: 1.5rem;
        }}
        
        .detail-row {{
            display: flex;
            justify-content: space-between;
            padding: 0.75rem 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }}
        
        .detail-label {{
            color: var(--text-muted);
            font-weight: 600;
        }}
        
        .detail-value {{
            color: var(--text-color);
            font-family: 'Courier New', monospace;
        }}
        
        .node-actions {{
            display: flex;
            gap: 1rem;
        }}
    </style>
    
    <script>
        async function pingNode(nodeId) {{
            console.log('Pinging node:', nodeId);
            // Implement ping functionality
        }}
    </script>
    "#,
        nodes_html
    );
    
    render_page("Node Management", &content)
}

pub fn render_repos(repos: &[Repository]) -> String {
    let repos_html = repos.iter().map(|repo| {
        let visibility = if repo.is_private != 0 { " Private" } else { " Public" };
        
        format!(
            r#"
        <div class="admin-repo-row">
            <div class="repo-info">
                <div class="repo-name"><a href="/r/{}">{}</a></div>
                <div class="repo-meta">
                    <span>{}</span>
                    <span>Size: {} KB</span>
                    <span>Owner ID: {}</span>
                    <span>Updated: {}</span>
                </div>
            </div>
            <div class="repo-admin-actions">
                <button class="btn btn-secondary" onclick="viewRepoDetails('{}')">Details</button>
                <button class="btn btn-primary" onclick="triggerReplication('{}')">Replicate</button>
                <button class="btn btn-danger">Delete</button>
            </div>
        </div>
        "#,
            repo.repo_hash,
            repo.name,
            visibility,
            repo.size / 1024,
            repo.owner_id,
            &repo.last_updated[..10],
            repo.repo_hash,
            repo.repo_hash,
        )
    }).collect::<Vec<_>>().join("\n");
    
    let content = format!(
        r#"
    <h1> Repository Management</h1>
    
    <div class="admin-toolbar">
        <input type="text" id="repo-search" placeholder="Search repositories..." class="search-input">
        <select class="filter-select">
            <option value="all">All Repos</option>
            <option value="unhealthy">Unhealthy</option>
            <option value="private">Private</option>
            <option value="public">Public</option>
        </select>
        <button class="btn btn-primary">Refresh</button>
    </div>
    
    <div class="admin-repos-list">
        {}
    </div>
    
    <style>
        .filter-select {{
            padding: 1rem 1.5rem;
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: 14px;
            color: var(--text-color);
            font-size: 1rem;
            cursor: pointer;
        }}
        
        .admin-repos-list {{
            display: flex;
            flex-direction: column;
            gap: 1rem;
        }}
        
        .admin-repo-row {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            transition: all 0.3s ease;
        }}
        
        .admin-repo-row:hover {{
            transform: translateX(8px);
            box-shadow: var(--shadow-sm);
            border-color: var(--border-glow);
        }}
        
        .repo-info {{
            flex: 1;
        }}
        
        .repo-name {{
            font-size: 1.3rem;
            margin-bottom: 0.5rem;
        }}
        
        .repo-name a {{
            color: var(--primary-color);
            text-decoration: none;
            font-weight: 700;
        }}
        
        .repo-meta {{
            display: flex;
            gap: 1.5rem;
            color: var(--text-muted);
            font-size: 0.9rem;
        }}
        
        .repo-admin-actions {{
            display: flex;
            gap: 1rem;
        }}
        
        .btn-danger {{
            background: rgba(255, 51, 102, 0.2);
            color: var(--danger-color);
            border: 2px solid var(--danger-color);
        }}
        
        .btn-danger:hover {{
            background: rgba(255, 51, 102, 0.3);
            box-shadow: 0 0 20px rgba(255, 51, 102, 0.4);
        }}
    </style>
    
    <script>
        async function viewRepoDetails(repoHash) {{
            window.location.href = `/r/${{repoHash}}`;
        }}
        
        async function triggerReplication(repoHash) {{
            if (!confirm('Trigger replication for this repository?')) return;
            
            try {{
                const response = await fetch(`/api/repos/${{repoHash}}/replicate`, {{
                    method: 'POST',
                    headers: {{
                        'Authorization': 'Bearer ' + localStorage.getItem('token')
                    }}
                }});
                
                const data = await response.json();
                alert(data.message);
            }} catch (e) {{
                alert('Error: ' + e.message);
            }}
        }}
    </script>
    "#,
        repos_html
    );
    
    render_page("Repository Management", &content)
}
