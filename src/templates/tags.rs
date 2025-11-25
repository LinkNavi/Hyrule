// src/templates/tags.rs
use super::render_page;

pub fn render(tags: &[(String, i64)]) -> String {
    let tags_html = if tags.is_empty() {
        r#"<div class="empty-state">
            <p>No tags found.</p>
        </div>"#.to_string()
    } else {
        tags.iter()
            .map(|(tag, count)| {
                format!(
                    r#"<a href="/search?tag={}" class="tag-card">
                        <span class="tag-name">{}</span>
                        <span class="tag-count">{} repos</span>
                    </a>"#,
                    tag, tag, count
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    let content = format!(
        r#"
    <h1>Browse by Tags</h1>
    <p class="subtitle">Discover repositories by topic</p>
    
    <div class="tags-grid">
        {}
    </div>
    
    <style>
        .tags-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
        }}
        
        .tag-card {{
            background: var(--bg-glass);
            border: 2px solid var(--border-color);
            border-radius: var(--border-radius);
            padding: 2rem;
            text-decoration: none;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 0.5rem;
            transition: all 0.3s ease;
        }}
        
        .tag-card:hover {{
            transform: translateY(-5px);
            border-color: var(--border-glow);
            box-shadow: var(--shadow-md);
        }}
        
        .tag-name {{
            font-size: 1.3rem;
            font-weight: 700;
            color: var(--primary-color);
        }}
        
        .tag-count {{
            color: var(--text-muted);
            font-size: 0.9rem;
        }}
    </style>
    "#,
        tags_html
    );
    
    render_page("Tags", &content)
}
