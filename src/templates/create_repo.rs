// src/templates/create_repo.rs
use super::render_page;

pub fn render() -> String {
    let content = r#"
    <h1>Create New Repository</h1>
    
    <form method="POST" action="/repos/new" class="repo-form">
        <div class="form-group">
            <label for="name">Repository Name *</label>
            <input type="text" id="name" name="name" required 
                   minlength="3" maxlength="64" placeholder="my-awesome-project">
            <small>3-64 characters, letters, numbers, hyphens allowed</small>
        </div>
        
        <div class="form-group">
            <label for="description">Description</label>
            <textarea id="description" name="description" rows="4" 
                      placeholder="A short description of your project..."></textarea>
        </div>
        
        <div class="form-group">
            <label>
                <input type="checkbox" name="is_private" value="true">
                Make this repository private
            </label>
            <small>Private repositories are only accessible to you</small>
        </div>
        
        <div class="form-actions">
            <button type="submit" class="btn btn-primary">Create Repository</button>
            <a href="/dashboard" class="btn btn-secondary">Cancel</a>
        </div>
    </form>
    
    <style>
        .repo-form {
            max-width: 700px;
            margin: 2rem auto;
            background: var(--bg-glass);
            padding: 3rem;
            border-radius: var(--border-radius);
            border: 2px solid var(--border-color);
        }
        
        textarea {
            width: 100%;
            padding: 1rem;
            border: 2px solid var(--border-color);
            border-radius: 14px;
            background: var(--bg-glass-dark);
            color: var(--text-color);
            font-family: inherit;
            resize: vertical;
        }
        
        textarea:focus {
            outline: none;
            border-color: var(--primary-color);
            box-shadow: 0 0 20px rgba(0, 255, 136, 0.3);
        }
        
        .form-group small {
            display: block;
            margin-top: 0.5rem;
            color: var(--text-muted);
        }
        
        .form-group label input[type="checkbox"] {
            margin-right: 0.5rem;
        }
        
        .form-actions {
            display: flex;
            gap: 1rem;
            margin-top: 2rem;
        }
    </style>
    "#;
    
    render_page("Create Repository", content)
}
