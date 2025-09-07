//! Configuration types for the Minecraft launcher

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::auth::Account;
use crate::version::ModLoaderType;

/// Main launcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    /// Root directory for Minecraft files
    pub minecraft_dir: PathBuf,
    /// Java executable path (optional, will auto-detect if None)
    pub java_path: Option<PathBuf>,
    /// JVM arguments
    pub jvm_args: Vec<String>,
    /// Game arguments
    pub game_args: Vec<String>,
    /// Memory allocation (in MB)
    pub memory_min: u32,
    pub memory_max: u32,
    /// Download timeout in seconds
    pub download_timeout: u64,
    /// Number of concurrent downloads
    pub concurrent_downloads: usize,
    /// Custom environment variables
    pub env_vars: HashMap<String, String>,
    /// Whether to enable debug logging
    pub debug: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            minecraft_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".minecraft"),
            java_path: None,
            jvm_args: vec![
                "-XX:+UnlockExperimentalVMOptions".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-XX:G1NewSizePercent=20".to_string(),
                "-XX:G1ReservePercent=20".to_string(),
                "-XX:MaxGCPauseMillis=50".to_string(),
                "-XX:G1HeapRegionSize=32M".to_string(),
            ],
            game_args: Vec::new(),
            memory_min: 4096, // 4GB
            memory_max: 8192, // 8GB
            download_timeout: 300, // 5 minutes
            concurrent_downloads: 8,
            env_vars: HashMap::new(),
            debug: false,
        }
    }
}

impl LauncherConfig {
    /// Create a new launcher configuration with the specified Minecraft directory
    pub fn new(minecraft_dir: PathBuf) -> Self {
        Self {
            minecraft_dir,
            ..Default::default()
        }
    }

    /// Set the Java executable path
    pub fn with_java_path(mut self, java_path: PathBuf) -> Self {
        self.java_path = Some(java_path);
        self
    }

    /// Set memory allocation
    pub fn with_memory(mut self, min_mb: u32, max_mb: u32) -> Self {
        self.memory_min = min_mb;
        self.memory_max = max_mb;
        self
    }

    /// Add JVM arguments
    pub fn with_jvm_args(mut self, args: Vec<String>) -> Self {
        self.jvm_args.extend(args);
        self
    }

    /// Add game arguments
    pub fn with_game_args(mut self, args: Vec<String>) -> Self {
        self.game_args.extend(args);
        self
    }

    /// Enable debug logging
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// Set download configuration
    pub fn with_download_config(mut self, timeout: u64, concurrent: usize) -> Self {
        self.download_timeout = timeout;
        self.concurrent_downloads = concurrent;
        self
    }

    /// Add environment variable
    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }
}

/// Launch configuration for a specific Minecraft instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Minecraft version to launch
    pub version: String,
    /// Instance name (for organizing multiple installations)
    pub instance_name: String,
    /// Account to use for authentication
    pub account: Account,
    /// Mod loader configuration
    pub mod_loader: Option<ModLoaderConfig>,
    /// Custom mods directory
    pub mods_dir: Option<PathBuf>,
    /// Resource packs directory
    pub resource_packs_dir: Option<PathBuf>,
    /// Shader packs directory
    pub shader_packs_dir: Option<PathBuf>,
    /// World saves directory
    pub saves_dir: Option<PathBuf>,
    /// Custom game directory (overrides instance-based directory)
    pub custom_game_dir: Option<PathBuf>,
    /// Window configuration
    pub window_config: WindowConfig,
    /// Whether to download missing assets
    pub download_assets: bool,
    /// Whether to download missing libraries
    pub download_libraries: bool,
    /// Additional JVM arguments for this launch
    pub additional_jvm_args: Vec<String>,
    /// Additional game arguments for this launch
    pub additional_game_args: Vec<String>,
}

/// Mod loader configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoaderConfig {
    /// Type of mod loader
    pub loader_type: ModLoaderType,
    /// Loader version
    pub version: String,
    /// Whether to enable the loader
    pub enabled: bool,
}

/// Window configuration for Minecraft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Whether to start in fullscreen
    pub fullscreen: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fullscreen: false,
        }
    }
}

impl LaunchConfig {
    /// Create a new launch configuration
    pub fn new(version: String, instance_name: String, account: Account) -> Self {
        Self {
            version,
            instance_name,
            account,
            mod_loader: None,
            mods_dir: None,
            resource_packs_dir: None,
            shader_packs_dir: None,
            saves_dir: None,
            custom_game_dir: None,
            window_config: WindowConfig::default(),
            download_assets: true,
            download_libraries: true,
            additional_jvm_args: Vec::new(),
            additional_game_args: Vec::new(),
        }
    }

    /// Enable mod loader
    pub fn with_mod_loader(mut self, loader_type: ModLoaderType, version: String) -> Self {
        self.mod_loader = Some(ModLoaderConfig {
            loader_type,
            version,
            enabled: true,
        });
        self
    }

    /// Set custom directories
    pub fn with_custom_dirs(
        mut self,
        mods: Option<PathBuf>,
        resource_packs: Option<PathBuf>,
        shader_packs: Option<PathBuf>,
        saves: Option<PathBuf>,
    ) -> Self {
        self.mods_dir = mods;
        self.resource_packs_dir = resource_packs;
        self.shader_packs_dir = shader_packs;
        self.saves_dir = saves;
        self
    }

    /// Set window configuration
    pub fn with_window(mut self, width: u32, height: u32, fullscreen: bool) -> Self {
        self.window_config = WindowConfig {
            width,
            height,
            fullscreen,
        };
        self
    }

    /// Disable asset/library downloads
    pub fn without_downloads(mut self) -> Self {
        self.download_assets = false;
        self.download_libraries = false;
        self
    }

    /// Add additional arguments
    pub fn with_additional_args(mut self, jvm_args: Vec<String>, game_args: Vec<String>) -> Self {
        self.additional_jvm_args.extend(jvm_args);
        self.additional_game_args.extend(game_args);
        self
    }
}
