//! Error types for the Minecraft launcher library

use thiserror::Error;

/// Result type alias for launcher operations
pub type Result<T> = std::result::Result<T, LauncherError>;

/// Main error type for launcher operations
#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    #[error("Download error: {0}")]
    Download(String),
    #[error("File operation failed: {0}")]
    File(String),
    #[error("JSON parsing error: {0}")]
    Json(String),
    #[error("Minecraft launch failed: {0}")]
    Launch(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Java runtime error: {0}")]
    Java(String),
}

impl LauncherError {
    /// Create a new authentication error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        Self::Auth(msg.into())
    }

    /// Create a new version not found error
    pub fn version_not_found<S: Into<String>>(version: S) -> Self {
        Self::VersionNotFound(version.into())
    }

    /// Create a new download error
    pub fn download<S: Into<String>>(msg: S) -> Self {
        Self::Download(msg.into())
    }

    /// Create a new file error
    pub fn file<S: Into<String>>(msg: S) -> Self {
        Self::File(msg.into())
    }

    /// Create a new launch error
    pub fn launch<S: Into<String>>(msg: S) -> Self {
        Self::Launch(msg.into())
    }

    /// Create a new configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Download(msg.into())
    }

    /// Create a new JSON error
    pub fn json<S: Into<String>>(msg: S) -> Self {
        Self::Json(msg.into())
    }

    /// Create a new process error
    pub fn process<S: Into<String>>(msg: S) -> Self {
        Self::Launch(msg.into())
    }

    /// Create a new mod loader error
    pub fn mod_loader<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    pub fn zip<S: Into<String>>(msg: S) -> Self {
        LauncherError::Zip(zip::result::ZipError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            msg.into(),
        )))
    }

    pub fn java<S: Into<String>>(msg: S) -> Self {
        LauncherError::Java(msg.into())
    }
}
