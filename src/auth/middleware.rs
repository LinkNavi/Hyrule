// src/auth/middleware.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn require_auth(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            
            // Validate token
            match crate::auth::jwt::validate_token(token) {
                Ok(_) => return Ok(next.run(request).await),
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

// Optional auth - doesn't fail if no auth provided
pub async fn optional_auth(
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            if let Ok(claims) = crate::auth::jwt::validate_token(token) {
                // You could add user info to request extensions here
                request.extensions_mut().insert(claims);
            }
        }
    }
    
    next.run(request).await
}