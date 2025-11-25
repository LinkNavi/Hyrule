// src/templates/starred.rs and pinned.rs
use super::render_page;
use crate::models::Repository;

pub fn render(repos: &[Repository]) -> String {
    let repos_html = if repos.is_empty() {
        r#"<div class="empty-state">
            <p>You haven't starred any repositories yet.</p>
            <a href="/explore" class="btn btn-primary">Explore Repositories</a>
        </div>"#.to_string()
    } else {
        repos.iter()
            .map(|r| super::search::render_repo_card(r))
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    let content = format!(
        r#"
    <h1> Starred Repositories</h1>
    <p class="subtitle">Repositories you've starred</p>
    
    <div class="repos-grid">
        {}
    </div>
    "#,
        repos_html
    );
    
    render_page("Starred Repositories", &content)
}
