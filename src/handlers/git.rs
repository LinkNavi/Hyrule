// src/handlers/git.rs - Enhanced Security Version
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use crate::utils::validation;
use std::sync::Arc;

use crate::auth::{AuthUser, OptionalAuthUser};
use crate::models::*;
use crate::AppState;
use serde::{Deserialize, Serialize};

// Security constants
const MAX_OBJECT_SIZE: usize = 100 * 1024 * 1024; // 100MB
const MAX_BATCH_SIZE: usize = 1000; // Maximum objects in batch
const MAX_OBJECT_ID_LENGTH: usize = 40;

#[derive(Debug, Deserialize)]
pub struct UploadObjectRequest {
    pub object_id: String,
    pub object_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct UploadObjectResponse {
    pub success: bool,
    pub object_id: String,
}

/// Upload a single Git object - REQUIRES AUTHENTICATION
pub async fn upload_object(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
    Json(payload): Json<UploadObjectRequest>,
) -> Result<Json<UploadObjectResponse>, StatusCode> {
    // Validate repo hash
    if !validation::validate_repo_hash(&repo_hash) {
        tracing::warn!("Invalid repo hash attempted: {}", repo_hash);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate object ID
    if !validation::validate_object_id(&payload.object_id) {
        tracing::warn!("Invalid object ID attempted: {}", payload.object_id);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate object type
    let valid_types = ["blob", "tree", "commit", "tag"];
    if !valid_types.contains(&payload.object_type.as_str()) {
        tracing::warn!("Invalid object type attempted: {}", payload.object_type);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let repo = state.db.get_repository(&repo_hash).await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if repo.owner_id != user.id {
        tracing::warn!("Unauthorized upload attempt by user {} to repo {}", user.id, repo_hash);
        return Err(StatusCode::FORBIDDEN);
    }

    use base64::{engine::general_purpose, Engine as _};
    let data = general_purpose::STANDARD
        .decode(&payload.data)
        .map_err(|e| {
            tracing::warn!("Base64 decode error: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Check size limit
    if data.len() > MAX_OBJECT_SIZE {
        tracing::warn!("Object too large: {} bytes", data.len());
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    state.git_storage.store_object(&repo_hash, &payload.object_id, &data)
        .map_err(|e| {
            tracing::error!("Failed to store object: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Object {} uploaded to repo {}", payload.object_id, repo_hash);

    Ok(Json(UploadObjectResponse {
        success: true,
        object_id: payload.object_id,
    }))
}

/// Upload multiple objects (batch) - REQUIRES AUTHENTICATION
#[derive(Debug, Deserialize)]
pub struct BatchUploadRequest {
    pub objects: Vec<UploadObjectRequest>,
}

#[derive(Debug, Serialize)]
pub struct BatchUploadResponse {
    pub uploaded: usize,
    pub failed: Vec<String>,
}

pub async fn batch_upload_objects(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
    Json(payload): Json<BatchUploadRequest>,
) -> Result<Json<BatchUploadResponse>, StatusCode> {
    // Validate repo hash
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Check batch size limit
    if payload.objects.len() > MAX_BATCH_SIZE {
        tracing::warn!("Batch too large: {} objects", payload.objects.len());
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check ownership
    if repo.owner_id != user.id {
        tracing::warn!("Unauthorized batch upload attempt by user {} to repo {}", user.id, repo_hash);
        return Err(StatusCode::FORBIDDEN);
    }

    let mut uploaded = 0;
    let mut failed = Vec::new();

    use base64::{engine::general_purpose, Engine as _};

    for obj in payload.objects {
        // Validate each object
        if !validation::validate_object_id(&obj.object_id) {
            failed.push(obj.object_id);
            continue;
        }

        match general_purpose::STANDARD.decode(&obj.data) {
            Ok(raw_data) => {
                // Check size
                if raw_data.len() > MAX_OBJECT_SIZE {
                    failed.push(obj.object_id.clone());
                    continue;
                }

                let git_object = format!("{} {}\0", obj.object_type, raw_data.len())
                    .into_bytes()
                    .into_iter()
                    .chain(raw_data.into_iter())
                    .collect::<Vec<u8>>();

                if let Err(e) = state
                    .git_storage
                    .store_object(&repo_hash, &obj.object_id, &git_object)
                {
                    tracing::error!("Failed to store object {}: {}", obj.object_id, e);
                    failed.push(obj.object_id);
                } else {
                    uploaded += 1;
                }
            }
            Err(e) => {
                tracing::warn!("Base64 decode error for object {}: {}", obj.object_id, e);
                failed.push(obj.object_id);
            }
        }
    }

    // Update repository size
    if let Ok(size) = state.git_storage.get_repo_size(&repo_hash) {
        let _ = state
            .db
            .update_repository_size(&repo_hash, size as i64)
            .await;
    }

    tracing::info!("Batch upload to repo {}: {} uploaded, {} failed", repo_hash, uploaded, failed.len());

    Ok(Json(BatchUploadResponse { uploaded, failed }))
}

/// Download a Git object - respects privacy
pub async fn download_object(
    State(state): State<Arc<AppState>>,
    OptionalAuthUser(maybe_user): OptionalAuthUser,
    Path((repo_hash, object_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate inputs
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if !validation::validate_object_id(&object_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user = maybe_user.ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user.id {
            tracing::warn!("Unauthorized download attempt by user {} for repo {}", user.id, repo_hash);
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let data = state
        .git_storage
        .read_object(&repo_hash, &object_id)
        .map_err(|e| {
            tracing::warn!("Object not found: {}", e);
            StatusCode::NOT_FOUND
        })?;

    Ok(([(header::CONTENT_TYPE, "application/octet-stream")], data))
}

/// Update a ref (branch/tag) - REQUIRES AUTHENTICATION
#[derive(Debug, Deserialize)]
pub struct UpdateRefRequest {
    pub ref_name: String,
    pub commit_id: String,
}

pub async fn update_ref(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(repo_hash): Path<String>,
    Json(payload): Json<UpdateRefRequest>,
) -> Result<StatusCode, StatusCode> {
    // Validate inputs
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if !validation::validate_ref_name(&payload.ref_name) {
        tracing::warn!("Invalid ref name attempted: {}", payload.ref_name);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if !validation::validate_object_id(&payload.commit_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check ownership
    if repo.owner_id != user.id {
        tracing::warn!("Unauthorized ref update attempt by user {} to repo {}", user.id, repo_hash);
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .git_storage
        .update_ref(&repo_hash, &payload.ref_name, &payload.commit_id)
        .map_err(|e| {
            tracing::error!("Failed to update ref: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Ref {} updated in repo {}", payload.ref_name, repo_hash);

    Ok(StatusCode::OK)
}

/// Get a ref - respects privacy
pub async fn get_ref(
    State(state): State<Arc<AppState>>,
    OptionalAuthUser(maybe_user): OptionalAuthUser,
    Path((repo_hash, ref_name)): Path<(String, String)>,
) -> Result<String, StatusCode> {
    // Validate inputs
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user = maybe_user.ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user.id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let ref_name = urlencoding::decode(&ref_name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate decoded ref name
    if !validation::validate_ref_name(&ref_name) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let commit_id = state
        .git_storage
        .read_ref(&repo_hash, &ref_name)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(commit_id)
}

/// List all objects - respects privacy
#[derive(Debug, Serialize)]
pub struct ListObjectsResponse {
    pub objects: Vec<String>,
    pub count: usize,
}

pub async fn list_objects(
    State(state): State<Arc<AppState>>,
    OptionalAuthUser(maybe_user): OptionalAuthUser,
    Path(repo_hash): Path<String>,
) -> Result<Json<ListObjectsResponse>, StatusCode> {
    // Validate repo hash
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user = maybe_user.ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user.id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let objects = state
        .git_storage
        .list_objects(&repo_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let count = objects.len();

    Ok(Json(ListObjectsResponse { objects, count }))
}

/// Create and download a packfile - respects privacy
pub async fn get_packfile(
    State(state): State<Arc<AppState>>,
    OptionalAuthUser(maybe_user): OptionalAuthUser,
    Path(repo_hash): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate repo hash
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user = maybe_user.ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user.id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let pack_data = state
        .git_storage
        .create_pack(&repo_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/x-git-packed-objects"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"pack.pack\"",
            ),
        ],
        pack_data,
    ))
}

/// Git info/refs endpoint - respects privacy
pub async fn git_info_refs(
    State(state): State<Arc<AppState>>,
    OptionalAuthUser(maybe_user): OptionalAuthUser,
    Path(repo_hash): Path<String>,
) -> Result<String, StatusCode> {
    // Validate repo hash
    if !validation::validate_repo_hash(&repo_hash) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state
        .db
        .get_repository(&repo_hash)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Check access for private repos
    if repo.is_private != 0 {
        let user = maybe_user.ok_or(StatusCode::UNAUTHORIZED)?;
        if repo.owner_id != user.id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let repo_path = state.git_storage.repo_path(&repo_hash);
    if !repo_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let head = state
        .git_storage
        .read_ref(&repo_hash, "HEAD")
        .unwrap_or_else(|_| "0000000000000000000000000000000000000000".to_string());

    let mut response = String::new();
    response.push_str(&format!("{}\tHEAD\n", head));

    if let Ok(main_ref) = state.git_storage.read_ref(&repo_hash, "refs/heads/main") {
        response.push_str(&format!("{}\trefs/heads/main\n", main_ref));
    }

    Ok(response)
}
