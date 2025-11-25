// src/auth/jwt.rs - Enhanced Security Version
use crate::auth::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use std::env;

const TOKEN_EXPIRY_HOURS: i64 = 24;
const MAX_TOKEN_AGE_HOURS: i64 = 48; // Maximum acceptable token age

pub fn generate_token(user_id: i64, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    // CRITICAL: JWT_SECRET must be set - no fallback
    let secret = env::var("JWT_SECRET")
        .expect("CRITICAL: JWT_SECRET environment variable must be set. Generate with: openssl rand -base64 32");

    // Validate secret strength
    if secret.len() < 32 {
        panic!("JWT_SECRET must be at least 32 characters long for security");
    }

    let now = Utc::now();
    let exp = (now + Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // Validate token format before processing
    if token.is_empty() || token.len() > 2048 {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    let secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set");

    // Use strict validation
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0; // No leeway for exp/nbf/iat
    validation.validate_exp = true;
    validation.validate_nbf = false;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    // Additional security checks
    let claims = token_data.claims;
    
    // Check token age
    let now = Utc::now().timestamp() as usize;
    let token_age_hours = (now - claims.iat) / 3600;
    
    if token_age_hours > MAX_TOKEN_AGE_HOURS as usize {
        return Err(jsonwebtoken::errors::ErrorKind::ExpiredSignature.into());
    }

    // Validate user_id is positive
    if claims.sub <= 0 {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    // Validate username format (alphanumeric, underscore, hyphen only)
    if !claims.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_validation() {
        std::env::set_var("JWT_SECRET", "test_secret_key_minimum_32_characters_long_12345");
        
        let token = generate_token(1, "testuser").unwrap();
        let claims = validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_invalid_username() {
        std::env::set_var("JWT_SECRET", "test_secret_key_minimum_32_characters_long_12345");
        
        // Token with invalid characters should fail
        let token = generate_token(1, "test<script>").unwrap();
        assert!(validate_token(&token).is_err());
    }
}
