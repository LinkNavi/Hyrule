// Hyrule/src/main.rs - Enhanced with Security Features
mod auth;
mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod storage;
mod templates;
mod utils;

use crate::auth::session::SessionStore;
use crate::config::Config;
use crate::db::Database;
use crate::middleware::cache::CacheService;
use crate::middleware::rate_limit::RateLimiter;
use crate::middleware::csrf::CsrfProtection;
use crate::routes::create_router;
use crate::services::health::HealthMonitor;
use crate::storage::git::GitStorage;
use axum::Extension;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub cache: CacheService,
    pub git_storage: Arc<GitStorage>,
    pub session_store: Arc<SessionStore>,
    pub rate_limiter: Arc<RateLimiter>,
    pub csrf_protection: Arc<CsrfProtection>,
}

async fn create_admin_if_not_exists(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    match db.get_user_by_username("admin").await {
        Ok(_) => {
            println!("✓ Admin user already exists");
            Ok(())
        }
        Err(_) => {
            // Generate secure random password
            use rand::Rng;
            let password: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();
            
            // Validate password meets requirements
            if let Err(e) = crate::utils::validation::validate_password(&password) {
                return Err(format!("Generated password failed validation: {}", e).into());
            }
            
            let password_hash = crate::auth::password::hash_password(&password)
                .map_err(|e| e.to_string())?;
            let public_key = hex::encode(blake3::hash(b"admin").as_bytes());

            let admin_req = crate::models::CreateUserRequest {
                username: "admin".to_string(),
                email: "admin@hyrule.local".to_string(),
                password_hash,
                public_key,
                storage_quota: 10737418240,
            };

            db.create_user(&admin_req).await?;
            
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("✓ Admin user created");
            println!("⚠️  SAVE THIS PASSWORD - IT WILL NOT BE SHOWN AGAIN:");
            println!("   Username: admin");
            println!("   Password: {}", password);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("⚠️  CHANGE THIS PASSWORD IMMEDIATELY AFTER FIRST LOGIN!");
            
            Ok(())
        }
    }
}

/// Validate critical security configuration
fn validate_security_config() -> Result<(), Box<dyn std::error::Error>> {
    // Check JWT_SECRET is set
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| "CRITICAL: JWT_SECRET environment variable must be set")?;
    
    if jwt_secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters long".into());
    }
    
    // Warn if HSTS not enabled in production
    if cfg!(not(debug_assertions)) {
        if std::env::var("ENABLE_HSTS").unwrap_or_default() != "true" {
            eprintln!("⚠️  WARNING: ENABLE_HSTS is not set in production mode");
            eprintln!("   Set ENABLE_HSTS=true to enable HTTP Strict Transport Security");
        }
    }
    
    // Validate allowed origins if set
    if let Ok(origins) = std::env::var("ALLOWED_ORIGINS") {
        for origin in origins.split(',') {
            if origin.trim().is_empty() {
                return Err("ALLOWED_ORIGINS contains empty value".into());
            }
        }
    }
    
    println!("✓ Security configuration validated");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with more details for security events
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_line_number(true)
        .init();

    tracing::info!("🚀 Starting Hyrule server...");

    // Load configuration
    dotenvy::dotenv().ok();
    
    // Validate security configuration BEFORE starting
    validate_security_config()?;
    
    let config = Config::from_env()?;

    tracing::info!("📊 Connecting to database: {}", config.database_url);

    // Initialize database
    let db = Database::new(&config.database_url).await?;

    tracing::info!("🔄 Running database migrations...");
    db.migrate().await?;

    // Initialize cache service
    let cache = CacheService::new();

    // Initialize Git storage
    tracing::info!("📦 Initializing Git storage...");
    let storage_path = PathBuf::from("storage/repos");
    let git_storage = Arc::new(GitStorage::new(storage_path)?);
    tracing::info!("✓ Git storage ready");

    // Initialize session store
    let session_store = Arc::new(SessionStore::new());

    // Initialize rate limiter
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));
    
    // Initialize CSRF protection
    let csrf_protection = Arc::new(CsrfProtection::new());

    // Create application state
    let state = Arc::new(AppState {
        db: db.clone(),
        config: config.clone(),
        cache,
        git_storage,
        session_store: session_store.clone(),
        rate_limiter: rate_limiter.clone(),
        csrf_protection: csrf_protection.clone(),
    });

    // Start background tasks
    tracing::info!("🔧 Starting background tasks...");
    
    // Session cleanup task
    let session_store_clone = session_store.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            session_store_clone.cleanup_expired().await;
            tracing::debug!("Session cleanup completed");
        }
    });
    
    // Rate limiter cleanup task
    let rate_limiter_clone = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            rate_limiter_clone.cleanup().await;
            tracing::debug!("Rate limiter cleanup completed");
        }
    });
    
    // CSRF token cleanup task
    let csrf_clone = csrf_protection.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(600)); // 10 minutes
        loop {
            interval.tick().await;
            csrf_clone.cleanup_expired().await;
            tracing::debug!("CSRF token cleanup completed");
        }
    });

    // Start health monitor
    tracing::info!("💚 Starting health monitoring service...");
    let health_monitor = HealthMonitor::new(
        db.clone(),
        config.min_replica_count,
        10, // Check every 10 minutes
    );

    tokio::spawn(async move {
        health_monitor.start().await;
    });

    // Create router with all security middleware
    let app = create_router(state.clone())
        .layer(Extension(state.session_store.clone()))
        .layer(Extension(state.csrf_protection.clone()))
        .layer(axum::middleware::from_fn(
            crate::middleware::security::security_headers
        ))
        .layer(axum::middleware::from_fn(
            crate::middleware::security::attack_prevention
        ))
        .layer(axum::middleware::from_fn(
            crate::middleware::rate_limit::global_rate_limit
        ))
        .layer(TraceLayer::new_for_http());

    // Create admin user if needed
    create_admin_if_not_exists(&db).await?;

    // Start server
    let addr = config.server_addr();
    tracing::info!("✅ Hyrule server listening on {}", addr);
    tracing::info!("🌐 Web UI: http://{}", addr);
    tracing::info!("🔌 API: http://{}/api", addr);
    tracing::info!("📁 Git storage: storage/repos/");
    
    // Security status
    tracing::info!("🔒 Security features enabled:");
    tracing::info!("   ✓ JWT authentication");
    tracing::info!("   ✓ Rate limiting");
    tracing::info!("   ✓ CSRF protection");
    tracing::info!("   ✓ Security headers");
    tracing::info!("   ✓ Input validation");
    
    if std::env::var("ENABLE_HSTS").unwrap_or_default() == "true" {
        tracing::info!("   ✓ HSTS enabled");
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
