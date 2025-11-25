// src/utils/validation.rs - Enhanced Security Version
use regex::Regex;
use std::path::Path;

// Security constants
const MAX_PATH_LENGTH: usize = 1024;
const MAX_REPO_HASH_LENGTH: usize = 64;
const MAX_USERNAME_LENGTH: usize = 32;
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_REPO_NAME_LENGTH: usize = 64;
const MIN_REPO_NAME_LENGTH: usize = 3;

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    static ref REPO_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    static ref REPO_HASH_REGEX: Regex = Regex::new(r"^[a-f0-9]{40}$").unwrap();
    static ref BRANCH_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_/-]+$").unwrap();
}

/// Validates file/directory paths to prevent path traversal attacks
pub fn is_safe_path(path: &str) -> bool {
    // Length check
    if path.is_empty() || path.len() > MAX_PATH_LENGTH {
        return false;
    }
    
    // Prevent path traversal
    if path.contains("..") {
        return false;
    }
    
    // Prevent absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    
    // Prevent Windows drive letters
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return false;
    }
    
    // Prevent null bytes
    if path.contains('\0') {
        return false;
    }
    
    // Prevent control characters
    if path.chars().any(|c| c.is_control()) {
        return false;
    }
    
    // Additional checks for specific dangerous patterns
    let dangerous_patterns = [
        "~", // Home directory
        "$", // Environment variables
        "`", // Command substitution
        "|", // Pipe
        ";", // Command separator
        "&", // Background process
        "!", // History expansion
    ];
    
    for pattern in &dangerous_patterns {
        if path.contains(pattern) {
            return false;
        }
    }
    
    // Verify the path doesn't try to escape when canonicalized
    let test_base = Path::new("/tmp/test");
    if let Ok(joined) = test_base.join(path).canonicalize() {
        if !joined.starts_with(test_base) {
            return false;
        }
    }
    
    true
}

/// Validates repository hash format
pub fn validate_repo_hash(hash: &str) -> bool {
    // Check length
    if hash.len() != 40 {
        return false;
    }
    
    // Check if it's valid hex
    REPO_HASH_REGEX.is_match(hash)
}

/// Validates username format and length
pub fn validate_username(username: &str) -> Result<(), &'static str> {
    if username.len() < MIN_USERNAME_LENGTH {
        return Err("Username too short (minimum 3 characters)");
    }
    
    if username.len() > MAX_USERNAME_LENGTH {
        return Err("Username too long (maximum 32 characters)");
    }
    
    if !USERNAME_REGEX.is_match(username) {
        return Err("Username can only contain letters, numbers, underscores, and hyphens");
    }
    
    // Prevent reserved names
    let reserved = ["admin", "root", "system", "api", "www", "ftp", "mail"];
    if reserved.contains(&username.to_lowercase().as_str()) {
        return Err("Username is reserved");
    }
    
    Ok(())
}

/// Validates repository name
pub fn validate_repo_name(name: &str) -> Result<(), &'static str> {
    if name.len() < MIN_REPO_NAME_LENGTH {
        return Err("Repository name too short (minimum 3 characters)");
    }
    
    if name.len() > MAX_REPO_NAME_LENGTH {
        return Err("Repository name too long (maximum 64 characters)");
    }
    
    if !REPO_NAME_REGEX.is_match(name) {
        return Err("Repository name can only contain letters, numbers, underscores, and hyphens");
    }
    
    // Prevent names that could cause issues
    let reserved = ["git", "config", "objects", "refs", "HEAD"];
    if reserved.contains(&name) {
        return Err("Repository name is reserved");
    }
    
    Ok(())
}

/// Validates Git branch/ref names
pub fn validate_ref_name(ref_name: &str) -> bool {
    // Length check
    if ref_name.is_empty() || ref_name.len() > 255 {
        return false;
    }
    
    // Must match allowed pattern
    if !BRANCH_NAME_REGEX.is_match(ref_name) {
        return false;
    }
    
    // Cannot start or end with /
    if ref_name.starts_with('/') || ref_name.ends_with('/') {
        return false;
    }
    
    // Cannot contain consecutive slashes
    if ref_name.contains("//") {
        return false;
    }
    
    // Cannot contain certain sequences that git disallows
    let disallowed = ["@{", "..", "~", "^", ":", "?", "*", "["];
    for pattern in &disallowed {
        if ref_name.contains(pattern) {
            return false;
        }
    }
    
    true
}

/// Validates password strength
pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    
    if password.len() > 128 {
        return Err("Password too long (maximum 128 characters)");
    }
    
    // Check for at least one number
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number");
    }
    
    // Check for at least one letter
    if !password.chars().any(|c| c.is_alphabetic()) {
        return Err("Password must contain at least one letter");
    }
    
    Ok(())
}

/// Sanitizes user input for HTML display
pub fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Validates object ID format (Git SHA-1)
pub fn validate_object_id(object_id: &str) -> bool {
    object_id.len() == 40 && object_id.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        assert!(!is_safe_path("../etc/passwd"));
        assert!(!is_safe_path("foo/../../../etc/passwd"));
        assert!(!is_safe_path("/etc/passwd"));
        assert!(is_safe_path("valid/path/to/file.txt"));
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("validuser123").is_ok());
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("user<script>").is_err()); // Invalid chars
        assert!(validate_username("admin").is_err()); // Reserved
    }

    #[test]
    fn test_repo_hash_validation() {
        assert!(validate_repo_hash("abcdef1234567890abcdef1234567890abcdef12"));
        assert!(!validate_repo_hash("invalid"));
        assert!(!validate_repo_hash("ABCDEF1234567890ABCDEF1234567890ABCDEF12")); // Uppercase
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("Secure123").is_ok());
        assert!(validate_password("short").is_err());
        assert!(validate_password("nodigitshere").is_err());
        assert!(validate_password("12345678").is_err()); // No letters
    }
}
