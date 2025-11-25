// src/middleware/cache.rs
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct CacheService {
    repo_cache: Arc<Cache<String, Vec<u8>>>,
    stats_cache: Arc<Cache<String, Vec<u8>>>,
}

impl CacheService {
    pub fn new() -> Self {
        Self {
            repo_cache: Arc::new(
                Cache::builder()
                    .max_capacity(1000)
                    .time_to_live(Duration::from_secs(300)) // 5 minutes
                    .build()
            ),
            stats_cache: Arc::new(
                Cache::builder()
                    .max_capacity(100)
                    .time_to_live(Duration::from_secs(60)) // 1 minute
                    .build()
            ),
        }
    }
    
    pub async fn get_repo(&self, key: &str) -> Option<Vec<u8>> {
        self.repo_cache.get(key).await
    }
    
    pub async fn set_repo(&self, key: String, value: Vec<u8>) {
        self.repo_cache.insert(key, value).await;
    }
    
    pub async fn invalidate_repo(&self, key: &str) {
        self.repo_cache.invalidate(key).await;
    }
    
    pub async fn get_stats(&self, key: &str) -> Option<Vec<u8>> {
        self.stats_cache.get(key).await
    }
    
    pub async fn set_stats(&self, key: String, value: Vec<u8>) {
        self.stats_cache.insert(key, value).await;
    }
}



