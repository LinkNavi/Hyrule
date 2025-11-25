-- migrations/20240101000000_initial.sql

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    public_key TEXT NOT NULL,
    storage_quota INTEGER NOT NULL DEFAULT 1073741824,
    storage_used INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Repositories table
CREATE TABLE IF NOT EXISTS repositories (
    repo_hash TEXT PRIMARY KEY,
    owner_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    size INTEGER NOT NULL DEFAULT 0,
    storage_tier TEXT NOT NULL DEFAULT 'free',
    is_private INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_updated TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

-- Nodes table
CREATE TABLE IF NOT EXISTS nodes (
    node_id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    last_seen TEXT NOT NULL DEFAULT (datetime('now')),
    reputation_score INTEGER NOT NULL DEFAULT 100,
    storage_capacity INTEGER NOT NULL DEFAULT 0,
    storage_used INTEGER NOT NULL DEFAULT 0,
    is_anchor INTEGER NOT NULL DEFAULT 0
);

-- Replicas table
CREATE TABLE IF NOT EXISTS replicas (
    repo_hash TEXT NOT NULL,
    node_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_verified TEXT,
    PRIMARY KEY (repo_hash, node_id),
    FOREIGN KEY (repo_hash) REFERENCES repositories(repo_hash),
    FOREIGN KEY (node_id) REFERENCES nodes(node_id)
);

-- Pins table for user-pinned repositories
CREATE TABLE IF NOT EXISTS pins (
    user_id INTEGER NOT NULL,
    repo_hash TEXT NOT NULL,
    pinned_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, repo_hash),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (repo_hash) REFERENCES repositories(repo_hash)
);

-- Repository tags for categorization
CREATE TABLE IF NOT EXISTS repo_tags (
    repo_hash TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (repo_hash, tag),
    FOREIGN KEY (repo_hash) REFERENCES repositories(repo_hash)
);

-- API keys for programmatic access
CREATE TABLE IF NOT EXISTS api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used TEXT,
    expires_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Activity log for auditing
CREATE TABLE IF NOT EXISTS activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    details TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Repository stars/favorites
CREATE TABLE IF NOT EXISTS repo_stars (
    user_id INTEGER NOT NULL,
    repo_hash TEXT NOT NULL,
    starred_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, repo_hash),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (repo_hash) REFERENCES repositories(repo_hash)
);

-- Repository access logs for analytics
CREATE TABLE IF NOT EXISTS repo_access_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_hash TEXT NOT NULL,
    access_type TEXT NOT NULL,
    user_id INTEGER,
    node_id TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    bytes_transferred INTEGER DEFAULT 0,
    FOREIGN KEY (repo_hash) REFERENCES repositories(repo_hash),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (node_id) REFERENCES nodes(node_id)
);

ALTER TABLE users ADD COLUMN is_admin INTEGER DEFAULT 0 NOT NULL;
UPDATE users SET is_admin = 1 WHERE username = 'admin' OR username = 'Link';

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_repos_owner ON repositories(owner_id);
CREATE INDEX IF NOT EXISTS idx_repos_updated ON repositories(last_updated);
CREATE INDEX IF NOT EXISTS idx_replicas_repo ON replicas(repo_hash);
CREATE INDEX IF NOT EXISTS idx_replicas_node ON replicas(node_id);
CREATE INDEX IF NOT EXISTS idx_nodes_last_seen ON nodes(last_seen);
CREATE INDEX IF NOT EXISTS idx_pins_user ON pins(user_id);
CREATE INDEX IF NOT EXISTS idx_pins_repo ON pins(repo_hash);
CREATE INDEX IF NOT EXISTS idx_repo_tags_tag ON repo_tags(tag);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_activity_log_user ON activity_log(user_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_created ON activity_log(created_at);
CREATE INDEX IF NOT EXISTS idx_repo_stars_repo ON repo_stars(repo_hash);
CREATE INDEX IF NOT EXISTS idx_repo_access_repo ON repo_access_log(repo_hash);
CREATE INDEX IF NOT EXISTS idx_repo_access_timestamp ON repo_access_log(timestamp);
