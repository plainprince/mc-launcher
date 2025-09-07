//! Minecraft version management and mod loader support

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::error::{LauncherError, Result};

/// Minecraft version manifest from Mojang
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Latest version information
    pub latest: LatestVersions,
    /// List of all available versions
    pub versions: Vec<VersionEntry>,
}

/// Latest version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    /// Latest release version
    pub release: String,
    /// Latest snapshot version
    pub snapshot: String,
}

/// Version entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Version ID (e.g., "1.21.4")
    pub id: String,
    /// Version type (release, snapshot, old_beta, old_alpha)
    #[serde(rename = "type")]
    pub version_type: String,
    /// URL to download the version JSON
    pub url: String,
    /// Release time
    pub time: DateTime<Utc>,
    /// Release time (different format)
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// SHA1 hash of the version JSON
    pub sha1: String,
    /// Compliance level
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
}

/// Complete version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Arguments for the game and JVM
    pub arguments: Option<Arguments>,
    /// Asset index information
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    /// Assets version
    pub assets: String,
    /// Compliance level
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
    /// Downloads information
    pub downloads: Downloads,
    /// Version ID
    pub id: String,
    /// Java version requirements
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    /// Libraries required by this version
    pub libraries: Vec<Library>,
    /// Logging configuration
    pub logging: Option<LoggingConfig>,
    /// Main class to launch
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// Minecraft arguments (legacy format)
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    /// Minimum launcher version
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<i32>,
    /// Release time
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// Time
    pub time: DateTime<Utc>,
    /// Version type
    #[serde(rename = "type")]
    pub version_type: String,
}

/// Game and JVM arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    /// Game arguments
    pub game: Vec<ArgumentValue>,
    /// JVM arguments
    pub jvm: Vec<ArgumentValue>,
}

/// Argument value (can be string or conditional)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    Conditional {
        rules: Vec<Rule>,
        value: Vec<String>,
    },
}

/// Rule for conditional arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Action to take (allow/disallow)
    pub action: String,
    /// Operating system rule
    pub os: Option<OsRule>,
    /// Features rule
    pub features: Option<HashMap<String, bool>>,
}

/// Operating system rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    /// OS name (windows, osx, linux)
    pub name: Option<String>,
    /// OS version regex
    pub version: Option<String>,
    /// OS architecture
    pub arch: Option<String>,
}

/// Asset index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    /// Asset index ID
    pub id: String,
    /// SHA1 hash
    pub sha1: String,
    /// File size
    pub size: u64,
    /// Total size of all assets
    #[serde(rename = "totalSize")]
    pub total_size: Option<u64>,
    /// Download URL
    pub url: String,
}

/// Download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    /// Client download info
    pub client: DownloadInfo,
    /// Client mappings (optional)
    pub client_mappings: Option<DownloadInfo>,
    /// Server download info (optional)
    pub server: Option<DownloadInfo>,
    /// Server mappings (optional)
    pub server_mappings: Option<DownloadInfo>,
    /// Windows server (optional)
    pub windows_server: Option<DownloadInfo>,
}

/// Download information for a specific file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// SHA1 hash
    pub sha1: String,
    /// File size
    pub size: u64,
    /// Download URL
    pub url: String,
}

/// Java version requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    /// Component (e.g., "java-runtime-gamma")
    pub component: String,
    /// Major version (e.g., 21)
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

/// Library information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// Library downloads
    pub downloads: Option<LibraryDownloads>,
    /// Library name (Maven coordinate)
    pub name: String,
    /// Rules for when this library applies
    pub rules: Option<Vec<Rule>>,
    /// Natives information
    pub natives: Option<HashMap<String, String>>,
    /// Extract information
    pub extract: Option<ExtractInfo>,
}

/// Library download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    /// Artifact download
    pub artifact: Option<DownloadInfo>,
    /// Classifiers (for natives)
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
}

/// Extract information for native libraries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractInfo {
    /// Files to exclude from extraction
    pub exclude: Option<Vec<String>>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Client logging configuration
    pub client: Option<LoggingClient>,
}

/// Client logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    /// Argument to pass to the JVM
    pub argument: String,
    /// File information
    pub file: DownloadInfo,
    /// Type of logging
    #[serde(rename = "type")]
    pub logging_type: String,
}

/// Mod loader types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModLoaderType {
    #[serde(rename = "forge")]
    Forge,
    #[serde(rename = "fabric")]
    Fabric,
    #[serde(rename = "quilt")]
    Quilt,
    #[serde(rename = "neoforge")]
    NeoForge,
    #[serde(rename = "legacy-fabric")]
    LegacyFabric,
}

impl std::fmt::Display for ModLoaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModLoaderType::Forge => write!(f, "forge"),
            ModLoaderType::Fabric => write!(f, "fabric"),
            ModLoaderType::Quilt => write!(f, "quilt"),
            ModLoaderType::NeoForge => write!(f, "neoforge"),
            ModLoaderType::LegacyFabric => write!(f, "legacy-fabric"),
        }
    }
}

/// Mod loader information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoader {
    /// Loader type
    pub loader_type: ModLoaderType,
    /// Loader version
    pub version: String,
    /// Minecraft version this loader supports
    pub minecraft_version: String,
    /// Whether this loader is stable
    pub stable: bool,
    /// Download URL or build information
    pub build_info: ModLoaderBuildInfo,
}

/// Mod loader build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoaderBuildInfo {
    /// Build number or version
    pub build: String,
    /// Download URL
    pub url: Option<String>,
    /// Maven coordinate (for Fabric, etc.)
    pub maven: Option<String>,
    /// Additional libraries required
    pub libraries: Vec<Library>,
}

/// Version manager for fetching and caching version information
pub struct VersionManager {
    client: reqwest::Client,
    #[allow(dead_code)]
    cache_dir: std::path::PathBuf,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new(cache_dir: std::path::PathBuf) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(format!("MinecraftLauncher/{}", crate::VERSION))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| LauncherError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, cache_dir })
    }

    /// Fetch the version manifest from Mojang
    pub async fn fetch_version_manifest(&self) -> Result<VersionManifest> {
        let response = self.client
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to fetch version manifest: {}", e)))?;

        let manifest: VersionManifest = response
            .json()
            .await
            .map_err(|e| LauncherError::json(format!("Failed to parse version manifest: {}", e)))?;

        Ok(manifest)
    }

    /// Fetch detailed version information for a specific version
    pub async fn fetch_version_info(&self, version_entry: &VersionEntry) -> Result<VersionInfo> {
        let response = self.client
            .get(&version_entry.url)
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to fetch version info: {}", e)))?;

        let version_info: VersionInfo = response
            .json()
            .await
            .map_err(|e| LauncherError::json(format!("Failed to parse version info: {}", e)))?;

        Ok(version_info)
    }

    /// Get available mod loader versions for a Minecraft version
    pub async fn get_mod_loader_versions(
        &self,
        loader_type: ModLoaderType,
        minecraft_version: &str,
    ) -> Result<Vec<ModLoader>> {
        match loader_type {
            ModLoaderType::Forge => self.get_forge_versions(minecraft_version).await,
            ModLoaderType::Fabric => self.get_fabric_versions(minecraft_version).await,
            ModLoaderType::Quilt => self.get_quilt_versions(minecraft_version).await,
            ModLoaderType::NeoForge => self.get_neoforge_versions(minecraft_version).await,
            ModLoaderType::LegacyFabric => self.get_legacy_fabric_versions(minecraft_version).await,
        }
    }

    /// Find a version entry by ID
    pub async fn find_version(&self, version_id: &str) -> Result<VersionEntry> {
        let manifest = self.fetch_version_manifest().await?;
        
        manifest.versions
            .into_iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| LauncherError::version_not_found(version_id))
    }

    /// Get the latest release version
    pub async fn get_latest_release(&self) -> Result<VersionEntry> {
        let manifest = self.fetch_version_manifest().await?;
        self.find_version(&manifest.latest.release).await
    }

    /// Get the latest snapshot version
    pub async fn get_latest_snapshot(&self) -> Result<VersionEntry> {
        let manifest = self.fetch_version_manifest().await?;
        self.find_version(&manifest.latest.snapshot).await
    }

    // Private methods for specific mod loader APIs

    async fn get_forge_versions(&self, _minecraft_version: &str) -> Result<Vec<ModLoader>> {
        // Implement Forge API integration
        // This would fetch from https://files.minecraftforge.net/net/minecraftforge/forge/
        Ok(Vec::new()) // Placeholder
    }

    async fn get_fabric_versions(&self, _minecraft_version: &str) -> Result<Vec<ModLoader>> {
        // Implement Fabric API integration
        // This would fetch from https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}
        Ok(Vec::new()) // Placeholder
    }

    async fn get_quilt_versions(&self, _minecraft_version: &str) -> Result<Vec<ModLoader>> {
        // Implement Quilt API integration
        Ok(Vec::new()) // Placeholder
    }

    async fn get_neoforge_versions(&self, _minecraft_version: &str) -> Result<Vec<ModLoader>> {
        // Implement NeoForge API integration
        Ok(Vec::new()) // Placeholder
    }

    async fn get_legacy_fabric_versions(&self, _minecraft_version: &str) -> Result<Vec<ModLoader>> {
        // Implement Legacy Fabric API integration
        Ok(Vec::new()) // Placeholder
    }
}
