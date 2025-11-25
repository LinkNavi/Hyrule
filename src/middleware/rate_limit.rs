// src/middleware/rate_limit.rs - Enhanced Security Version
use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::net::IpAddr;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    max_requests: usize,
    window: Duration,
}

struct RateLimitEntry {
    count: usize,
    window_start: Instant,
    last_request: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }
    
    pub async fn check(&self, key: &str) -> Result<(), ()> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        let entry = requests.entry(key.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                window_start: now,
                last_request: now,
            });
        
        // Reset window if expired
        if now.duration_since(entry.window_start) > self.window {
            entry.count = 0;
            entry.window_start = now;
        }
        
        // Check rate limit
        if entry.count >= self.max_requests {
            tracing::warn!("Rate limit exceeded for key: {}", key);
            return Err(());
        }
        
        entry.count += 1;
        entry.last_request = now;
        Ok(())
    }
    
    /// Cleanup old entries periodically (call this in a background task)
    pub async fn cleanup(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        requests.retain(|_, entry| {
            now.duration_since(entry.window_start) <= self.window
        });
    }
}

/// Extract client identifier from request
fn extract_client_id(headers: &HeaderMap) -> String {
    // Try to get real IP from headers (for reverse proxy setups)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP (client's original IP)
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Fallback to a generic identifier
    "unknown".to_string()
}

/// Global rate limiter middleware
pub async fn global_rate_limit(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_id = extract_client_id(request.headers());
    
    // Different rate limits for different endpoints
    let path = request.uri().path();
    let (max_requests, window_secs) = if path.starts_with("/api/auth/") {
        (10, 300) // 10 requests per 5 minutes for auth endpoints
    } else if path.starts_with("/api/") {
        (100, 60) // 100 requests per minute for API
    } else {
        (300, 60) // 300 requests per minute for web pages
    };
    
    let limiter = RateLimiter::new(max_requests, window_secs);
    
    match limiter.check(&client_id).await {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => {
            tracing::warn!("Rate limit exceeded for client: {} on path: {}", client_id, path);
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

/// Auth-specific rate limiter with stricter limits
pub async fn auth_rate_limit(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_id = extract_client_id(request.headers());
    
    // Very strict limit for auth attempts: 5 per 15 minutes
    let limiter = RateLimiter::new(5, 900);
    
    match limiter.check(&client_id).await {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => {
            tracing::warn!("Auth rate limit exceeded for client: {}", client_id);
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

/// Per-user rate limiter (for authenticated requests)
pub struct PerUserRateLimiter {
    limiters: Arc<RwLock<HashMap<i64, RateLimiter>>>,
}

impl PerUserRateLimiter {
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn check(&self, user_id: i64, max_requests: usize, window_secs: u64) -> Result<(), ()> {
        let mut limiters = self.limiters.write().await;
        let limiter = limiters.entry(user_id)
            .or_insert_with(|| RateLimiter::new(max_requests, window_secs));
        
        limiter.check(&user_id.to_string()).await
    }
    
    pub async fn cleanup(&self) {
        let limiters = self.limiters.read().await;
        for limiter in limiters.values() {
            limiter.cleanup().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(5, 60);
        
        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(limiter.check("test_client").await.is_ok());
        }
        
        // 6th request should fail
        assert!(limiter.check("test_client").await.is_err());
    }

    #[tokio::test]
    async fn test_window_reset() {
        let limiter = RateLimiter::new(2, 1); // 2 requests per 1 second
        
        assert!(limiter.check("test").await.is_ok());
        assert!(limiter.check("test").await.is_ok());
        assert!(limiter.check("test").await.is_err());
        
        // Wait for window to reset
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        assert!(limiter.check("test").await.is_ok());
    }
}
