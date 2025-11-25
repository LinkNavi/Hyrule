// src/templates/about.rs
use super::render_page_with_user;

pub fn render() -> String {
    render_with_user(None)
}

pub fn render_with_user(username: Option<&str>) -> String {
    let content = r#"
    <h1>ℹ️ About Hyrule</h1>
    
    <div class="section">
        <h2>What is Hyrule?</h2>
        <p>
            Hyrule is a distributed Git hosting platform designed for privacy, 
            resilience, and censorship resistance. Unlike traditional Git hosts,
            your repositories are replicated across a network of volunteer nodes,
            ensuring they remain available even if individual nodes go offline.
        </p>
    </div>
    
    <div class="section">
        <h2>How It Works</h2>
        <ol>
            <li><strong>Content Addressing:</strong> Repositories are identified by cryptographic hashes</li>
            <li><strong>Distributed Storage:</strong> Data is replicated across multiple nodes</li>
            <li><strong>DHT Coordination:</strong> A distributed hash table helps locate replicas</li>
            <li><strong>Auto-Healing:</strong> System maintains minimum replica count automatically</li>
        </ol>
    </div>
    
    <div class="section">
        <h2>Privacy Features</h2>
        <ul>
            <li>Anonymous cloning via Tor integration</li>
            <li>No tracking or analytics</li>
            <li>End-to-end encryption for private repos</li>
            <li>Onion routing for multi-hop privacy</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Get Involved</h2>
        <p>
            Hyrule is open source and community-driven. You can contribute by:
        </p>
        <ul>
            <li>Running a storage node</li>
            <li>Contributing code on GitHub</li>
            <li>Pinning repositories you care about</li>
            <li>Spreading the word</li>
        </ul>
    </div>
    "#;
    
    render_page_with_user("About", content, username)
}
