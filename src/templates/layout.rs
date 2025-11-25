// src/templates/layout.rs

pub fn render_page(title: &str, content: &str) -> String {
    render_page_with_user(title, content, None)
}

pub fn render_page_with_user(title: &str, content: &str, username: Option<&str>) -> String {
    let auth_links = if let Some(user) = username {
        format!(
            r#"<span class="logged-in-indicator">🟢 {}</span>
            <a href="/dashboard">Dashboard</a>
            {}
            <form method="POST" action="/logout" style="display:inline;margin:0;">
                <button type="submit" class="btn-link">Logout</button>
            </form>"#,
            user,
            if user == "admin" || user == "Link" {
                r#"<a href="/admin">Admin</a>"#
            } else {
                ""
            }
        )
    } else {
        r#"<a href="/login">Login</a>
           <a href="/signup">Sign Up</a>"#.to_string()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{} - Hyrule</title>
    <link rel="stylesheet" href="/static/css/main.css">
</head>
<body>
    <header class="header">
        <div class="container">
            <div class="header-content">
                <div class="logo">
                    <a href="/">Hyrule</a>
                </div>
                <nav class="nav">
                    <a href="/">Home</a>
                    <a href="/explore">Explore</a>
                    <a href="/docs">Docs</a>
                    {}
                </nav>
            </div>
        </div>
    </header>
    
    <main class="main">
        <div class="container">
            {}
        </div>
    </main>
    
    <footer class="footer">
        <div class="container">
            <p>Hyrule - Distributed Git Hosting</p>
            <p>
                <a href="/about">About</a> · 
                <a href="/docs">Documentation</a> · 
                <a href="https://github.com/hyrule">GitHub</a>
            </p>
        </div>
    </footer>
</body>
</html>"#,
        title, auth_links, content
    )
}

pub fn nav_link(href: &str, text: &str, active: bool) -> String {
    let class = if active { "nav-link active" } else { "nav-link" };
    format!(r#"<a href="{}" class="{}">{}</a>"#, href, class, text)
}
