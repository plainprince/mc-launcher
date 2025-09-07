//! Utility functions and helpers

use crate::error::{LauncherError, Result};
use futures::StreamExt;
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Java installation finder
pub struct JavaFinder {
    java_cache: tokio::sync::RwLock<std::collections::HashMap<i32, PathBuf>>,
}

impl JavaFinder {
    /// Create a new Java finder
    pub fn new() -> Self {
        Self {
            java_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Find a Java installation for the specified major version
    pub async fn find_java(&self, major_version: i32) -> Result<PathBuf> {
        // Check cache first
        {
            let cache = self.java_cache.read().await;
            if let Some(path) = cache.get(&major_version) {
                if self.verify_java_version(path, major_version).await? {
                    return Ok(path.clone());
                }
            }
        }

        // Search for Java installations
        let java_path = self.search_java_installations(major_version).await?;
        
        // Cache the result
        {
            let mut cache = self.java_cache.write().await;
            cache.insert(major_version, java_path.clone());
        }

        Ok(java_path)
    }

    /// Search for Java installations on the system
    async fn search_java_installations(&self, major_version: i32) -> Result<PathBuf> {
        let search_paths = self.get_java_search_paths();
        
        for search_path in search_paths {
            if let Ok(java_path) = self.find_java_in_directory(&search_path, major_version).await {
                return Ok(java_path);
            }
        }

        // Try system PATH as fallback
        if let Ok(java_path) = self.find_java_in_path(major_version).await {
            return Ok(java_path);
        }

        Err(LauncherError::config(format!(
            "Could not find Java {} installation",
            major_version
        )))
    }

    /// Get platform-specific Java search paths
    fn get_java_search_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // Windows search paths
            if let Some(program_files) = std::env::var_os("ProgramFiles") {
                paths.push(PathBuf::from(program_files).join("Java"));
                paths.push(PathBuf::from(program_files).join("Eclipse Adoptium"));
                paths.push(PathBuf::from(program_files).join("Microsoft"));
            }
            if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
                paths.push(PathBuf::from(program_files_x86).join("Java"));
                paths.push(PathBuf::from(program_files_x86).join("Eclipse Adoptium"));
            }
            // Minecraft launcher Java
            if let Some(appdata) = std::env::var_os("APPDATA") {
                paths.push(PathBuf::from(appdata).join(".minecraft").join("runtime"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS search paths
            paths.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
            paths.push(PathBuf::from("/System/Library/Java/JavaVirtualMachines"));
            
            // Homebrew paths
            paths.push(PathBuf::from("/opt/homebrew/opt"));
            paths.push(PathBuf::from("/usr/local/opt"));
            
            // Minecraft launcher Java
            if let Some(home) = std::env::var_os("HOME") {
                paths.push(PathBuf::from(home).join("Library/Application Support/minecraft/runtime"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux search paths
            paths.push(PathBuf::from("/usr/lib/jvm"));
            paths.push(PathBuf::from("/usr/java"));
            paths.push(PathBuf::from("/opt/java"));
            paths.push(PathBuf::from("/usr/lib64/jvm"));
            
            // Snap packages
            paths.push(PathBuf::from("/snap"));
            
            // Minecraft launcher Java
            if let Some(home) = std::env::var_os("HOME") {
                paths.push(PathBuf::from(home).join(".minecraft/runtime"));
            }
        }

        paths
    }

    /// Find Java in a specific directory
    async fn find_java_in_directory(&self, dir: &PathBuf, major_version: i32) -> Result<PathBuf> {
        if !dir.exists() {
            return Err(LauncherError::config("Directory does not exist"));
        }

        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to read directory: {}", e)))?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                // Check if this looks like a Java installation
                if let Ok(java_exe) = self.find_java_executable(&path).await {
                    if self.verify_java_version(&java_exe, major_version).await.unwrap_or(false) {
                        return Ok(java_exe);
                    }
                }
            }
        }

        Err(LauncherError::config("Java not found in directory"))
    }

    /// Find Java executable in PATH
    async fn find_java_in_path(&self, major_version: i32) -> Result<PathBuf> {
        let java_exe = if cfg!(windows) { "java.exe" } else { "java" };
        
        if let Some(path_var) = std::env::var_os("PATH") {
            for path in std::env::split_paths(&path_var) {
                let java_path = path.join(java_exe);
                if java_path.exists() {
                    if self.verify_java_version(&java_path, major_version).await.unwrap_or(false) {
                        return Ok(java_path);
                    }
                }
            }
        }

        Err(LauncherError::config("Java not found in PATH"))
    }

    /// Find Java executable in a Java installation directory
    async fn find_java_executable(&self, java_dir: &PathBuf) -> Result<PathBuf> {
        let java_exe = if cfg!(windows) { "java.exe" } else { "java" };
        
        // Common locations within a Java installation
        let candidates = [
            java_dir.join("bin").join(java_exe),
            java_dir.join("Contents/Home/bin").join(java_exe), // macOS
            java_dir.join(java_exe),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Ok(candidate.clone());
            }
        }

        Err(LauncherError::config("Java executable not found"))
    }

    /// Verify that a Java executable matches the required major version
    async fn verify_java_version(&self, java_path: &PathBuf, required_major: i32) -> Result<bool> {
        let output = tokio::process::Command::new(java_path)
            .arg("-version")
            .output()
            .await
            .map_err(|e| LauncherError::process(format!("Failed to run java -version: {}", e)))?;

        let version_output = String::from_utf8_lossy(&output.stderr);
        let actual_major = self.parse_java_major_version(&version_output)?;
        
        Ok(actual_major == required_major)
    }

    /// Parse major version from java -version output
    fn parse_java_major_version(&self, version_output: &str) -> Result<i32> {
        // Look for version patterns like:
        // - java version "1.8.0_XXX" (Java 8)
        // - java version "11.0.X" (Java 11)
        // - openjdk version "17.0.X" (Java 17)
        
        for line in version_output.lines() {
            if line.contains("version") {
                // Extract version string between quotes
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let version_str = &line[start + 1..start + 1 + end];
                        
                        // Parse version
                        if version_str.starts_with("1.") {
                            // Java 8 and below: "1.8.0_XXX" -> 8
                            if let Some(major_str) = version_str.split('.').nth(1) {
                                if let Ok(major) = major_str.parse::<i32>() {
                                    return Ok(major);
                                }
                            }
                        } else {
                            // Java 9+: "11.0.X" -> 11
                            if let Some(major_str) = version_str.split('.').next() {
                                if let Ok(major) = major_str.parse::<i32>() {
                                    return Ok(major);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(LauncherError::config("Could not parse Java version"))
    }

    /// List all available Java installations
    pub async fn list_java_installations(&self) -> Vec<(i32, PathBuf)> {
        let mut installations = Vec::new();
        let search_paths = self.get_java_search_paths();

        for search_path in search_paths {
            if let Ok(mut entries) = tokio::fs::read_dir(&search_path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(java_exe) = self.find_java_executable(&path).await {
                            if let Ok(output) = tokio::process::Command::new(&java_exe)
                                .arg("-version")
                                .output()
                                .await
                            {
                                let version_output = String::from_utf8_lossy(&output.stderr);
                                if let Ok(major_version) = self.parse_java_major_version(&version_output) {
                                    installations.push((major_version, java_exe));
                                }
                            }
                        }
                    }
                }
            }
        }

        installations
    }
}

impl Default for JavaFinder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) async fn download_file(
    client: &Client,
    url: &str,
    path: &Path,
    sha1: Option<&str>,
) -> Result<()> {
    if path.exists() {
        if let Some(sha1) = sha1 {
            let mut file = tokio::fs::File::open(path).await?;
            let mut hasher = Sha1::new();
            let mut buffer = [0; 1024];
            loop {
                let n = file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            let hash = format!("{:x}", hasher.finalize());
            if hash == sha1 {
                return Ok(());
            }
        }
    }

    let temp_path = path.with_extension("tmp");
    let response = client.get(url).send().await?;
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&temp_path).await?;

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
    }

    tokio::fs::rename(&temp_path, path).await?;

    Ok(())
}
