// Hyrule/src/templates/clone_page.rs
use super::render_page;
use crate::models::Repository;

pub fn render(repo: &Repository, server_url: &str) -> String {
    let git_url = format!("{}/git/{}.git", server_url, repo.repo_hash);
    let http_url = format!("{}/r/{}/download", server_url, repo.repo_hash);
    
    let content = format!(
        r#"
    <div class="breadcrumb">
        <a href="/r/{}">← Back to Repository</a>
    </div>
    
    <div class="repo-header">
        <h1>{}</h1>
        <p class="repo-description">Clone Instructions</p>
    </div>
    
    <div class="repo-nav">
        <a href="/r/{}/files" class="nav-tab">Files</a>
        <a href="/r/{}/commits" class="nav-tab">Commits</a>
        <a href="/r/{}/branches" class="nav-tab">Branches</a>
        <a href="/r/{}/clone" class="nav-tab active">Clone</a>
    </div>
    
    <div class="clone-instructions">
        <h2> Clone via Git (Smart HTTP)</h2>
        <div class="clone-method">
            <p>Standard git clone (works with any git client):</p>
            <div class="code-block">
                <pre><code>git clone {}</code></pre>
            </div>
            <button onclick="navigator.clipboard.writeText('git clone {}')" class="btn btn-primary">Copy Command</button>
            <noscript><p class="hint"> Copy the command above manually</p></noscript>
        </div>
        
        <h2> Direct Download (No Git Required)</h2>
        <div class="clone-method">
            <p>Download repository as a bundle:</p>
            <div class="action-box">
                <a href="{}" class="btn btn-primary" download>Download .bundle File</a>
            </div>
            <p class="hint"> To use: <code>git clone {}.bundle</code></p>
        </div>
        
        <h2> Clone via Tor (Anonymous)</h2>
        <div class="clone-method">
            <p>For anonymous cloning, use Tor:</p>
            <div class="code-block">
                <pre><code># Configure git to use Tor SOCKS proxy
git config --global http.proxy socks5h://127.0.0.1:9050

# Clone anonymously
git clone {}

# Reset proxy after cloning
git config --global --unset http.proxy</code></pre>
            </div>
            <p class="hint"> Requires Tor Browser or tor daemon running on port 9050</p>
        </div>
        
        <h2> Repository Information</h2>
        <div class="info-grid">
            <div class="info-card">
                <div class="info-label">Repository Hash</div>
                <div class="info-value"><code>{}</code></div>
            </div>
            <div class="info-card">
                <div class="info-label">Size</div>
                <div class="info-value">{} KB</div>
            </div>
            <div class="info-card">
                <div class="info-label">Visibility</div>
                <div class="info-value">{}</div>
            </div>
        </div>
        
        <h2> Push Changes</h2>
        <div class="clone-method">
            <p>After cloning, you can push changes:</p>
            <div class="code-block">
                <pre><code># Make your changes
git add .
git commit -m "Your changes"

# Push to Hyrule
git push origin main</code></pre>
            </div>
            <p class="hint"> Requires authentication. Make sure you're logged in.</p>
        </div>
    </div>
    
    <style>
        .clone-instructions {{
            max-width: 900px;
        }}
        
        .clone-instructions h2 {{
            color: var(--primary-color);
            margin-top: 3rem;
            margin-bottom: 1.5rem;
            font-size: 1.8rem;
        }}
        
        .clone-method {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 2rem;
            margin-bottom: 2rem;
        }}
        
        .clone-method p {{
            color: var(--text-secondary);
            margin-bottom: 1rem;
        }}
        
        .action-box {{
            margin: 1.5rem 0;
            text-align: center;
        }}
        
        .info-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-top: 1.5rem;
        }}
        
        .info-card {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 1.5rem;
        }}
        
        .info-label {{
            color: var(--text-muted);
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 0.5rem;
        }}
        
        .info-value {{
            color: var(--text-color);
            font-size: 1.2rem;
            font-weight: 600;
        }}
        
        .info-value code {{
            font-size: 1rem;
        }}
    </style>
    "#,
        repo.repo_hash,
        repo.name,
        repo.repo_hash,
        repo.repo_hash,
        repo.repo_hash,
        repo.repo_hash,
        git_url, git_url,
        http_url,
        repo.name,
        git_url,
        repo.repo_hash,
        repo.size / 1024,
        if repo.is_private != 0 { " Private" } else { " Public" }
    );
    
    render_page(&format!("Clone - {}", repo.name), &content)
}


