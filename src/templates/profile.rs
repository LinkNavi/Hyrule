
// src/templates/profile.rs
use super::render_page;
use crate::models::{User, Repository};

pub fn render(
    user: &User,
    repos: &[Repository],
    starred: &[Repository],
    pinned: &[Repository],
) -> String {
    let content = format!(
        r#"
    <h1>Profile: {}</h1>
    
    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-label">Repositories</div>
            <div class="stat-value">{}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Starred</div>
            <div class="stat-value">{}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Pinned</div>
            <div class="stat-value">{}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Storage Used</div>
            <div class="stat-value">{} MB / {} GB</div>
        </div>
    </div>
    
    <div class="section">
        <h2>Actions</h2>
        <div class="action-buttons">
            <a href="/repos/new" class="btn btn-primary">Create Repository</a>
            <a href="/dashboard" class="btn btn-secondary">Dashboard</a>
            <form method="POST" action="/logout" style="display:inline;">
                <button type="submit" class="btn btn-danger">Logout</button>
            </form>
        </div>
    </div>
    "#,
        user.username,
        repos.len(),
        starred.len(),
        pinned.len(),
        user.storage_used / (1024 * 1024),
        user.storage_quota / (1024 * 1024 * 1024)
    );
    
    render_page("Profile", &content)
}
