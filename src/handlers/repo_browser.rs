// Hyrule/src/handlers/repo_browser.rs - SECURITY FIXES
use crate::templates;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use std::process::Command;
use std::sync::Arc;

async fn get_session_user_id(state: &Arc<AppState>, jar: &CookieJar) -> Option<i64> {
    if let Some(session_id) = jar.get("session_id") {
        if let Some(session) = state.session_store.get_session(session_id.value()).await {
            return Some(session.user_id);
        }
    }
    None
}

#[derive(Debug, Deserialize)]
pub struct BranchQuery {
    pub branch: Option<String>,
}

#[derive(Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<usize>,
}

#[derive(Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub message: String,
}

pub async fn view_repo(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(repo_hash): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash format
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let replica_count = state.db.get_replica_count(&repo_hash).await.unwrap_or(0);
    let nodes = state
        .db
        .list_repo_replicas(&repo_hash)
        .await
        .unwrap_or_default();

    Ok(Html(templates::repo::render(&repo, replica_count, &nodes)))
}

pub async fn browse_files(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(repo_hash): Path<String>,
    Query(query): Query<BranchQuery>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let branch = query.branch.unwrap_or_else(|| "main".to_string());
    
    // SECURITY: Validate branch name
    if !crate::utils::validation::validate_ref_name(&branch) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    let files = list_files_in_repo(&repo_path, &branch, "")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(templates::files::render(&repo, &branch, "", &files)))
}

pub async fn browse_directory(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path((repo_hash, path)): Path<(String, String)>,
    Query(query): Query<BranchQuery>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash first
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        tracing::warn!("Invalid repo hash attempted: {}", repo_hash);
        return Err(StatusCode::BAD_REQUEST);
    }

    // SECURITY: Validate path to prevent traversal - CRITICAL
    if !crate::utils::validation::is_safe_path(&path) {
        tracing::warn!("Path traversal attempt detected: {}", path);
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let branch = query.branch.unwrap_or_else(|| "main".to_string());
    
    // SECURITY: Validate branch name
    if !crate::utils::validation::validate_ref_name(&branch) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    let files = list_files_in_repo(&repo_path, &branch, &path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(templates::files::render(&repo, &branch, &path, &files)))
}

pub async fn view_file(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path((repo_hash, file_path)): Path<(String, String)>,
    Query(query): Query<BranchQuery>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // SECURITY: Validate file path
    if !crate::utils::validation::is_safe_path(&file_path) {
        tracing::warn!("Unsafe file path attempted: {}", file_path);
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let branch = query.branch.unwrap_or_else(|| "main".to_string());
    
    // SECURITY: Validate branch name
    if !crate::utils::validation::validate_ref_name(&branch) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    let content =
        read_file_from_repo(&repo_path, &branch, &file_path)
            .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Html(templates::file_view::render(
        &repo, &branch, &file_path, &content,
    )))
}

pub async fn list_commits(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(repo_hash): Path<String>,
    Query(query): Query<BranchQuery>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let branch = query.branch.unwrap_or_else(|| "main".to_string());
    
    // SECURITY: Validate branch name
    if !crate::utils::validation::validate_ref_name(&branch) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    let commits = get_commits(&repo_path, &branch, 50)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(templates::commits::render(&repo, &branch, &commits)))
}

pub async fn view_commit(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path((repo_hash, commit_hash)): Path<(String, String)>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // SECURITY: Validate commit hash (40 hex chars)
    if !crate::utils::validation::validate_object_id(&commit_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    let commit_info =
        get_commit_info(&repo_path, &commit_hash).map_err(|_| StatusCode::NOT_FOUND)?;
    let diff =
        get_commit_diff(&repo_path, &commit_hash)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(templates::commits::commit_view::render(
        &repo,
        &commit_hash,
        &commit_info,
        &diff,
    )))
}

pub async fn list_branches(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(repo_hash): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // SECURITY: Validate repo hash
    if !crate::utils::validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user_id = get_session_user_id(&state, &jar)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    let branches = get_branches(&repo_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(templates::commits::branches::render(&repo, &branches)))
}

// Helper functions - these interact with git directly, so we need to be careful

fn list_files_in_repo(
    repo_path: &std::path::Path,
    branch: &str,
    subpath: &str,
) -> Result<Vec<FileEntry>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    // SECURITY: Construct path argument safely
    let path_arg = if subpath.is_empty() {
        format!("{}:", branch)
    } else {
        format!("{}:{}", branch, subpath)
    };

    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("ls-tree")
        .arg("-l")
        .arg(&path_arg)
        .output()?;

    if !output.status.success() {
        tracing::error!("git ls-tree failed for {:?}", repo_path);
        tracing::error!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(files);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let obj_type = parts[1];
            let size_str = parts[3];
            let name = parts[4..].join(" ");

            let is_dir = obj_type == "tree";
            let size = if size_str == "-" {
                None
            } else {
                size_str.parse().ok()
            };

            let full_path = if subpath.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", subpath, name)
            };

            files.push(FileEntry {
                name,
                path: full_path,
                is_dir,
                size,
            });
        }
    }

    Ok(files)
}

fn read_file_from_repo(
    repo_path: &std::path::Path,
    branch: &str,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let path_arg = format!("{}:{}", branch, file_path);

    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("show")
        .arg(&path_arg)
        .output()?;

    if !output.status.success() {
        return Err("File not found".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn get_commits(
    repo_path: &std::path::Path,
    branch: &str,
    limit: usize,
) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("log")
        .arg(branch)
        .arg("--format=%H%x00%an%x00%ae%x00%at%x00%s")
        .arg(format!("-{}", limit))
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\0').collect();
        if parts.len() >= 5 {
            commits.push(CommitInfo {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                email: parts[2].to_string(),
                timestamp: parts[3].parse().unwrap_or(0),
                message: parts[4].to_string(),
            });
        }
    }

    Ok(commits)
}

fn get_commit_info(
    repo_path: &std::path::Path,
    commit_hash: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("show")
        .arg("--no-patch")
        .arg("--format=Author: %an <%ae>%nDate: %ad%n%n%B")
        .arg(commit_hash)
        .output()?;

    if !output.status.success() {
        return Err("Commit not found".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn get_commit_diff(
    repo_path: &std::path::Path,
    commit_hash: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("show")
        .arg("--format=")
        .arg(commit_hash)
        .output()?;

    if !output.status.success() {
        return Err("Commit not found".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn get_branches(repo_path: &std::path::Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_path)
        .arg("branch")
        .arg("--list")
        .output()?;

    if !output.status.success() {
        return Ok(vec!["main".to_string()]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout.lines().map(|line| line.trim().to_string()).collect();

    Ok(branches)
}
