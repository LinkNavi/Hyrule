// src/templates/signup.rs
use super::render_page_with_user;

pub fn render() -> String {
    render_with_user(None)
}

pub fn render_with_user(username: Option<&str>) -> String {
    let content = r#"
    <div class="auth-container">
        <h1>📝 Create Account</h1>
        <form class="auth-form" method="POST" action="/signup">
            <div class="form-group">
                <label for="username">Username</label>
                <input type="text" id="username" name="username" required minlength="3" maxlength="32" autocomplete="username">
                <small style="color: var(--text-muted); display: block; margin-top: 0.5rem;">3-32 characters</small>
            </div>
            <div class="form-group">
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required minlength="8" autocomplete="new-password">
                <small style="color: var(--text-muted); display: block; margin-top: 0.5rem;">At least 8 characters</small>
            </div>
            <button type="submit" class="btn btn-primary btn-full">Sign Up</button>
        </form>
        <p class="auth-footer">
            Already have an account? <a href="/login">Login</a>
        </p>
    </div>
    
    <style>
        .error-container {
            max-width: 600px;
            margin: 2rem auto;
            text-align: center;
        }
        
        .error-message {
            background: rgba(255, 51, 102, 0.1);
            border: 2px solid rgba(255, 51, 102, 0.3);
            border-radius: 15px;
            padding: 2rem;
            margin: 2rem 0;
        }
        
        .error-message p {
            color: #ff3366;
            font-size: 1.1rem;
            margin: 0;
        }
        
        .success-message {
            background: rgba(0, 255, 136, 0.1);
            border: 2px solid rgba(0, 255, 136, 0.3);
            border-radius: 15px;
            padding: 1rem 2rem;
            margin: 1rem 0;
            color: var(--primary-color);
        }
    </style>
    "#;
    
    render_page_with_user("Sign Up", content, username)
}
