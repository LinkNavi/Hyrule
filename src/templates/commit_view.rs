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
