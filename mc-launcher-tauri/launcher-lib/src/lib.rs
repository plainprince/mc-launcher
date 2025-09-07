//! # Minecraft Launcher Library
//!
//! A comprehensive Minecraft launcher library written in Rust, providing functionality for:
//! - Microsoft authentication
//! - Minecraft version management and launching
//! - Mod loader support (Forge, Fabric, Quilt, NeoForge)
//! - Asset and library downloading
//! - Progress tracking and event handling
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use minecraft_launcher_lib::{Launcher, LauncherConfig, AuthenticatorConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = LauncherConfig::new("./minecraft".into());
//!     let mut launcher = Launcher::new(config).await?;
//!     
//!     // Authenticate with Microsoft
//!     let auth_config = AuthenticatorConfig::new("your-client-id".to_string());
//!     let account = launcher.authenticate(auth_config).await?;
//!     
//!     // Launch Minecraft
//!     let launch_config = launcher.create_launch_config("1.21.4", &account)?;
//!     let process = launcher.launch(launch_config).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod config;
pub mod downloader;
pub mod error;
pub mod launcher;
pub mod minecraft;
pub mod utils;
pub mod version;
pub mod java;

// Re-export main types
pub use auth::{Authenticator, AuthenticatorConfig, Account};
pub use config::{LauncherConfig, LaunchConfig};
pub use error::{LauncherError, Result};
pub use launcher::Launcher;
pub use minecraft::{MinecraftProcess, ProcessStatus};
pub use version::{VersionManifest, VersionInfo, ModLoader, ModLoaderType};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the logger with default settings
pub fn init_logger() {
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
