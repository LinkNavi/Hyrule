// Hyrule/src/templates/commits.rs
use super::render_page;
use crate::models::Repository;
use crate::handlers::repo_browser::CommitInfo;

pub fn render(repo: &Repository, branch: &str, commits: &[CommitInfo]) -> String {
    let commits_html = if commits.is_empty() {
        "<p class='empty-state'>No commits yet</p>".to_string()
    } else {
        commits.iter().map(|commit| {
            let short_hash = &commit.hash[..8];
            let date = format_timestamp(commit.timestamp);
            
            format!(
                r#"<div class="commit-item">
                    <div class="commit-header">
                        <a href="/r/{}/commit/{}" class="commit-hash">{}</a>
                        <span class="commit-date">{}</span>
                    </div>
                    <div class="commit-message">{}</div>
                    <div class="commit-author">By {} &lt;{}&gt;</div>
                </div>"#,
                repo.repo_hash, commit.hash, short_hash,
                date,
                html_escape(&commit.message),
                html_escape(&commit.author),
                html_escape(&commit.email)
            )
        }).collect::<Vec<_>>().join("\n")
    };
    
    let content = format!(
        r#"
    <div class="breadcrumb">
        <a href="/r/{}">← Back to Repository</a>
    </div>
    
    <div class="repo-header">
        <h1>{}</h1>
        <p class="repo-description">Commit History</p>
    </div>
    
    <div class="repo-nav">
        <a href="/r/{}/files?branch={}" class="nav-tab">Files</a>
        <a href="/r/{}/commits?branch={}" class="nav-tab active">Commits</a>
        <a href="/r/{}/branches" class="nav-tab">Branches</a>
        <a href="/r/{}/clone" class="nav-tab">Clone</a>
    </div>
    
    <div class="branch-info">
        <strong>Branch:</strong> {} | <strong>Commits:</strong> {}
    </div>
    
    <div class="commits-list">
        {}
    </div>
    
    <style>
        .branch-info {{
            background: var(--bg-glass);
            padding: 1rem 2rem;
            border-radius: var(--border-radius);
            margin: 2rem 0;
            color: var(--text-secondary);
        }}
        
        .commits-list {{
            display: flex;
            flex-direction: column;
            gap: 1rem;
        }}
        
        .commit-item {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 1.5rem 2rem;
            transition: all 0.3s ease;
        }}
        
        .commit-item:hover {{
            border-color: var(--border-glow);
            transform: translateY(-2px);
            box-shadow: var(--shadow-sm);
        }}
        
        .commit-header {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.75rem;
        }}
        
        .commit-hash {{
            font-family: 'Courier New', monospace;
            color: var(--primary-color);
            text-decoration: none;
            font-weight: 700;
            padding: 0.3rem 0.7rem;
            background: rgba(0, 255, 136, 0.1);
            border-radius: 8px;
        }}
        
        .commit-hash:hover {{
            background: rgba(0, 255, 136, 0.2);
        }}
        
        .commit-date {{
            color: var(--text-muted);
            font-size: 0.9rem;
        }}
        
        .commit-message {{
            font-size: 1.1rem;
            color: var(--text-color);
            margin-bottom: 0.75rem;
            font-weight: 600;
        }}
        
        .commit-author {{
            color: var(--text-secondary);
            font-size: 0.9rem;
        }}
    </style>
    "#,
        repo.repo_hash,
        repo.name,
        repo.repo_hash, branch,
        repo.repo_hash, branch,
        repo.repo_hash,
        repo.repo_hash,
        branch,
        commits.len(),
        commits_html
    );
    
    render_page(&format!("Commits - {}", repo.name), &content)
}

fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Utc, TimeZone};
    let dt: DateTime<Utc> = Utc.timestamp_opt(ts, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Hyrule/src/templates/commit_view.rs
pub mod commit_view {
    use super::{render_page, html_escape};
    use crate::models::Repository;
    
    pub fn render(repo: &Repository, commit_hash: &str, info: &str, diff: &str) -> String {
        let short_hash = &commit_hash[..8];
        
        let diff_html = if diff.is_empty() {
            "<p class='empty-state'>No changes in this commit</p>".to_string()
        } else {
            let lines: Vec<&str> = diff.lines().collect();
            let formatted_lines = lines.iter().map(|line| {
                let (class, symbol) = if line.starts_with('+') && !line.starts_with("+++") {
                    ("diff-add", "+")
                } else if line.starts_with('-') && !line.starts_with("---") {
                    ("diff-remove", "-")
                } else if line.starts_with("@@") {
                    ("diff-info", "@")
                } else {
                    ("diff-context", " ")
                };
                
                format!(
                    r#"<tr class="{}">
                        <td class="diff-marker">{}</td>
                        <td class="diff-line"><code>{}</code></td>
                    </tr>"#,
                    class, symbol, html_escape(line)
                )
            }).collect::<Vec<_>>().join("\n");
            
            format!(r#"<table class="diff-table"><tbody>{}</tbody></table>"#, formatted_lines)
        };
        
        let content = format!(
            r#"
        <div class="breadcrumb">
            <a href="/r/{}/commits">← Back to Commits</a>
        </div>
        
        <div class="commit-detail">
            <div class="commit-header-box">
                <h1>Commit <code class="commit-hash-display">{}</code></h1>
                <p class="commit-full-hash">{}</p>
            </div>
            
            <div class="commit-info-box">
                <pre class="commit-message">{}</pre>
            </div>
            
            <div class="diff-viewer">
                <h2>Changes</h2>
                <div class="diff-content">
                    {}
                </div>
            </div>
        </div>
        
        <style>
            .commit-detail {{
                max-width: 1400px;
                margin: 0 auto;
            }}
            
            .commit-header-box {{
                background: var(--bg-glass);
                padding: 2rem;
                border-radius: var(--border-radius);
                border: 2px solid var(--border-color);
                margin-bottom: 2rem;
            }}
            
            .commit-hash-display {{
                color: var(--primary-color);
                font-size: 1.5rem;
            }}
            
            .commit-full-hash {{
                font-family: 'Courier New', monospace;
                color: var(--text-muted);
                font-size: 0.9rem;
                margin-top: 0.5rem;
            }}
            
            .commit-info-box {{
                background: var(--bg-glass);
                padding: 2rem;
                border-radius: var(--border-radius);
                border: 2px solid var(--border-color);
                margin-bottom: 2rem;
            }}
            
            .commit-message {{
                color: var(--text-color);
                font-size: 1.1rem;
                line-height: 1.6;
                white-space: pre-wrap;
                margin: 0;
            }}
            
            .diff-viewer {{
                background: var(--bg-glass);
                border: 2px solid var(--border-color);
                border-radius: var(--border-radius);
                overflow: hidden;
            }}
            
            .diff-viewer h2 {{
                background: rgba(0, 255, 136, 0.05);
                padding: 1rem 2rem;
                margin: 0;
                border-bottom: 1px solid var(--border-color);
            }}
            
            .diff-content {{
                overflow-x: auto;
            }}
            
            .diff-table {{
                width: 100%;
                border-collapse: collapse;
                font-family: 'Courier New', monospace;
                font-size: 0.9rem;
            }}
            
            .diff-marker {{
                width: 30px;
                padding: 0.3rem 0.5rem;
                text-align: center;
                background: rgba(0, 0, 0, 0.3);
                border-right: 1px solid var(--border-color);
                user-select: none;
            }}
            
            .diff-line {{
                padding: 0.3rem 1rem;
                white-space: pre;
            }}
            
            .diff-line code {{
                background: none;
                border: none;
                padding: 0;
                color: inherit;
            }}
            
            .diff-add {{
                background: rgba(0, 255, 0, 0.1);
            }}
            
            .diff-add .diff-marker {{
                background: rgba(0, 255, 0, 0.2);
                color: #0f0;
            }}
            
            .diff-remove {{
                background: rgba(255, 0, 0, 0.1);
            }}
            
            .diff-remove .diff-marker {{
                background: rgba(255, 0, 0, 0.2);
                color: #f00;
            }}
            
            .diff-info {{
                background: rgba(0, 136, 255, 0.1);
                color: var(--text-muted);
            }}
            
            .diff-context {{
                background: transparent;
            }}
        </style>
        "#,
            repo.repo_hash,
            short_hash,
            commit_hash,
            html_escape(info),
            diff_html
        );
        
        render_page(&format!("Commit {} - {}", short_hash, repo.name), &content)
    }
}

// Hyrule/src/templates/branches.rs
pub mod branches {
    use super::render_page;
    use crate::models::Repository;
    
    pub fn render(repo: &Repository, branches: &[String]) -> String {
        let branches_html = if branches.is_empty() {
            "<p class='empty-state'>No branches</p>".to_string()
        } else {
            branches.iter().map(|branch| {
                let clean_branch = branch.trim();
                let is_current = clean_branch.starts_with('*');
                let branch_name = clean_branch.trim_start_matches("* ").trim();
                
                format!(
                    r#"<div class="branch-item {}">
                        <div class="branch-name">
                            <span class="branch-icon"></span>
                            <strong>{}</strong>
                            {}
                        </div>
                        <div class="branch-actions">
                            <a href="/r/{}/files?branch={}" class="btn btn-secondary">View Files</a>
                            <a href="/r/{}/commits?branch={}" class="btn btn-secondary">View Commits</a>
                        </div>
                    </div>"#,
                    if is_current { "branch-current" } else { "" },
                    branch_name,
                    if is_current { "<span class='current-badge'>Current</span>" } else { "" },
                    repo.repo_hash, branch_name,
                    repo.repo_hash, branch_name
                )
            }).collect::<Vec<_>>().join("\n")
        };
        
        let content = format!(
            r#"
        <div class="breadcrumb">
            <a href="/r/{}">← Back to Repository</a>
        </div>
        
        <div class="repo-header">
            <h1>{}</h1>
            <p class="repo-description">Branches</p>
        </div>
        
        <div class="repo-nav">
            <a href="/r/{}/files" class="nav-tab">Files</a>
            <a href="/r/{}/commits" class="nav-tab">Commits</a>
            <a href="/r/{}/branches" class="nav-tab active">Branches</a>
            <a href="/r/{}/clone" class="nav-tab">Clone</a>
        </div>
        
        <div class="branches-list">
            {}
        </div>
        
        <style>
            .branches-list {{
                display: flex;
                flex-direction: column;
                gap: 1rem;
                margin-top: 2rem;
            }}
            
            .branch-item {{
                background: var(--bg-glass);
                border: 2px solid var(--border-color);
                border-radius: var(--border-radius);
                padding: 1.5rem 2rem;
                display: flex;
                justify-content: space-between;
                align-items: center;
                transition: all 0.3s ease;
            }}
            
            .branch-item:hover {{
                border-color: var(--border-glow);
                transform: translateX(8px);
            }}
            
            .branch-current {{
                border-color: var(--primary-color);
                background: rgba(0, 255, 136, 0.05);
            }}
            
            .branch-name {{
                display: flex;
                align-items: center;
                gap: 1rem;
                font-size: 1.2rem;
            }}
            
            .branch-icon {{
                font-size: 1.5rem;
            }}
            
            .current-badge {{
                background: var(--primary-color);
                color: #000;
                padding: 0.3rem 0.8rem;
                border-radius: 15px;
                font-size: 0.8rem;
                font-weight: 700;
                margin-left: 1rem;
            }}
            
            .branch-actions {{
                display: flex;
                gap: 1rem;
            }}
        </style>
        "#,
            repo.repo_hash,
            repo.name,
            repo.repo_hash,
            repo.repo_hash,
            repo.repo_hash,
            repo.repo_hash,
            branches_html
        );
        
        render_page(&format!("Branches - {}", repo.name), &content)
    }
}

pub use commit_view::*;
pub use branches::*;
