// Hyrule/src/handlers/web_enhanced.rs
use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, Redirect},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::session::SessionUser;
use crate::templates;
use crate::AppState;

// Helper function to get session user from cookie
async fn get_session_user(state: &Arc<AppState>, jar: &CookieJar) -> Option<(i64, String)> {
    if let Some(session_id) = jar.get("session_id") {
        if let Some(session) = state.session_store.get_session(session_id.value()).await {
            return Some((session.user_id, session.username));
        }
    }
    None
}

// Dashboard with full functionality
pub async fn dashboard_enhanced(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    // Get session from cookie
    let (user_id, username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    let repos = state
        .db
        .list_user_repositories(user_id)
        .await
        .unwrap_or_default();

    // Get user stats
    let pinned_count = state
        .db
        .get_pinned_repositories(user_id)
        .await
        .map(|r| r.len())
        .unwrap_or(0);

    let starred_count = state
        .db
        .get_starred_repositories(user_id)
        .await
        .map(|r| r.len())
        .unwrap_or(0);

    Ok(Html(templates::dashboard_enhanced::render(
        &repos,
        &username,
        pinned_count,
        starred_count,
    )))
}

fn redirect_to_login() -> String {
    templates::render_page(
        "Login Required",
        r#"<div class="section">
            <h1>Login Required</h1>
            <p>Please login to access the dashboard.</p>
            <a href="/login" class="btn btn-primary">Go to Login</a>
        </div>"#,
    )
}

// Enhanced repository view with actions
pub async fn repo_enhanced(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    jar: CookieJar,
) -> Result<Html<String>, StatusCode> {
    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    let maybe_user = get_session_user(&state, &jar).await;
    if repo.is_private != 0 {
        let (user_id, _) = maybe_user.as_ref().ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != *user_id {
            // Note the dereference here
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let owner = state
        .db
        .get_user_by_id(repo.owner_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let replica_count = state.db.get_replica_count(&repo_hash).await.unwrap_or(0);
    let nodes = state
        .db
        .list_repo_replicas(&repo_hash)
        .await
        .unwrap_or_default();
    let tags = state.db.get_repo_tags(&repo_hash).await.unwrap_or_default();
    let star_count = state.db.get_repo_star_count(&repo_hash).await.unwrap_or(0);

    // Check if user is logged in and owns repo
    let (is_owner, is_starred, is_pinned) = if let Some((user_id, _)) = maybe_user {
        let owns = repo.owner_id == user_id;
        let starred = state
            .db
            .has_starred(&repo_hash, user_id)
            .await
            .unwrap_or(false);
        let pinned = state
            .db
            .has_pinned(&repo_hash, user_id)
            .await
            .unwrap_or(false);
        (owns, starred, pinned)
    } else {
        (false, false, false)
    };

    // Try to fetch README
    let readme_html = {
        use crate::auth::OptionalAuthUser;
        use crate::handlers::api_complete::get_repo_readme;

        // For readme, pass None for auth since we already checked access above
        match get_repo_readme(State(state.clone()), Path(repo_hash.clone())).await {
            Ok(html) => Some(html),
            Err(_) => None,
        }
    };

    Ok(Html(
        templates::repo_enhanced::render_with_readme(
            &repo,
            &owner.username,
            replica_count,
            &nodes,
            &tags,
            star_count,
            is_owner,
            is_starred,
            is_pinned,
            readme_html,
        )
        .await,
    ))
}

// Repository actions (star/unstar/pin/unpin)
#[derive(Debug, Deserialize)]
pub struct RepoAction {
    pub action: String,
    pub repo_hash: String,
}

pub async fn repo_action(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<RepoAction>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let (user_id, _username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    match form.action.as_str() {
        "star" => {
            state
                .db
                .star_repository(&form.repo_hash, user_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(error_page("Failed to star repository")),
                    )
                })?;
        }
        "unstar" => {
            state
                .db
                .unstar_repository(&form.repo_hash, user_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(error_page("Failed to unstar repository")),
                    )
                })?;
        }
        "pin" => {
            state
                .db
                .pin_repository(&form.repo_hash, user_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(error_page("Failed to pin repository")),
                    )
                })?;
        }
        "unpin" => {
            state
                .db
                .unpin_repository(&form.repo_hash, user_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(error_page("Failed to unpin repository")),
                    )
                })?;
        }
        "delete" => {
            // Verify ownership
            let repo = state
                .db
                .get_repository(&form.repo_hash)
                .await
                .map_err(|_| {
                    (
                        StatusCode::NOT_FOUND,
                        Html(error_page("Repository not found")),
                    )
                })?;

            if repo.owner_id != user_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    Html(error_page("You don't own this repository")),
                ));
            }

            state
                .db
                .delete_repository_complete(&form.repo_hash)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(error_page("Failed to delete repository")),
                    )
                })?;

            let _ = state.git_storage.delete_repo(&form.repo_hash);

            return Ok(Redirect::to("/dashboard"));
        }
        _ => return Err((StatusCode::BAD_REQUEST, Html(error_page("Invalid action")))),
    }

    Ok(Redirect::to(&format!("/r/{}", form.repo_hash)))
}

fn error_page(message: &str) -> String {
    templates::render_page(
        "Error",
        &format!(
            r#"<div class="section">
                <h1>Error</h1>
                <p>{}</p>
                <a href="javascript:history.back()" class="btn btn-secondary">Go Back</a>
            </div>"#,
            message
        ),
    )
}

// Create repository form
pub async fn create_repo_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, StatusCode> {
    // Check if logged in
    if get_session_user(&state, &jar).await.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Html(templates::create_repo::render()))
}

#[derive(Debug, Deserialize)]
pub struct ForkRepoForm {
    pub repo_hash: String,
    pub new_name: Option<String>,
    pub description: Option<String>,
}

pub async fn fork_repo_form(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<ForkRepoForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let (user_id, _username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    // Get original repository
    let original_repo = state
        .db
        .get_repository(&form.repo_hash)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Html(error_page("Repository not found")),
            )
        })?;

    // Generate new hash for fork
    let fork_name = form
        .new_name
        .unwrap_or(format!("{}-fork", original_repo.name));
    let fork_hash = crate::utils::hash::generate_repo_hash(&fork_name, user_id);

    // Copy repository data
    let original_path = state.git_storage.repo_path(&form.repo_hash);
    let fork_path = state.git_storage.repo_path(&fork_hash);

    std::fs::create_dir_all(&fork_path).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(error_page("Failed to create fork directory")),
        )
    })?;

    // Copy all git objects
    copy_dir_recursive(&original_path, &fork_path).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(error_page("Failed to copy repository")),
        )
    })?;

    // Create database entry
    let fork_request = crate::models::CreateRepoRequest {
        name: fork_name.clone(),
        description: form
            .description
            .or(Some(format!("Fork of {}", original_repo.name))),
        storage_tier: "free".to_string(),
        is_private: original_repo.is_private != 0,
    };

    state
        .db
        .create_repository(&fork_request, user_id, &fork_hash)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(error_page("Failed to create repository")),
            )
        })?;

    Ok(Redirect::to(&format!("/r/{}", fork_hash)))
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoForm {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<String>,
}

pub async fn create_repo_submit(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<CreateRepoForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let (user_id, _username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    // Validate
    if form.name.len() < 3 || form.name.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Html(error_page("Repository name must be 3-64 characters")),
        ));
    }

    let repo_hash = crate::utils::hash::generate_repo_hash(&form.name, user_id);

    let create_req = crate::models::CreateRepoRequest {
        name: form.name,
        description: form.description,
        storage_tier: "free".to_string(),
        is_private: form.is_private.is_some(),
    };

    state
        .db
        .create_repository(&create_req, user_id, &repo_hash)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(error_page("Failed to create repository")),
            )
        })?;

    state.git_storage.init_repo(&repo_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(error_page("Failed to initialize repository storage")),
        )
    })?;

    Ok(Redirect::to(&format!("/r/{}", repo_hash)))
}

// Search page
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub tag: Option<String>,
}

pub async fn search_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Html<String>, StatusCode> {
    let (results, search_term) = if let Some(q) = query.q {
        let repos = state
            .db
            .search_repositories(&q, 50)
            .await
            .unwrap_or_default();
        (repos, q)
    } else if let Some(tag) = query.tag {
        let repos = state.db.get_repos_by_tag(&tag).await.unwrap_or_default();
        (repos, format!("tag:{}", tag))
    } else {
        (vec![], String::new())
    };

    Ok(Html(templates::search::render(&results, &search_term)))
}

// User profile
pub async fn profile_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (user_id, _username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    let user = state
        .db
        .get_user_by_id(user_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, Html(error_page("User not found"))))?;

    let repos = state
        .db
        .list_user_repositories(user_id)
        .await
        .unwrap_or_default();

    let starred = state
        .db
        .get_starred_repositories(user_id)
        .await
        .unwrap_or_default();

    let pinned = state
        .db
        .get_pinned_repositories(user_id)
        .await
        .unwrap_or_default();

    Ok(Html(templates::profile::render(
        &user, &repos, &starred, &pinned,
    )))
}

// Tags page
pub async fn tags_page(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let tags = state.db.get_all_tags().await.unwrap_or_default();

    Ok(Html(templates::tags::render(&tags)))
}

// Starred repositories
pub async fn starred_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (user_id, _username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    let starred = state
        .db
        .get_starred_repositories(user_id)
        .await
        .unwrap_or_default();

    Ok(Html(templates::starred::render(&starred)))
}

// Pinned repositories
pub async fn pinned_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let (user_id, _username) = get_session_user(&state, &jar)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Html(redirect_to_login())))?;

    let pinned = state
        .db
        .get_pinned_repositories(user_id)
        .await
        .unwrap_or_default();

    Ok(Html(templates::pinned::render(&pinned)))
}
