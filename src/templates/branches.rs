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
