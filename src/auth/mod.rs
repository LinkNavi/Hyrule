// src/auth/mod.rs
pub mod jwt;
pub mod password;
pub mod middleware;
pub mod auth_session;
pub mod session;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
// Admin user extractor
#[derive(Debug)]
pub struct AdminUser {
    pub id: i64,
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        
        // Get database from state
        let state = parts.extensions.get::<Arc<crate::AppState>>()
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Check is_admin flag from database
        let user = state.db.get_user_by_id(auth_user.id).await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        
        if user.is_admin != 0 {
            Ok(AdminUser {
                id: auth_user.id,
                username: auth_user.username,
            })
        } else {
            Err(axum::http::StatusCode::FORBIDDEN)
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,  // user id
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header manually (Axum 0.7 doesn't have TypedHeader)
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];
        let claims = jwt::validate_token(token)
            .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser {
            id: claims.sub,
            username: claims.username,
        })
    }
}

// Optional authentication - doesn't fail if not authenticated
#[derive(Debug)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                if let Ok(claims) = jwt::validate_token(token) {
                    return Ok(OptionalAuthUser(Some(AuthUser {
                        id: claims.sub,
                        username: claims.username,
                    })));
                }
            }
        }

        Ok(OptionalAuthUser(None))
    }
}
