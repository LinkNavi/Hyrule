// src/middleware/security.rs - Enhanced Security Version
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{header, HeaderValue},
};

pub async fn security_headers(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Prevent clickjacking
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY")
    );
    
    // Prevent MIME sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff")
    );
    
    // Enable XSS protection (legacy, but doesn't hurt)
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block")
    );
    
    // Strict referrer policy
    headers.insert(
        header::REFERER,
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );
    
    // Content Security Policy - strict but functional
    let csp = [
        "default-src 'self'",
        "script-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com", // unsafe-inline needed for inline scripts
        "style-src 'self' 'unsafe-inline'", // unsafe-inline needed for inline styles
        "img-src 'self' data: https:",
        "font-src 'self' https://cdnjs.cloudflare.com",
        "connect-src 'self'",
        "frame-ancestors 'none'",
        "base-uri 'self'",
        "form-action 'self'",
    ].join("; ");
    
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_str(&csp).unwrap()
    );
    
    // Permissions Policy (formerly Feature-Policy)
    let permissions = [
        "geolocation=()",
        "microphone=()",
        "camera=()",
        "payment=()",
        "usb=()",
        "magnetometer=()",
        "gyroscope=()",
        "accelerometer=()",
    ].join(", ");
    
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_str(&permissions).unwrap()
    );
    
    // Strict Transport Security (HSTS) - 1 year
    // Only enable in production with HTTPS
    if std::env::var("ENABLE_HSTS").unwrap_or_default() == "true" {
        headers.insert(
            header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload")
        );
    }
    
    // Remove server header to avoid leaking version info
    headers.remove(header::SERVER);
    
    // Add custom security header
    headers.insert(
        header::HeaderName::from_static("x-security-policy"),
        HeaderValue::from_static("Hyrule-Secure-1.0")
    );
    
    response
}

/// CORS middleware with security considerations

pub async fn secure_cors(
    request: Request,
    next: Next,
) -> Response {
    // COPY the origin into an owned String so the borrow ends
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string()); // <-- this is the key fix
    
    // now it's safe to move `request`
    let mut response = next.run(request).await;

    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    if let Some(origin) = origin {
        if allowed_origins
            .split(',')
            .any(|allowed| allowed.trim() == origin)
        {
            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_str(&origin).unwrap(),
            );

            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
        }
    }

    response
}


/// Request size limiter
pub async fn request_size_limit(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Check content length
    if let Some(content_length) = request.headers().get(header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_REQUEST_SIZE: usize = 100 * 1024 * 1024; // 100MB
                if length > MAX_REQUEST_SIZE {
                    tracing::warn!("Request too large: {} bytes", length);
                    return Err(axum::http::StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Prevent common attacks via headers
pub async fn attack_prevention(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let headers = request.headers();
    
    // Check for suspicious User-Agent patterns
    if let Some(user_agent) = headers.get(header::USER_AGENT) {
        if let Ok(ua_str) = user_agent.to_str() {
            let suspicious_patterns = [
                "sqlmap",
                "nikto",
                "masscan",
                "nmap",
                "havij",
                "acunetix",
            ];
            
            let ua_lower = ua_str.to_lowercase();
            for pattern in &suspicious_patterns {
                if ua_lower.contains(pattern) {
                    tracing::warn!("Suspicious user agent detected: {}", ua_str);
                    return Err(axum::http::StatusCode::FORBIDDEN);
                }
            }
        }
    }
    
    // Check for header injection attempts
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            // Look for CRLF injection attempts
            if value_str.contains('\r') || value_str.contains('\n') {
                tracing::warn!("Header injection attempt detected in {}: {}", name, value_str);
                return Err(axum::http::StatusCode::BAD_REQUEST);
            }
        }
    }
    
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request as HttpRequest};

    #[tokio::test]
    async fn test_security_headers_added() {
        let request = HttpRequest::builder()
            .body(Body::empty())
            .unwrap();
        
        // Would need actual middleware testing setup here
        // This is a placeholder showing the test structure
    }
}
