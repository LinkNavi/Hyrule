// src/config.rs
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub default_storage_quota: i64,
    pub min_replica_count: i32,
    pub node_heartbeat_timeout_minutes: i32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://hyrule.db".to_string()),
            host: std::env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            default_storage_quota: std::env::var("DEFAULT_STORAGE_QUOTA")
                .unwrap_or_else(|_| "1073741824".to_string()) // 1GB
                .parse()?,
            min_replica_count: std::env::var("MIN_REPLICA_COUNT")
                .unwrap_or_else(|_| "3".to_string())
                .parse()?,
            node_heartbeat_timeout_minutes: std::env::var("NODE_HEARTBEAT_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
        })
    }
    
    pub fn server_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid server address")
    }
}