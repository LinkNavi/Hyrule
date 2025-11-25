// src/templates/repo_enhanced.rs
use super::render_page;
use crate::models::{NodeInfo, Repository};

pub async fn render_with_readme(
    repo: &Repository,
    owner_username: &str,
    replica_count: i64,
    nodes: &[NodeInfo],
    tags: &[String],
    star_count: i64,
    is_owner: bool,
    is_starred: bool,
    is_pinned: bool,
    readme_html: Option<String>,
) -> String {
    let health_status = if replica_count >= 5 {
        ("Excellent", "status-excellent")
    } else if replica_count >= 3 {
        ("Good", "status-good")
    } else {
        ("Needs Replication", "status-warning")
    };

    let tags_html = if tags.is_empty() {
        String::new()
    } else {
        let tags_list = tags
            .iter()
            .map(|tag| {
                format!(
                    r#"<a href="/search?tag={}" class="tag-badge">{}</a>"#,
                    tag, tag
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!(r#"<div class="repo-tags">{}</div>"#, tags_list)
    };

    let action_buttons = if is_owner {
        format!(
            r#"
        <form method="POST" action="/repos/action" style="display:inline;">
            <input type="hidden" name="repo_hash" value="{}">
            <input type="hidden" name="action" value="delete">
            <button type="submit" class="btn btn-danger" 
                onclick="return confirm('Delete this repository permanently?')">Delete Repository</button>
        </form>
        "#,
            repo.repo_hash
        )
    } else {
        let star_action = if is_starred { "unstar" } else { "star" };
        let star_text = if is_starred { "⭐ Unstar" } else { "⭐ Star" };
        let pin_action = if is_pinned { "unpin" } else { "pin" };
        let pin_text = if is_pinned { "📌 Unpin" } else { "📌 Pin" };

        format!(
            r#"
        <form method="POST" action="/repos/action" style="display:inline;">
            <input type="hidden" name="repo_hash" value="{}">
            <input type="hidden" name="action" value="{}">
            <button type="submit" class="btn btn-secondary">{}</button>
        </form>
        <form method="POST" action="/repos/action" style="display:inline;">
            <input type="hidden" name="repo_hash" value="{}">
            <input type="hidden" name="action" value="{}">
            <button type="submit" class="btn btn-secondary">{}</button>
        </form>
        <button onclick="showForkDialog()" class="btn btn-primary">🍴 Fork</button>
        
        <div id="fork-dialog" style="display:none; position:fixed; top:50%; left:50%; transform:translate(-50%,-50%); background:var(--bg-glass); padding:3rem; border-radius:var(--border-radius); border:2px solid var(--border-color); z-index:9999; backdrop-filter:blur(20px);">
            <h2>Fork Repository</h2>
            <form method="POST" action="/repos/fork">
                <input type="hidden" name="repo_hash" value="{}">
                <div class="form-group">
                    <label>New Name (optional)</label>
                    <input type="text" name="new_name" placeholder="{}-fork">
                </div>
                <div class="form-group">
                    <label>Description (optional)</label>
                    <textarea name="description" rows="3"></textarea>
                </div>
                <div style="display:flex; gap:1rem; margin-top:2rem;">
                    <button type="submit" class="btn btn-primary">Create Fork</button>
                    <button type="button" onclick="document.getElementById('fork-dialog').style.display='none'" class="btn btn-secondary">Cancel</button>
                </div>
            </form>
        </div>
        
        <div id="fork-overlay" onclick="document.getElementById('fork-dialog').style.display='none'; this.style.display='none';" style="display:none; position:fixed; inset:0; background:rgba(0,0,0,0.8); z-index:9998;"></div>
        
        <noscript>
            <form method="POST" action="/repos/fork" style="display:inline; margin-left:1rem;">
                <input type="hidden" name="repo_hash" value="{}">
                <button type="submit" class="btn btn-primary">🍴 Fork (No-JS)</button>
            </form>
        </noscript>
        
        <script>
        function showForkDialog() {{
            document.getElementById('fork-dialog').style.display='block';
            document.getElementById('fork-overlay').style.display='block';
        }}
        </script>
        "#,
            repo.repo_hash,
            star_action,
            star_text,
            repo.repo_hash,
            pin_action,
            pin_text,
            repo.repo_hash,
            repo.name,
            repo.repo_hash
        )
    };
    let readme_section = if let Some(html) = readme_html {
        format!(
            r#"
    <div class="section readme-section">
        <h2>📄 README</h2>
        <div class="readme-content">
            {}
        </div>
    </div>
    
    <style>
        .readme-section {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 3rem;
            margin: 3rem 0;
        }}
        
        .readme-content {{
            color: var(--text-color);
            line-height: 1.8;
        }}
        
        .readme-content h1, .readme-content h2, .readme-content h3 {{
            color: var(--primary-color);
            margin-top: 2rem;
            margin-bottom: 1rem;
        }}
        
        .readme-content pre {{
            background: linear-gradient(135deg, #0a1510 0%, #0d1a14 100%);
            padding: 1.5rem;
            border-radius: 12px;
            overflow-x: auto;
            border: 1px solid var(--border-color);
        }}
        
        .readme-content code {{
            background: rgba(0, 255, 136, 0.1);
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-family: 'Courier New', monospace;
        }}
        
        .readme-content a {{
            color: var(--primary-color);
            text-decoration: none;
        }}
        
        .readme-content a:hover {{
            text-shadow: 0 0 10px var(--primary-glow);
        }}
    </style>
    "#,
            html
        )
    } else {
        String::new()
    };
    let content = format!(
        r#"
    <div class="breadcrumb">
        <a href="/explore">← Back to Explore</a>
    </div>
    
    <div class="repo-header">
        <h1>{}</h1>
        <p class="repo-description">{}</p>
        <p class="repo-owner">by <strong>{}</strong></p>
        {}
    </div>
    
    <div class="repo-nav">
        <a href="/r/{}/files" class="nav-tab">Files</a>
        <a href="/r/{}/commits" class="nav-tab">Commits</a>
        <a href="/r/{}/branches" class="nav-tab">Branches</a>
        <a href="/r/{}/clone" class="nav-tab">Clone</a>
    </div>
    
    <div class="repo-actions">
        {}
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
            <div class="stat-label">Stars</div>
            <div class="stat-value"> {}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Health</div>
            <div class="stat-value {}">
                <span class="status-indicator"></span>
                {}
            </div>
        </div>
    </div>
    
    {}
    
    <div class="section">
        <h2>Clone This Repository</h2>
        <a href="/r/{}/clone" class="btn btn-primary">View Clone Instructions</a>
    </div>
    "#,
        repo.name,
        repo.description.as_deref().unwrap_or("No description"),
        owner_username,
        tags_html,
        repo.repo_hash,
        repo.repo_hash,
        repo.repo_hash,
        repo.repo_hash,
        action_buttons,
        &repo.repo_hash[..16.min(repo.repo_hash.len())],
        repo.size / 1024,
        replica_count,
        star_count,
        health_status.1,
        health_status.0,
        readme_section, // Now properly used
        repo.repo_hash,
    );

    render_page(&repo.name, &content)
}
