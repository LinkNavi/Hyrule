// src/utils/hash.rs
use blake3::Hasher;

pub fn generate_repo_hash(name: &str, owner_id: i64) -> String {
    let mut hasher = Hasher::new();
    hasher.update(name.as_bytes());
    hasher.update(&owner_id.to_le_bytes());
    hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
    hex::encode(&hasher.finalize().as_bytes()[..20])
}