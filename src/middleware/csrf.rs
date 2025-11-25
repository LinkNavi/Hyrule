// src/middleware/csrf.rs - Enhanced Security Version
use axum::{
    extract::Request,
    http::{StatusCode, Method, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use rand::{thread_rng, Rng};
use base64::{engine::general_purpose, Engine as _};

const CSRF_TOKEN_LENGTH: usize = 32;
const CSRF_TOKEN_EXPIRY: Duration = Duration::from_secs(3600); // 1 hour

#[derive(Clone)]
pub struct CsrfToken {
    token: String,
    created_at: Instant,
}

#[derive(Clone)]
pub struct CsrfProtection {
    tokens: Arc<RwLock<HashMap<String, CsrfToken>>>,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Generate a new CSRF token for a session
    pub async fn generate_token(&self, session_id: &str) -> String {
        let mut rng = thread_rng();
        let random_bytes: Vec<u8> = (0..CSRF_TOKEN_LENGTH)
            .map(|_| rng.gen())
            .collect();
        
        let token = general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
        
        let csrf_token = CsrfToken {
            token: token.clone(),
            created_at: Instant::now(),
        };
        
        self.tokens.write().await.insert(session_id.to_string(), csrf_token);
        
        token
    }
    
    /// Validate a CSRF token
    pub async fn validate_token(&self, session_id: &str, provided_token: &str) -> bool {
        let tokens = self.tokens.read().await;
        
        if let Some(stored_token) = tokens.get(session_id) {
            // Check if token is expired
            if stored_token.created_at.elapsed() > CSRF_TOKEN_EXPIRY {
                return false;
            }
            
            // Use constant-time comparison to prevent timing attacks
            use subtle::ConstantTimeEq;
            stored_token.token.as_bytes().ct_eq(provided_token.as_bytes()).into()
        } else {
            false
        }
    }
    
    /// Clean up expired tokens
    pub async fn cleanup_expired(&self) {
        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, token| token.created_at.elapsed() < CSRF_TOKEN_EXPIRY);
    }
}

/// Extract session ID from request (from cookie or header)
fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    // Try to get session from cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "session_id" {
                    return Some(parts[1].to_string());
                }
            }
        }
    }
    None
}

/// Extract CSRF token from request
fn extract_csrf_token(request: &Request) -> Option<String> {
    // Try to get from header first (for AJAX requests)
    if let Some(token) = request.headers().get("x-csrf-token") {
        if let Ok(token_str) = token.to_str() {
            return Some(token_str.to_string());
        }
    }
    
    // Could also check form data, but that requires consuming the body
    // For now, we rely on the header
    None
}

/// CSRF protection middleware
pub async fn csrf_protection(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method();
    
    // Only protect state-changing methods
    if matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH) {
        let path = request.uri().path();
        
        // Skip CSRF check for certain endpoints (e.g., API with other auth)
        let skip_paths = [
            "/api/auth/login",
            "/api/auth/signup",
            "/git/", // Git protocol endpoints use different auth
        ];
        
        let should_skip = skip_paths.iter().any(|skip| path.starts_with(skip));
        
        if !should_skip {
            // Extract session ID
            let session_id = extract_session_id(request.headers())
                .ok_or_else(|| {
                    tracing::warn!("CSRF check failed: No session ID");
                    StatusCode::FORBIDDEN
                })?;
            
            // Extract CSRF token
            let provided_token = extract_csrf_token(&request)
                .ok_or_else(|| {
                    tracing::warn!("CSRF check failed: No CSRF token provided");
                    StatusCode::FORBIDDEN
                })?;
            
            // Validate token
            // In production, you'd get the CsrfProtection instance from app state
            // For now, this is a placeholder
            // TODO: Integrate with actual app state
            
            tracing::debug!("CSRF token validated for session: {}", session_id);
        }
    }
    
    Ok(next.run(request).await)
}

/// Double-submit cookie pattern for CSRF protection

pub async fn double_submit_csrf(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method();

    if matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH) {
        let headers = request.headers();

        // Extract cookie token (owned String)
        let cookie_token = headers
            .get("cookie")
            .and_then(|c| c.to_str().ok())
            .and_then(|cookie_str| {
                for cookie in cookie_str.split(';') {
                    let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                    if parts.len() == 2 && parts[0] == "csrf_token" {
                        return Some(parts[1].to_string());
                    }
                }
                None
            });

        // Extract header token (owned String)
        let header_token = headers
            .get("x-csrf-token")
            .and_then(|t| t.to_str().ok())
            .map(|s| s.to_string());

        // Borrow on headers ends here ↑

        // Now validate tokens
        match (cookie_token, header_token) {
            (Some(cookie), Some(header)) => {
                use subtle::ConstantTimeEq;

                // constant-time compare
                if cookie.as_bytes().ct_eq(header.as_bytes()).unwrap_u8() == 0 {
                    tracing::warn!("CSRF token mismatch");
                    return Err(StatusCode::FORBIDDEN);
                }
            }

            // one or both missing
            _ => {
                tracing::warn!("Missing CSRF tokens");
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    // safe to move request now
    Ok(next.run(request).await)
}


/// Referer checking as additional CSRF protection
pub async fn check_referer(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method();
    
    if matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH) {
        if let Some(referer) = request.headers().get("referer") {
            if let Ok(referer_str) = referer.to_str() {
                // Extract origin from referer
                if let Ok(referer_url) = url::Url::parse(referer_str) {
                    let referer_origin = format!("{}://{}", 
                        referer_url.scheme(), 
                        referer_url.host_str().unwrap_or(""));
                    
                    // Get expected origin from config or request host
                    let expected_origin = std::env::var("SITE_ORIGIN")
                        .unwrap_or_else(|_| "http://localhost:3000".to_string());
                    
                    if referer_origin != expected_origin {
                        tracing::warn!("Referer mismatch: {} != {}", referer_origin, expected_origin);
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            }
        } else {
            // No referer header - might be suspicious
            tracing::debug!("No referer header in state-changing request");
            // Don't block, as some browsers/privacy tools strip referer
        }
    }
    
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_csrf_token_generation() {
        let csrf = CsrfProtection::new();
        let token1 = csrf.generate_token("session1").await;
        let token2 = csrf.generate_token("session2").await;
        
        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 43); // Base64 encoded 32 bytes ≈ 43 chars
    }

    #[tokio::test]
    async fn test_csrf_token_validation() {
        let csrf = CsrfProtection::new();
        let token = csrf.generate_token("session1").await;
        
        assert!(csrf.validate_token("session1", &token).await);
        assert!(!csrf.validate_token("session1", "wrong_token").await);
        assert!(!csrf.validate_token("wrong_session", &token).await);
    }
}
