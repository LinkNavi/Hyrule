// Hyrule/src/storage/git.rs
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;

pub struct GitStorage {
    base_path: PathBuf,
}

impl GitStorage {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }
    
    pub fn repo_path(&self, repo_hash: &str) -> PathBuf {
        self.base_path.join(repo_hash)
    }
    
    pub fn objects_path(&self, repo_hash: &str) -> PathBuf {
        self.repo_path(repo_hash).join("objects")
    }
    
    pub fn refs_path(&self, repo_hash: &str) -> PathBuf {
        self.repo_path(repo_hash).join("refs")
    }
    
    /// Initialize a repository storage structure
  /// Initialize a repository storage structure
pub fn init_repo(&self, repo_hash: &str) -> Result<()> {
    let repo_path = self.repo_path(repo_hash);
    
    // Create a bare Git repository using git init --bare
    let output = std::process::Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(&repo_path)
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to initialize bare git repository: {}", 
            String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}
    
    /// Store a Git object
    pub fn store_object(&self, repo_hash: &str, object_id: &str, data: &[u8]) -> Result<()> {
        let objects_dir = self.objects_path(repo_hash);
        
        // Git stores objects in subdirectories: objects/ab/cdef123...
        let subdir = &object_id[..2];
        let filename = &object_id[2..];
        
        let subdir_path = objects_dir.join(subdir);
        fs::create_dir_all(&subdir_path)?;
        
        let object_path = subdir_path.join(filename);
        
        // Compress with zlib
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;
        
        fs::write(object_path, compressed)?;
        Ok(())
    }
    
    /// Read a Git object
    pub fn read_object(&self, repo_hash: &str, object_id: &str) -> Result<Vec<u8>> {
        let subdir = &object_id[..2];
        let filename = &object_id[2..];
        
        let object_path = self.objects_path(repo_hash)
            .join(subdir)
            .join(filename);
        
        if !object_path.exists() {
            anyhow::bail!("Object not found: {}", object_id);
        }
        
        // Decompress
        use flate2::read::ZlibDecoder;
        use std::io::Read;
        
        let compressed = fs::read(object_path)?;
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;
        
        Ok(data)
    }
    
    /// Update a ref (branch, tag, etc.)
    pub fn update_ref(&self, repo_hash: &str, ref_name: &str, commit_id: &str) -> Result<()> {
        let ref_path = self.repo_path(repo_hash).join(ref_name);
        
        if let Some(parent) = ref_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(ref_path, format!("{}\n", commit_id))?;
        Ok(())
    }
    
    /// Read a ref
    pub fn read_ref(&self, repo_hash: &str, ref_name: &str) -> Result<String> {
        let ref_path = self.repo_path(repo_hash).join(ref_name);
        
        if !ref_path.exists() {
            anyhow::bail!("Ref not found: {}", ref_name);
        }
        
        let content = fs::read_to_string(ref_path)?;
        Ok(content.trim().to_string())
    }
    
    /// List all objects in a repository
    pub fn list_objects(&self, repo_hash: &str) -> Result<Vec<String>> {
        let objects_dir = self.objects_path(repo_hash);
        let mut objects = Vec::new();
        
        if !objects_dir.exists() {
            return Ok(objects);
        }
        
        for entry in fs::read_dir(objects_dir)? {
            let entry = entry?;
            let subdir_name = entry.file_name();
            let subdir_path = entry.path();
            
            if subdir_path.is_dir() {
                for obj_entry in fs::read_dir(subdir_path)? {
                    let obj_entry = obj_entry?;
                    let obj_name = obj_entry.file_name();
                    let object_id = format!(
                        "{}{}",
                        subdir_name.to_string_lossy(),
                        obj_name.to_string_lossy()
                    );
                    objects.push(object_id);
                }
            }
        }
        
        Ok(objects)
    }
    
    /// Get repository size
    pub fn get_repo_size(&self, repo_hash: &str) -> Result<u64> {
        let repo_path = self.repo_path(repo_hash);
        
        if !repo_path.exists() {
            return Ok(0);
        }
        
        let mut total_size = 0u64;
        for entry in walkdir::WalkDir::new(&repo_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total_size += entry.metadata()?.len();
            }
        }
        
        Ok(total_size)
    }
    
    /// Create a packfile from loose objects
    pub fn create_pack(&self, repo_hash: &str) -> Result<Vec<u8>> {
        // Simplified pack creation - just concatenate objects
        let objects = self.list_objects(repo_hash)?;
        let mut pack_data = Vec::new();
        
        for object_id in objects {
            let data = self.read_object(repo_hash, &object_id)?;
            pack_data.extend_from_slice(&data);
        }
        
        Ok(pack_data)
    }
    
    /// Delete a repository
    pub fn delete_repo(&self, repo_hash: &str) -> Result<()> {
        let repo_path = self.repo_path(repo_hash);
        if repo_path.exists() {
            fs::remove_dir_all(repo_path)?;
        }
        Ok(())
    }
}
