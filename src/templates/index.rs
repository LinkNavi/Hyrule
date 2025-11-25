use crate::models::User;

// src/templates/index.rs
use super::render_page_with_user;

pub fn render_with_user(username: Option<&str>) -> String {
    let content = r#"
    <div class="hero">
        <h1 class="hero-title">Privacy-First Git Hosting</h1>
        <p class="hero-subtitle">Distributed, anonymous, and censorship-resistant</p>
        
        <div class="hero-features">
            <div class="feature">
                <span class="feature-icon">🔒</span>
                <h3>Anonymous Cloning</h3>
                <p>Clone repositories via Tor or proxy network</p>
            </div>
            <div class="feature">
                <span class="feature-icon">🌐</span>
                <h3>Distributed Storage</h3>
                <p>Replicated across multiple nodes worldwide</p>
            </div>
            <div class="feature">
                <span class="feature-icon">⚡</span>
                <h3>Always Available</h3>
                <p>No single point of failure</p>
            </div>
        </div>
        
        <div class="getting-started">
            <h2>Get Started</h2>
            <div class="code-block">
                <pre><code># Install TriForge CLI
$ cargo install triforge

# Clone a repository anonymously
$ triforge clone repo-hash-here

# Push your code
$ triforge push</code></pre>
            </div>
        </div>
    </div>
    
    <div class="section">
        <h2>Why Hyrule?</h2>
        <ul class="benefits-list">
            <li><strong>Privacy:</strong> No tracking, no analytics, works on Tor</li>
            <li><strong>Resilience:</strong> Auto-replication ensures your code is always available</li>
            <li><strong>Freedom:</strong> Content-addressed repos can't be censored</li>
            <li><strong>Community:</strong> Free tier supported by volunteer node operators</li>
        </ul>
    </div>
    
    <div class="section cta-section">
        <h2>Ready to host your code?</h2>
        <div class="cta-buttons">
            <a href="/signup" class="btn btn-primary">Sign Up Free</a>
            <a href="/explore" class="btn btn-secondary">Explore Repositories</a>
        </div>
    </div>
    "#;
    
    render_page_with_user("Home", content, username)
}
