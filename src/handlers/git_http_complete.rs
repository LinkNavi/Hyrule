// Hyrule/src/handlers/git_http_complete.rs
use axum::body::Bytes;
use axum::{
    body::Body,

    extract::{Path, Query, State},
    http::{header, StatusCode, HeaderMap},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GitService {
    service: String,
}

/// Helper to extract user_id from authentication
async fn get_authenticated_user_id(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Option<i64> {
    // Try cookie-based session first
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "session_id" {
                    if let Some(session) = state.session_store.get_session(parts[1]).await {
                        return Some(session.user_id);
                    }
                }
            }
        }
    }
    
    // Try Authorization header (Bearer token)
    if let Some(auth) = headers.get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if let Ok(claims) = crate::auth::jwt::validate_token(token) {
                    return Some(claims.sub);
                }
            }
        }
    }
    
    None
}

/// Git info/refs endpoint - handles auth for private repos
pub async fn git_info_refs(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    Query(params): Query<GitService>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check authentication for private repos or receive-pack
    if repo.is_private != 0 || params.service == "git-receive-pack" {
        let user_id = get_authenticated_user_id(&state, &headers)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    if !repo_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let service = &params.service;
    let git_command = if service == "git-upload-pack" {
        "upload-pack"
    } else if service == "git-receive-pack" {
        "receive-pack"
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let output = Command::new("git")
        .arg(git_command)
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.status.success() {
        eprintln!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut response_body = Vec::new();
    let service_line = format!("# service={}\n", service);
    response_body.extend_from_slice(format_pkt_line(&service_line).as_bytes());
    response_body.extend_from_slice(b"0000");
    response_body.extend_from_slice(&output.stdout);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            format!("application/x-{}-advertisement", service),
        )
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(response_body))
        .unwrap())
}

/// Handle git-upload-pack (clone/fetch) - respects privacy
pub async fn git_upload_pack(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    headers: HeaderMap,
    body: Bytes, // Changed from Vec<u8>
) -> Result<Response, StatusCode> {
    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check authentication for private repos
    if repo.is_private != 0 {
        let user_id = get_authenticated_user_id(&state, &headers)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);

    let mut child = Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&body) // body is already &[u8] via Bytes deref
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let output = child
        .wait_with_output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.status.success() {
        eprintln!(
            "git-upload-pack failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}

/// Handle git-receive-pack (push) - REQUIRES AUTHENTICATION
pub async fn git_receive_pack(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    headers: HeaderMap,
    body: Bytes, // Changed from Vec<u8>
) -> Result<Response, StatusCode> {
    // Push always requires authentication
    let user_id = get_authenticated_user_id(&state, &headers)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check ownership
    if repo.owner_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);

    let mut child = Command::new("git")
        .arg("receive-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&body) // body is already &[u8] via Bytes deref
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let output = child
        .wait_with_output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.status.success() {
        eprintln!(
            "git-receive-pack failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Update repository size after push
    if let Ok(size) = state.git_storage.get_repo_size(&repo_hash) {
        let _ = state
            .db
            .update_repository_size(&repo_hash, size as i64)
            .await;
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-receive-pack-result",
        )
        .body(Body::from(output.stdout))
        .unwrap())
}

fn format_pkt_line(data: &str) -> String {
    let len = data.len() + 4;
    format!("{:04x}{}", len, data)
}

/// Simple dumb HTTP clone - respects privacy
pub async fn dumb_clone(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check authentication for private repos
    if repo.is_private != 0 {
        let user_id = get_authenticated_user_id(&state, &headers)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        if repo.owner_id != user_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);

    let output = Command::new("git")
        .arg("bundle")
        .arg("create")
        .arg("-")
        .arg("--all")
        .current_dir(&repo_path)
        .output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-bundle")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.bundle\"", repo.name),
        )
        .body(Body::from(output.stdout))
        .unwrap())
}
