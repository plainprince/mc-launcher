//! File downloader with progress tracking and verification

use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use crate::error::{LauncherError, Result};

/// File downloader with concurrent download support
pub struct Downloader {
    client: reqwest::Client,
    concurrent_downloads: usize,
    timeout: u64,
}

impl Downloader {
    /// Create a new downloader
    pub fn new(concurrent_downloads: usize, timeout: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(format!("MinecraftLauncher/{}", crate::VERSION))
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| LauncherError::download(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            concurrent_downloads,
            timeout,
        })
    }

    /// Download a single file
    pub async fn download_file(
        &self,
        url: &str,
        destination: &PathBuf,
        expected_hash: Option<&str>,
    ) -> Result<()> {
        // Check if file already exists and is valid
        if let Some(hash) = expected_hash {
            if destination.exists() {
                if let Ok(existing_hash) = self.calculate_sha1(destination).await {
                    if existing_hash == hash {
                        log::debug!("File {} already exists with correct hash", destination.display());
                        return Ok(());
                    }
                }
            }
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| LauncherError::file(format!("Failed to create directory {}: {}", parent.display(), e)))?;
        }

        log::debug!("Downloading {} to {}", url, destination.display());

        // Download the file
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| LauncherError::download(format!("Failed to start download from {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(LauncherError::download(format!(
                "HTTP error {} when downloading from {}",
                response.status(),
                url
            )));
        }

        // Stream the response to a temporary file
        let temp_path = destination.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to create temporary file {}: {}", temp_path.display(), e)))?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| LauncherError::download(format!("Failed to read chunk: {}", e)))?;
            
            file.write_all(&chunk)
                .await
                .map_err(|e| LauncherError::file(format!("Failed to write chunk: {}", e)))?;
        }

        file.flush()
            .await
            .map_err(|e| LauncherError::file(format!("Failed to flush file: {}", e)))?;

        drop(file);

        // Verify hash if provided
        if let Some(expected_hash) = expected_hash {
            let actual_hash = self.calculate_sha1(&temp_path).await?;
            if actual_hash != expected_hash {
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(LauncherError::validation(format!(
                    "Hash mismatch for {}: expected {}, got {}",
                    destination.display(),
                    expected_hash,
                    actual_hash
                )));
            }
        }

        // Move temporary file to final destination
        tokio::fs::rename(&temp_path, destination)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to move file to final destination: {}", e)))?;

        log::debug!("Successfully downloaded {}", destination.display());
        Ok(())
    }

    /// Download multiple files concurrently
    pub async fn download_files(&self, downloads: Vec<(String, PathBuf, String)>) -> Result<()> {
        if downloads.is_empty() {
            return Ok(());
        }

        log::info!("Starting download of {} files", downloads.len());

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.concurrent_downloads));
        let mut tasks = Vec::new();

        for (url, path, hash) in downloads {
            let semaphore = semaphore.clone();
            let downloader = self.clone();
            
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                downloader.download_file(&url, &path, Some(&hash)).await
            });
            
            tasks.push(task);
        }

        // Wait for all downloads to complete
        let mut failed_downloads = Vec::new();
        for (i, task) in tasks.into_iter().enumerate() {
            match task.await {
                Ok(Ok(())) => {
                    log::debug!("Download {} completed successfully", i);
                }
                Ok(Err(e)) => {
                    log::error!("Download {} failed: {}", i, e);
                    failed_downloads.push(e);
                }
                Err(e) => {
                    log::error!("Download task {} panicked: {}", i, e);
                    failed_downloads.push(LauncherError::download(format!("Task panicked: {}", e)));
                }
            }
        }

        if !failed_downloads.is_empty() {
            return Err(LauncherError::download(format!(
                "{} downloads failed. First error: {}",
                failed_downloads.len(),
                failed_downloads[0]
            )));
        }

        log::info!("All downloads completed successfully");
        Ok(())
    }

    /// Calculate SHA1 hash of a file
    async fn calculate_sha1(&self, file_path: &PathBuf) -> Result<String> {
        use sha1::{Sha1, Digest};
        
        let content = tokio::fs::read(file_path)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to read file for hashing: {}", e)))?;

        let mut hasher = Sha1::new();
        hasher.update(&content);
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }

    /// Get download progress information
    pub async fn download_file_with_progress<F>(
        &self,
        url: &str,
        destination: &PathBuf,
        expected_hash: Option<&str>,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        // Check if file already exists and is valid
        if let Some(hash) = expected_hash {
            if destination.exists() {
                if let Ok(existing_hash) = self.calculate_sha1(destination).await {
                    if existing_hash == hash {
                        log::debug!("File {} already exists with correct hash", destination.display());
                        return Ok(());
                    }
                }
            }
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| LauncherError::file(format!("Failed to create directory {}: {}", parent.display(), e)))?;
        }

        log::debug!("Downloading {} to {}", url, destination.display());

        // Start the download
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| LauncherError::download(format!("Failed to start download from {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(LauncherError::download(format!(
                "HTTP error {} when downloading from {}",
                response.status(),
                url
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let temp_path = destination.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to create temporary file {}: {}", temp_path.display(), e)))?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| LauncherError::download(format!("Failed to read chunk: {}", e)))?;
            
            file.write_all(&chunk)
                .await
                .map_err(|e| LauncherError::file(format!("Failed to write chunk: {}", e)))?;

            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        file.flush()
            .await
            .map_err(|e| LauncherError::file(format!("Failed to flush file: {}", e)))?;

        drop(file);

        // Verify hash if provided
        if let Some(expected_hash) = expected_hash {
            let actual_hash = self.calculate_sha1(&temp_path).await?;
            if actual_hash != expected_hash {
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(LauncherError::validation(format!(
                    "Hash mismatch for {}: expected {}, got {}",
                    destination.display(),
                    expected_hash,
                    actual_hash
                )));
            }
        }

        // Move temporary file to final destination
        tokio::fs::rename(&temp_path, destination)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to move file to final destination: {}", e)))?;

        progress_callback(total_size, total_size); // 100% complete
        log::debug!("Successfully downloaded {}", destination.display());
        Ok(())
    }
}

// Implement Clone for Downloader to allow sharing across tasks
impl Clone for Downloader {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            concurrent_downloads: self.concurrent_downloads,
            timeout: self.timeout,
        }
    }
}
