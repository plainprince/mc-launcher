//! This module handles the downloading and management of Java runtimes.

use crate::error::LauncherError;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

// Azul Zulu API URL (better Java 8 support than Adoptium)
const AZUL_API_URL: &str = "https://api.azul.com/metadata/v1/zulu/packages";

#[derive(Debug, Serialize, Deserialize)]
struct ZuluPackage {
    name: String,
    download_url: String,
    sha256_hash: Option<String>,
}

/// Manages Java runtimes for Minecraft.
#[derive(Debug)]
pub struct JavaManager {
    runtime_dir: PathBuf,
    client: reqwest::Client,
}

impl JavaManager {
    /// Creates a new `JavaManager`.
    pub fn new(runtime_dir: PathBuf) -> Self {
        Self {
            runtime_dir,
            client: reqwest::Client::new(),
        }
    }

    /// Gets the path to a suitable Java runtime for the given Minecraft version.
    pub async fn get_java_runtime(&self, version: &str) -> Result<Option<PathBuf>, LauncherError> {
        let major_version = self.get_required_java_version(version).await?;
        self.find_java_runtime(major_version)
    }

    /// Downloads and installs a suitable Java runtime using the Azul Zulu API.
    pub async fn download_java_runtime(&self, version: &str) -> Result<PathBuf, LauncherError> {
        let major_version = self.get_required_java_version(version).await?;

        log::info!(
            "No suitable Java runtime found, attempting to download Java {} from Azul Zulu...",
            major_version
        );

        let (os, arch) = self.get_os_arch();
        let url = format!(
            "{}?java_version={}&os={}&arch={}&archive_type=zip&java_package_type=jre",
            AZUL_API_URL, major_version, os, arch
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Azul API Error for URL {}: {}", url, error_text);
            return Err(LauncherError::java(format!(
                "Failed to find a download for Java {} on Azul. Status: {}",
                major_version,
                status
            )));
        }

        let packages: Vec<ZuluPackage> = response.json().await?;
        let package = packages.get(0).ok_or_else(|| {
            LauncherError::java(format!(
                "No download package found for Java {}",
                major_version
            ))
        })?;

        let download_url = &package.download_url;
        let file_name = &package.name;
        let download_path = self.runtime_dir.join(file_name);
        
        // Note: Azul provides sha256, but for simplicity we are not verifying it here.
        // In a production-ready launcher, you would want to implement sha256 verification.
        crate::utils::download_file(&self.client, download_url, &download_path, None).await?;

        let extraction_dir_name = self.get_extraction_dir_name(file_name);
        let extraction_path = self.runtime_dir.join(extraction_dir_name);
        self.extract_archive(&download_path, &extraction_path)?;

        self.find_java_runtime(major_version)?
            .ok_or_else(|| LauncherError::java("Failed to find Java runtime after extraction".to_string()))
    }

    fn get_extraction_dir_name(&self, file_name: &str) -> String {
        let base_name = file_name
            .replace(".tar.gz", "")
            .replace(".zip", "");
        
        // On ARM64, we download x86_64 Java for Rosetta 2 compatibility
        if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
            log::info!("Using x86_64 Java runtime for Rosetta 2 compatibility on ARM64");
        }
        
        base_name
    }

    /// Gets the OS and architecture in the format required by the Azul API.
    fn get_os_arch(&self) -> (&'static str, &'static str) {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            // For ARM64 systems, download x86_64 Java to run under Rosetta 2 for pre-1.17 Minecraft compatibility
            log::info!("ARM64 detected: downloading x86_64 Java runtime for Rosetta 2 compatibility");
            "x64"
        } else {
            "x32"
        };

        (os, arch)
    }

    /// Extracts the downloaded archive.
    fn extract_archive(&self, archive_path: &Path, extraction_path: &Path) -> Result<(), LauncherError> {
        let file = std::fs::File::open(archive_path)?;
        if archive_path.extension().map_or(false, |e| e == "gz") {
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(extraction_path)?;
        } else if archive_path.extension().map_or(false, |e| e == "zip") {
            let mut archive = ZipArchive::new(file)?;
            archive.extract(extraction_path)?;
        }
        Ok(())
    }

    /// Finds a Java runtime for the given major version in the runtime directory.
    fn find_java_runtime(&self, major_version: u32) -> Result<Option<PathBuf>, LauncherError> {
        if !self.runtime_dir.exists() {
            std::fs::create_dir_all(&self.runtime_dir)?;
            return Ok(None);
        }

        for entry in std::fs::read_dir(&self.runtime_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(executable) = self.find_java_executable(&path) {
                    if self.check_java_version(&executable, major_version)? {
                        return Ok(Some(executable));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Finds the Java executable within a given directory.
    fn find_java_executable(&self, dir: &Path) -> Option<PathBuf> {
        let executable_name = if cfg!(windows) { "java.exe" } else { "java" };
        
        // For Azul Zulu, we need to search recursively since the structure varies
        self.find_java_executable_recursive(dir, executable_name)
    }
    
    /// Recursively searches for the Java executable in the directory tree
    fn find_java_executable_recursive(&self, dir: &Path, executable_name: &str) -> Option<PathBuf> {
        // First check direct bin directory
        let direct_bin = dir.join("bin").join(executable_name);
        if direct_bin.exists() {
            return Some(direct_bin);
        }
        
        // Check macOS structure
        let macos_bin = dir.join("Contents").join("Home").join("bin").join(executable_name);
        if macos_bin.exists() {
            return Some(macos_bin);
        }
        
        // Recursively search subdirectories (common with Azul Zulu extractions)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = self.find_java_executable_recursive(&path, executable_name) {
                        return Some(found);
                    }
                }
            }
        }
        
        None
    }

    /// Checks if the Java executable at the given path has the correct major version.
    fn check_java_version(
        &self,
        java_path: &Path,
        expected_major_version: u32,
    ) -> Result<bool, LauncherError> {
        let output = std::process::Command::new(java_path).arg("-version").output()?;
        let version_string = String::from_utf8_lossy(&output.stderr);

        if let Some(line) = version_string.lines().next() {
            let version_pattern = format!("\"{}", expected_major_version);
            let legacy_pattern = format!("\"1.{}", expected_major_version);
            if line.contains(&version_pattern) || line.contains(&legacy_pattern) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Gets the required Java major version for the given Minecraft version.
    async fn get_required_java_version(&self, version: &str) -> Result<u32, LauncherError> {
        let version_parts: Vec<&str> = version.split('.').collect();
        if version_parts.len() >= 2 {
            let major: u32 = version_parts[0].parse().unwrap_or(1);
            let minor: u32 = version_parts[1].parse().unwrap_or(0);

            if major == 1 {
                return match minor {
                    v if v >= 20 => Ok(21),
                    v if v >= 17 => Ok(17),
                    _ => Ok(8),
                };
            }
        }
        Ok(8) // Default to Java 8
    }
}
