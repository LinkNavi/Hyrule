// src/templates/mod.rs
pub mod index;
pub mod explore;
pub mod repo;
pub mod dashboard;
pub mod login;
pub mod signup;
pub mod docs;
pub mod about;
pub mod files;
pub mod commits;
pub mod clone_page;
pub mod admin;
pub mod admin_enhanced;

// NEW Enhanced templates
pub mod dashboard_enhanced;
pub mod repo_enhanced;
pub mod create_repo;
pub mod search;
pub mod tags;
pub mod starred;
pub mod pinned;
pub mod profile;

mod layout;

pub use layout::{render_page, render_page_with_user};
pub use files::file_view;

// Helper function for HTML escaping
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
