// Hyrule/src/templates/files.rs
use super::render_page;
use crate::models::Repository;
use crate::handlers::repo_browser::FileEntry;

pub fn render(repo: &Repository, branch: &str, current_path: &str, files: &[FileEntry]) -> String {
    let breadcrumb = render_breadcrumb(&repo.repo_hash, current_path, branch);
    
    let files_html = if files.is_empty() {
        "<p class='empty-state'>This directory is empty or the branch doesn't exist yet.</p>".to_string()
    } else {
        files.iter().map(|file| render_file_entry(repo, branch, file)).collect::<Vec<_>>().join("\n")
    };
    
    let content = format!(
        r#"
    {}
    
    <div class="repo-header">
        <h1>{}</h1>
        <p class="repo-description">Browse Files</p>
    </div>
    
    <div class="repo-nav">
        <a href="/r/{}/files?branch={}" class="nav-tab active">Files</a>
        <a href="/r/{}/commits?branch={}" class="nav-tab">Commits</a>
        <a href="/r/{}/branches" class="nav-tab">Branches</a>
        <a href="/r/{}/clone" class="nav-tab">Clone</a>
    </div>
    
    <div class="branch-selector">
        <strong>Branch:</strong> {} 
        <a href="/r/{}/branches" class="btn btn-secondary">Switch Branch</a>
    </div>
    
    <div class="file-browser">
        <div class="file-list">
            {}
        </div>
    </div>
    
    <style>
        .branch-selector {{
            background: var(--bg-glass);
            padding: 1rem 2rem;
            border-radius: var(--border-radius);
            margin: 2rem 0;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border: 2px solid var(--border-color);
        }}
        
        .file-browser {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            overflow: hidden;
            margin-top: 2rem;
        }}
        
        .file-list {{
            display: flex;
            flex-direction: column;
        }}
        
        .file-entry {{
            display: flex;
            align-items: center;
            padding: 1rem 2rem;
            border-bottom: 1px solid var(--border-color);
            transition: all 0.3s ease;
            text-decoration: none;
            color: var(--text-color);
        }}
        
        .file-entry:last-child {{
            border-bottom: none;
        }}
        
        .file-entry:hover {{
            background: rgba(0, 255, 136, 0.05);
            transform: translateX(8px);
        }}
        
        .file-icon {{
            font-size: 1.5rem;
            margin-right: 1rem;
            min-width: 2rem;
            text-align: center;
        }}
        
        .file-name {{
            flex: 1;
            font-weight: 600;
            color: var(--text-color);
        }}
        
        .file-size {{
            color: var(--text-muted);
            font-size: 0.9rem;
            min-width: 100px;
            text-align: right;
        }}
        
        .parent-dir {{
            background: rgba(0, 255, 136, 0.05);
        }}
    </style>
    "#,
        breadcrumb,
        repo.name,
        repo.repo_hash, branch,
        repo.repo_hash, branch,
        repo.repo_hash,
        repo.repo_hash,
        branch,
        repo.repo_hash,
        files_html
    );
    
    render_page(&format!("Files - {}", repo.name), &content)
}

fn render_breadcrumb(repo_hash: &str, current_path: &str, branch: &str) -> String {
    if current_path.is_empty() {
        format!(
            r#"<div class="breadcrumb">
                <a href="/r/{}">← Back to Repository</a>
            </div>"#,
            repo_hash
        )
    } else {
        let parts: Vec<&str> = current_path.split('/').collect();
        let mut path_so_far = String::new();
        let mut breadcrumb_parts = vec![
            format!(r#"<a href="/r/{}/files?branch={}">root</a>"#, repo_hash, branch)
        ];
        
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                path_so_far.push('/');
            }
            path_so_far.push_str(part);
            
            if i == parts.len() - 1 {
                breadcrumb_parts.push(part.to_string());
            } else {
                breadcrumb_parts.push(format!(
                    r#"<a href="/r/{}/files/{}?branch={}">{}</a>"#,
                    repo_hash, urlencoding::encode(&path_so_far), branch, part
                ));
            }
        }
        
        format!(
            r#"<div class="breadcrumb">
                <a href="/r/{}">← Repository</a> / {}
            </div>"#,
            repo_hash,
            breadcrumb_parts.join(" / ")
        )
    }
}

fn render_file_entry(repo: &Repository, branch: &str, file: &FileEntry) -> String {
    let (icon, url) = if file.is_dir {
        ("", format!("/r/{}/files/{}?branch={}", repo.repo_hash, urlencoding::encode(&file.path), branch))
    } else {
        ("", format!("/r/{}/file/{}?branch={}", repo.repo_hash, urlencoding::encode(&file.path), branch))
    };
    
    let size_display = if let Some(size) = file.size {
        format_size(size)
    } else {
        "-".to_string()
    };
    
    format!(
        r#"<a href="{}" class="file-entry">
            <span class="file-icon">{}</span>
            <span class="file-name">{}</span>
            <span class="file-size">{}</span>
        </a>"#,
        url, icon, html_escape(&file.name), size_display
    )
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Hyrule/src/templates/file_view.rs
pub mod file_view {
    use super::{render_page, html_escape};
    use crate::models::Repository;
    
    pub fn render(repo: &Repository, branch: &str, file_path: &str, content: &str) -> String {
        let is_binary = content.chars().any(|c| c == '\0' || (c as u32) < 32 && c != '\n' && c != '\r' && c != '\t');
        
        let content_html = if is_binary {
            "<p class='binary-file'> Binary file - cannot display</p>".to_string()
        } else {
            format!(
                r#"<pre class="file-content"><code>{}</code></pre>"#,
                html_escape(content)
            )
        };
        
        let file_name = file_path.split('/').last().unwrap_or(file_path);
        
        let content_div = format!(
            r#"
        <div class="breadcrumb">
            <a href="/r/{}/files?branch={}">← Back to Files</a>
        </div>
        
        <div class="repo-header">
            <h1>{}</h1>
            <p class="repo-description">{}</p>
        </div>
        
        <div class="repo-nav">
            <a href="/r/{}/files?branch={}" class="nav-tab">Files</a>
            <a href="/r/{}/commits?branch={}" class="nav-tab">Commits</a>
            <a href="/r/{}/branches" class="nav-tab">Branches</a>
            <a href="/r/{}/clone" class="nav-tab">Clone</a>
        </div>
        
        <div class="file-viewer">
            <div class="file-header">
                <span class="file-path">{}</span>
                <span class="file-branch">Branch: {}</span>
            </div>
            <div class="file-content-wrapper">
                {}
            </div>
        </div>
        
        <style>
            .file-viewer {{
                background: var(--bg-glass);
                border: 2px solid var(--border-color);
                border-radius: var(--border-radius);
                overflow: hidden;
                margin-top: 2rem;
            }}
            
            .file-header {{
                background: rgba(0, 255, 136, 0.05);
                padding: 1rem 2rem;
                border-bottom: 1px solid var(--border-color);
                display: flex;
                justify-content: space-between;
                align-items: center;
            }}
            
            .file-path {{
                font-family: 'Courier New', monospace;
                color: var(--primary-color);
                font-weight: 700;
            }}
            
            .file-branch {{
                color: var(--text-muted);
                font-size: 0.9rem;
            }}
            
            .file-content-wrapper {{
                overflow-x: auto;
            }}
            
            .file-content {{
                margin: 0;
                padding: 2rem;
                background: linear-gradient(135deg, #0a1510 0%, #0d1a14 100%);
                color: var(--primary-color);
                font-family: 'Courier New', monospace;
                font-size: 0.95rem;
                line-height: 1.6;
                max-height: 80vh;
                overflow: auto;
            }}
            
            .file-content code {{
                background: none;
                border: none;
                padding: 0;
                color: inherit;
            }}
            
            .binary-file {{
                padding: 3rem;
                text-align: center;
                color: var(--text-muted);
                font-size: 1.2rem;
            }}
        </style>
        "#,
            repo.repo_hash, branch,
            repo.name,
            file_name,
            repo.repo_hash, branch,
            repo.repo_hash, branch,
            repo.repo_hash,
            repo.repo_hash,
            file_path,
            branch,
            content_html
        );
        
        render_page(&format!("{} - {}", file_name, repo.name), &content_div)
    }
}
