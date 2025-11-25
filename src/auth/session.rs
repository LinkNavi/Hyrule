
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use rand::rngs::OsRng;
use rand::RngCore;

fn generate_session_id() -> String {
    let mut random_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut random_bytes);
    hex::encode(random_bytes)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: i64,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

// In-memory session store (in production, use Redis or database)
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn create_session(&self, user_id: i64, username: String) -> String {
        let session_id = generate_session_id();
        let session = Session {
            user_id,
            username,
            expires_at: Utc::now() + Duration::hours(24),
        };
        
        self.sessions.write().await.insert(session_id.clone(), session);
        session_id
    }
    
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)?;
        
        // Check if expired
        if session.expires_at < Utc::now() {
            return None;
        }
        
        Some(session.clone())
    }
    
    pub async fn delete_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }
    
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        self.sessions.write().await.retain(|_, session| {
            session.expires_at > now
        });
    }
}



// Extract session from cookie
#[derive(Debug, Clone)]
pub struct SessionUser {
    pub user_id: i64,
    pub username: String,
}

impl SessionUser {
    pub fn is_admin(&self) -> bool {
        self.username == "admin" || self.username == "Link"
    }
}
#[async_trait]
impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get session store from extensions
        let session_store = parts
            .extensions
            .get::<Arc<SessionStore>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Get cookie jar
        let cookies = parts
            .extensions
            .get::<CookieJar>()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        // Get session cookie
        let session_id = cookies
            .get("session_id")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .value();
        
        // Validate session
        let session = session_store
            .get_session(session_id)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        Ok(SessionUser {
            user_id: session.user_id,
            username: session.username,
        })
    }
}

// Optional session - doesn't fail if not authenticated
pub struct OptionalSession(pub Option<SessionUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalSession
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match SessionUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalSession(Some(user))),
            Err(_) => Ok(OptionalSession(None)),
        }
    }
}
