// src/templates/dashboard_enhanced.rs
use super::render_page;
use crate::models::Repository;

/// Minimal HTML escape to avoid injecting user-controlled strings directly into templates.
fn escape_html(s: &str) -> String {
    s.chars().map(|c| {
        match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#x27;".to_string(),
            '/' => "&#x2F;".to_string(),
            other => other.to_string(),
        }
    }).collect()
}

/// Public API used by handlers: accepts an optional username.
pub fn render_with_user(
    repos: &[Repository],
    username: Option<&str>,
    pinned_count: usize,
    starred_count: usize,
) -> String {
    // sign-in indicator (either show username or sign-in/sign-up buttons)
    let sign_in_html = match username {
        Some(u) => {
            let esc = escape_html(u);
            format!(
                r#"<div class="signin-indicator">Signed in as <a href="/profile">{}</a></div>"#,
                esc
            )
        }
        None => {
            r#"<div class="signin-indicator">
                <a href="/signin" class="btn btn-secondary">Sign In</a>
                <a href="/signup" class="btn btn-primary">Sign Up</a>
              </div>"#
            .to_string()
        }
    };

    // repo html
    let repos_html = if repos.is_empty() {
        r#"<div class="empty-state">
            <p>You haven't created any repositories yet.</p>
            <a href="/repos/new" class="btn btn-primary">Create Your First Repository</a>
        </div>"#
        .to_string()
    } else {
        repos.iter().map(render_repo_row).collect::<Vec<_>>().join("\n")
    };

    // display name used in the welcome header (fallback to "Guest" for None)
    let display_name = username.map(|s| escape_html(s)).unwrap_or_else(|| "Guest".to_string());

    let content = format!(
        r#"
    <div class="dashboard-top">
        {sign_in}
        <h1>Welcome back, {display}!</h1>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-label">Your Repositories</div>
            <div class="stat-value">{repo_count}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Starred</div>
            <div class="stat-value">{starred}</div>
            <a href="/starred" class="btn btn-secondary">View Starred</a>
        </div>
        <div class="stat-card">
            <div class="stat-label">Pinned</div>
            <div class="stat-value">{pinned}</div>
            <a href="/pinned" class="btn btn-secondary">View Pinned</a>
        </div>
    </div>

    <div class="dashboard-header">
        <h2>Your Repositories</h2>
        <div class="action-buttons">
            <a href="/repos/new" class="btn btn-primary">Create Repository</a>
            <a href="/profile" class="btn btn-secondary">View Profile</a>
        </div>
    </div>

    <div class="repos-table">
        {repos_html}
    </div>
    "#,
        sign_in = sign_in_html,
        display = display_name,
        repo_count = repos.len(),
        starred = starred_count,
        pinned = pinned_count,
        repos_html = repos_html
    );

    render_page("Dashboard", &content)
}

/// Convenience wrapper if you want to call render with a guaranteed username string.
/// Not strictly necessary if you always call `render_with_user(..., Option<&str>, ...)`.
pub fn render(
    repos: &[Repository],
    username: &str,
    pinned_count: usize,
    starred_count: usize,
) -> String {
    render_with_user(repos, Some(username), pinned_count, starred_count)
}

fn render_repo_row(repo: &Repository) -> String {
    let visibility = if repo.is_private == 1 { "Private" } else { "Public" };
    let short_hash = &repo.repo_hash[..std::cmp::min(12, repo.repo_hash.len())];
    let desc = repo.description.as_deref().unwrap_or("No description");
    let desc_esc = escape_html(desc);
    let name_esc = escape_html(&repo.name);

    // last_updated is assumed to be an ISO-like string; take first 10 chars as date if available
    let updated = if repo.last_updated.len() >= 10 {
        &repo.last_updated[..10]
    } else {
        &repo.last_updated
    };

    format!(
        r#"
    <div class="repo-row">
        <div class="repo-row-main">
            <h3><a href="/r/{hash}">{name}</a></h3>
            <p>{desc}</p>
        </div>
        <div class="repo-row-meta">
            <span class="visibility">{vis}</span>
            <span class="hash"><code>{short}</code></span>
            <span class="size">{size} KB</span>
            <span class="updated">Updated {updated}</span>
        </div>
        <div class="repo-row-actions">
            <a href="/r/{hash}/files" class="btn btn-secondary">Files</a>
            <a href="/r/{hash}/clone" class="btn btn-secondary">Clone</a>
            <form method="POST" action="/repos/action" style="display:inline;">
                <input type="hidden" name="repo_hash" value="{hash}">
                <input type="hidden" name="action" value="delete">
                <button type="submit" class="btn btn-danger" 
                    onclick="return confirm('Are you sure you want to delete this repository?')">Delete</button>
            </form>
        </div>
    </div>
    "#,
        hash = repo.repo_hash,
        name = name_esc,
        desc = desc_esc,
        vis = visibility,
        short = short_hash,
        size = repo.size / 1024,
        updated = updated
    )
}
