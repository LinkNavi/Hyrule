// Hyrule/src/templates/docs.rs - FIXED VERSION
use super::render_page_with_user;

pub fn render() -> String {
    render_with_user(None)
}

pub fn render_with_user(username: Option<&str>) -> String {
    let content = r#"
    <h1>📚 TriForge CLI Documentation</h1>
    
    <div class="docs-section">
        <h2>Installation</h2>
        <div class="code-block">
            <pre><code>$ cargo install triforge</code></pre>
        </div>
        <p>Requires: Rust toolchain (cargo)</p>
    </div>
    
    <div class="docs-section">
        <h2>Authentication</h2>
        <div class="code-block">
            <pre><code># Create a new account
$ triforge signup

# Login to existing account  
$ triforge login

# Logout
$ triforge logout</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Repository Management</h2>
        
        <h3>Initialize & Commit</h3>
        <div class="code-block">
            <pre><code># Initialize new repository
$ triforge init
$ triforge init --name "my-project" --description "My awesome project"

# Add files
$ triforge add file.txt
$ triforge add .        # Add all files
$ triforge add --all    # Add all files

# Commit changes
$ triforge commit -m "Commit message"
$ triforge commit -m "Message" --all

# View status
$ triforge status
$ triforge status --short

# View commit history
$ triforge log
$ triforge log --limit 20
$ triforge log --oneline</code></pre>
        </div>
        
        <h3>Branches</h3>
        <div class="code-block">
            <pre><code># List branches
$ triforge branch

# Create branch
$ triforge branch create feature-x

# Switch branches
$ triforge checkout main
$ triforge checkout -b new-branch  # Create and switch

# Delete branch
$ triforge branch delete old-branch
$ triforge branch delete --force old-branch</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Network Operations</h2>
        
        <h3>Push to Hyrule</h3>
        <div class="code-block">
            <pre><code># Push public repository
$ triforge push

# Push with metadata
$ triforge push --name "my-repo" --description "Description"

# Push private repository
$ triforge push --private

# Get repository hash
$ triforge hash</code></pre>
        </div>
        
        <h3>Clone from Hyrule</h3>
        <div class="code-block">
            <pre><code># Clone by hash
$ triforge clone &lt;repo-hash&gt;

# Clone to specific directory
$ triforge clone &lt;repo-hash&gt; my-dir

# Anonymous clone (via Tor - coming soon)
$ triforge clone --anonymous &lt;repo-hash&gt;</code></pre>
        </div>
        
        <h3>Pull Changes</h3>
        <div class="code-block">
            <pre><code># Pull from default remote
$ triforge pull

# Pull from specific remote
$ triforge pull origin</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Repository Discovery</h2>
        <div class="code-block">
            <pre><code># List your repositories
$ triforge list
$ triforge list --starred
$ triforge list --pinned

# Search repositories
$ triforge search "rust"
$ triforge search "web" --tags frontend,javascript
$ triforge search --user alice

# View trending
$ triforge trending
$ triforge trending --limit 20

# View popular
$ triforge popular
$ triforge popular --limit 10

# Get repository info
$ triforge info &lt;repo-hash&gt;</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Social Features</h2>
        <div class="code-block">
            <pre><code># Star repositories
$ triforge star &lt;repo-hash&gt;
$ triforge unstar &lt;repo-hash&gt;

# Pin to profile
$ triforge pin &lt;repo-hash&gt;
$ triforge unpin &lt;repo-hash&gt;

# Fork repository
$ triforge fork &lt;repo-hash&gt;
$ triforge fork &lt;repo-hash&gt; --name "my-fork"

# Delete repository
$ triforge delete &lt;repo-hash&gt;
$ triforge delete &lt;repo-hash&gt; --force</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Tags</h2>
        <div class="code-block">
            <pre><code># Add tags to repository
$ triforge tags add &lt;repo-hash&gt; rust cli tool

# List all tags
$ triforge tags list

# List tags for repository
$ triforge tags list &lt;repo-hash&gt;

# Find repos by tag
$ triforge tags search rust</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Network & Admin</h2>
        <div class="code-block">
            <pre><code># View network statistics
$ triforge stats

# List storage nodes
$ triforge nodes

# Verify repository
$ triforge verify
$ triforge verify --fix</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Configuration</h2>
        <div class="code-block">
            <pre><code># Show configuration
$ triforge config show

# Set values
$ triforge config set server http://localhost:3000
$ triforge config set private true

# Get values
$ triforge config get server</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Remotes</h2>
        <div class="code-block">
            <pre><code># Add remote
$ triforge remote add origin &lt;repo-hash&gt;

# List remotes
$ triforge remote list

# Remove remote
$ triforge remote remove origin</code></pre>
        </div>
    </div>
    
    <div class="docs-section">
        <h2>Standard Git Compatibility</h2>
        <p>You can also use standard Git commands with Hyrule:</p>
        <div class="code-block">
            <pre><code># Clone via Git Smart HTTP
$ git clone http://localhost:3000/git/&lt;repo-hash&gt;.git

# Push via Git
$ git push origin main

# Pull via Git
$ git pull origin main</code></pre>
        </div>
        <p><strong>Note:</strong> Authentication for Git operations uses the same session cookies or JWT tokens from the web interface.</p>
    </div>
    
    <div class="docs-section">
        <h2>Advanced Features</h2>
        <ul>
            <li><strong>File Browser:</strong> View files, commits, and branches on the web interface</li>
            <li><strong>README Display:</strong> Markdown READMEs are rendered on repository pages</li>
            <li><strong>Health Monitoring:</strong> Automatic replication health checks</li>
            <li><strong>Replica Management:</strong> Minimum 3 replicas maintained automatically</li>
            <li><strong>Content Addressing:</strong> Repos identified by BLAKE3 hash</li>
        </ul>
    </div>
    "#;
    
    render_page_with_user("Documentation", content, username)
}
