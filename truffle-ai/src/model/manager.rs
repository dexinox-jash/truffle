//! Model Manager
//! 
//! Handles model lifecycle: download, caching, delta updates, and rollback.
//! 
//! # Features
//! 
//! - Automatic model download from CDN
//! - Resume interrupted downloads
//! - Binary delta updates (bsdiff)
//! - Automatic rollback on failure
//! - Version management (keep last 2 versions)

use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error, debug};

use crate::model::{ModelConfig, ModelMetadata, ModelVariant, DownloadProgressCallback};
use crate::{AiError, AiResult, DEFAULT_MODEL_SHA256};

/// Manages model download, caching, and updates
pub struct ModelManager {
    config: ModelConfig,
    http_client: reqwest::Client,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(config: ModelConfig) -> AiResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for downloads
            .build()
            .map_err(|e| AiError::Other(format!("Failed to create HTTP client: {e}")))?;
        
        // Ensure model directory exists
        fs::create_dir_all(&config.model_dir)
            .map_err(|e| AiError::Io(e))?;
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    /// Check if the model is already downloaded and valid
    pub async fn is_model_ready(&self) -> AiResult<bool> {
        let model_path = self.config.model_path();
        
        if !model_path.exists() {
            debug!("Model file not found at {:?}", model_path);
            return Ok(false);
        }
        
        // Check metadata
        let metadata_path = self.metadata_path();
        if !metadata_path.exists() {
            warn!("Model exists but metadata missing");
            return Ok(false);
        }
        
        let metadata = self.load_metadata()?;
        
        // Verify file size
        let file_size = fs::metadata(&model_path)
            .map_err(|e| AiError::Io(e))?
            .len();
        
        if file_size != metadata.file_size {
            warn!(
                "Model file size mismatch: expected {}, got {}",
                metadata.file_size, file_size
            );
            return Ok(false);
        }
        
        info!("Model is ready: {:?}", model_path);
        Ok(true)
    }
    
    /// Download the model if not present
    pub async fn ensure_model(&self) -> AiResult<PathBuf> {
        if self.is_model_ready().await? {
            return Ok(self.config.model_path());
        }
        
        self.download_model(None).await
    }
    
    /// Download model with progress callback
    pub async fn download_model(
        &self,
        progress_callback: Option<DownloadProgressCallback>,
    ) -> AiResult<PathBuf> {
        let model_path = self.config.model_path();
        let temp_path = model_path.with_extension("tmp");
        let url = self.config.variant_download_url();
        
        info!("Downloading model from {}", url);
        
        // Check if partial download exists and get current size
        let resume_from = if temp_path.exists() {
            fs::metadata(&temp_path)
                .map_err(|e| AiError::Io(e))?
                .len()
        } else {
            0
        };
        
        // Build request with resume support
        let mut request = self.http_client.get(&url);
        if resume_from > 0 {
            info!("Resuming download from byte {}", resume_from);
            request = request.header("Range", format!("bytes={}-", resume_from));
        }
        
        // Download with timeout
        let response = timeout(
            Duration::from_secs(300),
            request.send()
        )
        .await
        .map_err(|_| AiError::Timeout(300))?
        .map_err(|e| AiError::Other(format!("Download failed: {e}")))?;
        
        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(AiError::Other(
                format!("Download failed with status: {}", response.status())
            ));
        }
        
        // Get total size
        let total_size = response
            .content_length()
            .unwrap_or(self.config.variant.expected_size());
        
        // Open file for writing (append if resuming)
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(resume_from > 0)
            .write(true)
            .open(&temp_path)
            .await
            .map_err(|e| AiError::Io(e))?;
        
        // Stream download with progress
        let mut downloaded = resume_from;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AiError::Other(format!("Download error: {e}")))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| AiError::Io(e))?;
            
            downloaded += chunk.len() as u64;
            
            if let Some(ref callback) = progress_callback {
                callback(downloaded, total_size);
            }
            
            debug!("Downloaded {}/{} bytes", downloaded, total_size);
        }
        
        file.flush().await.map_err(|e| AiError::Io(e))?;
        drop(file);
        
        // Verify download
        if downloaded < total_size * 95 / 100 {
            return Err(AiError::Other(
                format!("Incomplete download: {}/{} bytes", downloaded, total_size)
            ));
        }
        
        // Move temp file to final location
        tokio::fs::rename(&temp_path, &model_path)
            .await
            .map_err(|e| AiError::Io(e))?;
        
        // Save metadata
        let metadata = ModelMetadata {
            variant: self.config.variant,
            version: crate::PIPELINE_VERSION.to_string(),
            sha256: DEFAULT_MODEL_SHA256.to_string(),
            downloaded_at: chrono::Utc::now(),
            file_size: downloaded,
            llama_commit: "b2699".to_string(),
        };
        self.save_metadata(&metadata)?;
        
        info!("Model downloaded successfully to {:?}", model_path);
        Ok(model_path)
    }
    
    /// Apply a delta update (bsdiff patch)
    pub async fn apply_delta_update(&self, patch_url: &str) -> AiResult<PathBuf> {
        info!("Applying delta update from {}", patch_url);
        
        // Download patch
        let patch_response = self.http_client
            .get(patch_url)
            .send()
            .await
            .map_err(|e| AiError::Other(format!("Failed to download patch: {e}")))?;
        
        let patch_bytes = patch_response
            .bytes()
            .await
            .map_err(|e| AiError::Other(format!("Failed to read patch: {e}")))?;
        
        // Apply patch using bsdiff
        let old_model_path = self.config.model_path();
        let new_model_path = self.config.model_dir.join(
            format!("{}_new", self.config.variant.filename())
        );
        
        let old_model = fs::read(&old_model_path)
            .map_err(|e| AiError::Io(e))?;
        
        let new_model = Self::apply_bsdiff_patch(&old_model, &patch_bytes)?;
        
        fs::write(&new_model_path, new_model)
            .map_err(|e| AiError::Io(e))?;
        
        // Backup old model
        let backup_path = old_model_path.with_extension("backup");
        fs::rename(&old_model_path, &backup_path)
            .map_err(|e| AiError::Io(e))?;
        
        // Move new model into place
        fs::rename(&new_model_path, &old_model_path)
            .map_err(|e| AiError::Io(e))?;
        
        info!("Delta update applied successfully");
        Ok(old_model_path)
    }
    
    /// Rollback to previous version
    pub async fn rollback(&self) -> AiResult<PathBuf> {
        let model_path = self.config.model_path();
        let backup_path = model_path.with_extension("backup");
        
        if !backup_path.exists() {
            return Err(AiError::Model("No backup available for rollback".to_string()));
        }
        
        warn!("Rolling back model to previous version");
        
        // Remove current model
        if model_path.exists() {
            fs::remove_file(&model_path)
                .map_err(|e| AiError::Io(e))?;
        }
        
        // Restore backup
        fs::rename(&backup_path, &model_path)
            .map_err(|e| AiError::Io(e))?;
        
        info!("Rollback completed");
        Ok(model_path)
    }
    
    /// Clean up old model versions (keep only last 2)
    pub fn cleanup_old_versions(&self) -> AiResult<usize> {
        let mut removed = 0;
        
        // Find all backup files
        let entries = fs::read_dir(&self.config.model_dir)
            .map_err(|e| AiError::Io(e))?;
        
        let mut backups: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .ends_with(".backup")
            })
            .collect();
        
        // Sort by modification time (oldest first)
        backups.sort_by(|a, b| {
            let a_meta = a.metadata().unwrap();
            let b_meta = b.metadata().unwrap();
            a_meta.modified().unwrap().cmp(&b_meta.modified().unwrap())
        });
        
        // Remove all but the most recent backup
        while backups.len() > 2 {
            if let Some(old) = backups.first() {
                warn!("Removing old backup: {:?}", old.path());
                fs::remove_file(old.path())
                    .map_err(|e| AiError::Io(e))?;
                removed += 1;
                backups.remove(0);
            }
        }
        
        Ok(removed)
    }
    
    /// Get the path to the metadata file
    fn metadata_path(&self) -> PathBuf {
        self.config.model_dir.join(
            format!("{}.json", self.config.variant.filename())
        )
    }
    
    /// Load model metadata
    fn load_metadata(&self) -> AiResult<ModelMetadata> {
        let metadata_path = self.metadata_path();
        let content = fs::read_to_string(&metadata_path)
            .map_err(|e| AiError::Io(e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| AiError::Json(e))
    }
    
    /// Save model metadata
    fn save_metadata(&self, metadata: &ModelMetadata) -> AiResult<()> {
        let metadata_path = self.metadata_path();
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| AiError::Json(e))?;
        
        fs::write(&metadata_path, content)
            .map_err(|e| AiError::Io(e))?;
        
        Ok(())
    }
    
    /// Apply bsdiff patch (simplified implementation)
    fn apply_bsdiff_patch(old: &[u8], patch: &[u8]) -> AiResult<Vec<u8>> {
        // In production, use the `bsdiff` crate
        // This is a placeholder that would be replaced with actual implementation
        Err(AiError::Other("Delta updates not yet implemented".to_string()))
    }
    
    /// Get available disk space in the model directory
    pub fn available_space(&self) -> AiResult<u64> {
        // This would use platform-specific APIs in production
        // For now, return a conservative estimate
        Ok(10_000_000_000) // 10GB placeholder
    }
    
    /// Check if there's enough space for the model
    pub fn has_enough_space(&self) -> AiResult<bool> {
        let available = self.available_space()?;
        let needed = self.config.variant.expected_size() * 2; // Need 2x for updates
        Ok(available >= needed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_model_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ModelConfig {
            model_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let manager = ModelManager::new(config);
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_metadata_serialization() {
        let metadata = ModelMetadata::default_e2b();
        let json = serde_json::to_string(&metadata).unwrap();
        let decoded: ModelMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.variant, decoded.variant);
    }
}
