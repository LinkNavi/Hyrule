// src/templates/dashboard.rs
use super::render_page_with_user;
use crate::models::Repository;

pub fn render(repos: &[Repository]) -> String {
    render_with_user(repos, None)
}

pub fn render_with_user(repos: &[Repository], username: Option<&str>) -> String {
    let repos_html = if repos.is_empty() {
        r#"<div class="empty-state">
            <p>You haven't created any repositories yet.</p>
            <a href="/docs" class="btn btn-primary">Learn How to Get Started</a>
        </div>"#.to_string()
    } else {
        repos.iter().map(render_repo_row).collect::<Vec<_>>().join("\n")
    };
    
    let content = format!(
        r#"
    <h1>📊 Dashboard</h1>
    
    <div class="dashboard-header">
        <div class="action-buttons">
            <a href="/repos/new" class="btn btn-primary">Create Repository</a>
            <a href="/explore" class="btn btn-secondary">Explore</a>
        </div>
    </div>
    
    <div class="section">
        <h2>Your Repositories</h2>
        <div class="repos-table">
            {}
        </div>
    </div>
    
    <div class="section">
        <h2>Storage Usage</h2>
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Storage Used</div>
                <div class="stat-value">0 MB / 1 GB</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Repositories</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Bandwidth This Month</div>
                <div class="stat-value">0 GB</div>
            </div>
        </div>
    </div>
    "#,
        repos_html,
        repos.len()
    );
    
    render_page_with_user("Dashboard", &content, username)
}

fn render_repo_row(repo: &Repository) -> String {
    let visibility = if repo.is_private == 1 { "🔒 Private" } else { "🌐 Public" };
    let short_hash = &repo.repo_hash[..12.min(repo.repo_hash.len())];
    
    format!(
        r#"
    <div class="repo-row">
        <div class="repo-row-main">
            <h3><a href="/r/{}">{}</a></h3>
            <p>{}</p>
        </div>
        <div class="repo-row-meta">
            <span class="visibility">{}</span>
            <span class="hash"><code>{}</code></span>
            <span class="size">{} KB</span>
            <span class="updated">Updated {}</span>
        </div>
    </div>
    "#,
        repo.repo_hash,
        repo.name,
        repo.description.as_deref().unwrap_or("No description"),
        visibility,
        short_hash,
        repo.size / 1024,
        &repo.last_updated[..10]
    )
}
