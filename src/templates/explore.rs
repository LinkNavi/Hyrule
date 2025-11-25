// src/templates/explore.rs
use super::{render_page_with_user};
use crate::models::Repository;

pub fn render(repos: &[Repository]) -> String {
    render_with_user(repos, None)
}

pub fn render_with_user(repos: &[Repository], username: Option<&str>) -> String {
    let repos_html = if repos.is_empty() {
        "<p class='empty-state'>No repositories found. Be the first to publish!</p>".to_string()
    } else {
        repos.iter().map(render_repo_card).collect::<Vec<_>>().join("\n")
    };
    
    let content = format!(
        r#"
    <h1>Explore Repositories</h1>
    <p class="subtitle">Discover public repositories on the Hyrule network</p>
    
    <div class="explore-controls">
        <div class="search-box">
            <form method="GET" action="/search" style="display:flex;gap:1rem;width:100%;">
                <input type="text" name="q" placeholder="Search repositories..." class="search-input">
                <button type="submit" class="btn btn-primary">Search</button>
            </form>
        </div>
    </div>
    
    <div id="repos-container" class="repos-grid">
        {}
    </div>
    "#,
        repos_html
    );
    
    render_page_with_user("Explore", &content, username)
}

fn render_repo_card(repo: &Repository) -> String {
    let short_hash = &repo.repo_hash[..16.min(repo.repo_hash.len())];
    let description = repo.description.as_deref().unwrap_or("No description provided");
    let size_kb = repo.size / 1024;
    
    format!(
        r#"
    <div class="repo-card">
        <h3 class="repo-name">
            <a href="/r/{}">{}</a>
        </h3>
        <p class="repo-description">{}</p>
        <div class="repo-meta">
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
    </div>
    "#,
        repo.repo_hash,
        repo.name,
        description,
        short_hash,
        size_kb,
        &repo.last_updated[..10]
    )
}
