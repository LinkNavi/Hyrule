// Hyrule/src/handlers/git_http.rs
// Full implementation of Git HTTP protocol (Smart HTTP)
use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, header, HeaderMap},
    response::{IntoResponse, Response},
    body::Body,
};
use std::sync::Arc;
use std::process::{Command, Stdio};
use std::io::Write;
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GitService {
    service: String,
}

/// Handle git-upload-pack (for clone/fetch)
pub async fn git_upload_pack(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    headers: HeaderMap,
    body: Vec<u8>,
) -> Result<Response, StatusCode> {
    // Verify repository exists
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Check if private and needs auth
    if repo.is_private != 0 {
        if let Some(auth) = headers.get("authorization") {
            // TODO: Verify authorization
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    
    let repo_path = state.git_storage.repo_path(&repo_hash);
    
    // Execute git-upload-pack
    let mut cmd = Command::new("git");
    cmd.arg("upload-pack")
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    let output = cmd.output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let mut response_body = Vec::new();
    response_body.extend_from_slice(format_packet("# service=git-upload-pack\n").as_bytes());
    response_body.extend_from_slice(b"0000");
    response_body.extend_from_slice(&output.stdout);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-advertisement")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(response_body))
        .unwrap())
}

/// Handle git-receive-pack (for push)
pub async fn git_receive_pack(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    headers: HeaderMap,
    body: Vec<u8>,
) -> Result<Response, StatusCode> {
    // Must be authenticated for push
    // TODO: Check authentication from headers
    
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let repo_path = state.git_storage.repo_path(&repo_hash);
    
    // Execute git-receive-pack
    let mut cmd = Command::new("git");
    cmd.arg("receive-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Write request body to git-receive-pack stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    let output = child.wait_with_output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-receive-pack-result")
        .body(Body::from(output.stdout))
        .unwrap())
}

/// Info/refs endpoint for git smart HTTP
pub async fn git_info_refs(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
    Query(params): Query<GitService>,
) -> Result<Response, StatusCode> {
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Check if private
    if repo.is_private != 0 {
        // TODO: Check auth
    }
    
    let repo_path = state.git_storage.repo_path(&repo_hash);
    
    let service = &params.service;
    let git_command = if service == "git-upload-pack" {
        "upload-pack"
    } else if service == "git-receive-pack" {
        "receive-pack"
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };
    
    // Execute git command
    let mut cmd = Command::new("git");
    cmd.arg(git_command)
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    let output = cmd.output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Format response according to git protocol
    let mut response_body = Vec::new();
    response_body.extend_from_slice(format_packet(&format!("# service={}\n", service)).as_bytes());
    response_body.extend_from_slice(b"0000");
    response_body.extend_from_slice(&output.stdout);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, format!("application/x-{}-advertisement", service))
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(response_body))
        .unwrap())
}

/// Format a git protocol packet
fn format_packet(data: &str) -> String {
    let len = data.len() + 4;
    format!("{:04x}{}", len, data)
}

/// Clone a repository (simple web-based download)
pub async fn web_clone(
    State(state): State<Arc<AppState>>,
    Path(repo_hash): Path<String>,
) -> Result<Response, StatusCode> {
    let repo = state.db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Create a bundle of the repository
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
        .header(header::CONTENT_DISPOSITION, 
            format!("attachment; filename=\"{}.bundle\"", repo.name))
        .body(Body::from(output.stdout))
        .unwrap())
}
