// src/templates/pinned.rs
use super::render_page;
use crate::models::Repository;

pub fn render(repos: &[Repository]) -> String {
    let repos_html = if repos.is_empty() {
        r#"<div class="empty-state">
            <p>You haven't pinned any repositories yet.</p>
            <p>Pin repositories to keep quick access to your favorites.</p>
            <a href="/explore" class="btn btn-primary">Explore Repositories</a>
        </div>"#.to_string()
    } else {
        format!(
            r#"<div class="repos-grid">{}</div>"#,
            repos.iter()
                .map(render_pinned_card)
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    
    let content = format!(
        r#"
    <div class="page-header">
        <h1> Pinned Repositories</h1>
        <p class="subtitle">Quick access to your pinned repositories</p>
    </div>
    
    <div class="pinned-stats">
        <div class="stat-badge">
            <span class="stat-number">{}</span>
            <span class="stat-label">Pinned Repos</span>
        </div>
        <div class="action-links">
            <a href="/dashboard" class="btn btn-secondary">Back to Dashboard</a>
            <a href="/starred" class="btn btn-secondary">View Starred</a>
        </div>
    </div>
    
    {}
    
    <style>
        .page-header {{
            text-align: center;
            margin-bottom: 3rem;
        }}
        
        .page-header h1 {{
            font-size: 3rem;
            color: var(--primary-color);
            text-shadow: 0 0 30px var(--text-glow);
            margin-bottom: 1rem;
        }}
        
        .pinned-stats {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: var(--bg-glass);
            padding: 2rem;
            border-radius: var(--border-radius);
            border: 2px solid var(--border-color);
            margin-bottom: 3rem;
            flex-wrap: wrap;
            gap: 1.5rem;
        }}
        
        .stat-badge {{
            display: flex;
            align-items: center;
            gap: 1rem;
        }}
        
        .stat-number {{
            font-size: 3rem;
            font-weight: 800;
            color: var(--primary-color);
            text-shadow: 0 0 20px var(--primary-glow);
        }}
        
        .stat-label {{
            font-size: 1.1rem;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        .action-links {{
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
        }}
        
        @media (max-width: 768px) {{
            .page-header h1 {{
                font-size: 2rem;
            }}
            
            .pinned-stats {{
                flex-direction: column;
                align-items: flex-start;
            }}
            
            .action-links {{
                width: 100%;
                flex-direction: column;
            }}
            
            .action-links a {{
                width: 100%;
            }}
        }}
    </style>
    "#,
        repos.len(),
        repos_html
    );
    
    render_page("Pinned Repositories", &content)
}

fn render_pinned_card(repo: &Repository) -> String {
    let short_hash = &repo.repo_hash[..16.min(repo.repo_hash.len())];
    let description = repo.description.as_deref().unwrap_or("No description provided");
    let size_kb = repo.size / 1024;
    let visibility = if repo.is_private != 0 { " Private" } else { " Public" };
    
    format!(
        r#"
    <div class="repo-card pinned-card">
        <div class="pin-indicator"></div>
        <h3 class="repo-name">
            <a href="/r/{}">{}</a>
        </h3>
        <p class="repo-description">{}</p>
        <div class="repo-meta">
            <span class="meta-item visibility-badge">{}</span>
            <span class="meta-item">
                <span class="meta-label">Hash:</span>
                <code class="repo-hash">{}</code>
            </span>
            <span class="meta-item">
                <span class="meta-label">Size:</span>
                {} KB
            </span>
            <span class="meta-item">
                <span class="meta-label">Updated:</span>
                {}
            </span>
        </div>
        <div class="card-actions">
            <a href="/r/{}/files" class="btn btn-secondary">Browse Files</a>
            <a href="/r/{}/clone" class="btn btn-secondary">Clone</a>
            <form method="POST" action="/repos/action" style="display:inline;">
                <input type="hidden" name="repo_hash" value="{}">
                <input type="hidden" name="action" value="unpin">
                <button type="submit" class="btn btn-outline">Unpin</button>
            </form>
        </div>
    </div>
    
    <style>
        .pinned-card {{
            position: relative;
            overflow: visible;
        }}
        
        .pin-indicator {{
            position: absolute;
            top: -10px;
            right: -10px;
            font-size: 2rem;
            filter: drop-shadow(0 0 10px var(--primary-glow));
            animation: pin-pulse 2s ease-in-out infinite;
            z-index: 10;
        }}
        
        @keyframes pin-pulse {{
            0%, 100% {{ 
                transform: scale(1) rotate(0deg);
                filter: drop-shadow(0 0 10px var(--primary-glow));
            }}
            50% {{ 
                transform: scale(1.1) rotate(-15deg);
                filter: drop-shadow(0 0 20px var(--primary-glow));
            }}
        }}
        
        .visibility-badge {{
            background: linear-gradient(135deg, 
                rgba(0, 255, 136, 0.2) 0%, 
                rgba(0, 170, 102, 0.2) 100%);
            padding: 0.4rem 0.8rem;
            border-radius: 15px;
            font-weight: 700;
            font-size: 0.85rem;
        }}
        
        .card-actions {{
            display: flex;
            gap: 0.75rem;
            margin-top: 1.5rem;
            flex-wrap: wrap;
        }}
        
        .card-actions form {{
            flex: 1;
            min-width: 120px;
        }}
        
        .card-actions button {{
            width: 100%;
        }}
        
        .btn-outline {{
            background: transparent;
            color: var(--text-secondary);
            border: 2px solid var(--border-color);
        }}
        
        .btn-outline:hover {{
            background: rgba(255, 51, 102, 0.1);
            color: var(--danger-color);
            border-color: var(--danger-color);
        }}
        
        @media (max-width: 768px) {{
            .card-actions {{
                flex-direction: column;
            }}
            
            .card-actions a,
            .card-actions form {{
                width: 100%;
            }}
        }}
    </style>
    "#,
        repo.repo_hash,
        repo.name,
        description,
        visibility,
        short_hash,
        size_kb,
        &repo.last_updated[..10],
        repo.repo_hash,
        repo.repo_hash,
        repo.repo_hash
    )
}
