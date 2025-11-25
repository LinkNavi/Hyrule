
// src/templates/search.rs
use super::render_page;
use crate::models::Repository;

pub fn render(results: &[Repository], search_term: &str) -> String {
    let results_html = if results.is_empty() {
        format!(
            r#"<div class="empty-state">
                <p>No repositories found for "{}"</p>
                <a href="/explore" class="btn btn-secondary">Browse All Repositories</a>
            </div>"#,
            search_term
        )
    } else {
        format!(
            r#"<p class="search-results-count">Found {} repositories for "{}"</p>
            <div class="repos-grid">{}</div>"#,
            results.len(),
            search_term,
            results.iter()
                .map(|r| render_repo_card(r))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    
    let content = format!(
        r#"
    <h1>Search Repositories</h1>
    
    <form method="GET" action="/search" class="search-form">
        <input type="text" name="q" placeholder="Search repositories..." 
               value="{}" class="search-input" autofocus>
        <button type="submit" class="btn btn-primary">Search</button>
    </form>
    
    <div class="search-results">
        {}
    </div>
    
    <style>
        .search-form {{
            display: flex;
            gap: 1rem;
            margin: 2rem 0 3rem 0;
        }}
        
        .search-input {{
            flex: 1;
            padding: 1rem 1.5rem;
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: 14px;
            color: var(--text-color);
            font-size: 1.1rem;
        }}
        
        .search-input:focus {{
            outline: none;
            border-color: var(--primary-color);
            box-shadow: 0 0 20px rgba(0, 255, 136, 0.3);
        }}
        
        .search-results-count {{
            color: var(--text-secondary);
            margin-bottom: 2rem;
            font-size: 1.1rem;
        }}
    </style>
    "#,
        search_term,
        results_html
    );
    
    render_page("Search", &content)
}

pub fn render_repo_card(repo: &Repository) -> String {
    let short_hash = &repo.repo_hash[..16.min(repo.repo_hash.len())];
    let description = repo.description.as_deref().unwrap_or("No description provided");
    
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
        </div>
    </div>
    "#,
        repo.repo_hash,
        repo.name,
        description,
        short_hash,
        repo.size / 1024
    )
}
