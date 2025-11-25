// src/templates/login.rs
use super::render_page_with_user;

pub fn render(success_msg: Option<&str>) -> String {
    render_with_user(success_msg, None)
}

pub fn render_with_user(success_msg: Option<&str>, username: Option<&str>) -> String {
    let success_html = if let Some(msg) = success_msg {
        format!(r#"<div class="success-message">✅ {}</div>"#, msg)
    } else {
        String::new()
    };
    
    let content = format!(
        r#"
    <div class="auth-container">
        <h1>🔐 Login to Hyrule</h1>
        {}
        <form class="auth-form" method="POST" action="/login">
            <div class="form-group">
                <label for="username">Username</label>
                <input type="text" id="username" name="username" required autocomplete="username">
            </div>
            <div class="form-group">
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required autocomplete="current-password">
            </div>
            <button type="submit" class="btn btn-primary btn-full">Login</button>
        </form>
        <p class="auth-footer">
            Don't have an account? <a href="/signup">Sign up</a>
        </p>
    </div>
    
    <style>
        .success-message {{
            background: rgba(0, 255, 136, 0.1);
            border: 2px solid rgba(0, 255, 136, 0.3);
            border-radius: 15px;
            padding: 1rem 2rem;
            margin: 0 0 2rem 0;
            color: var(--primary-color);
            text-align: center;
        }}
    </style>
    "#,
        success_html
    );
    
    render_page_with_user("Login", &content, username)
}
