# Minecraft Launcher Library (Rust)

A comprehensive Minecraft launcher library written in Rust, providing functionality for authentication, version management, mod loader support, and game launching.

## Features

- 🔐 **Microsoft Authentication** - Complete OAuth2 flow with token management
- 🎮 **Minecraft Version Management** - Download and manage any Minecraft version
- 🔧 **Mod Loader Support** - Forge, Fabric, Quilt, NeoForge, and Legacy Fabric
- 📥 **Parallel Downloads** - Efficient downloading with progress tracking
- ⚡ **Async/Await** - Modern Rust async programming
- 🛠️ **Java Management** - Automatic Java detection and version matching
- 📊 **Progress Tracking** - Real-time download and launch progress
- 🔒 **File Verification** - SHA1 hash verification for all downloads
- 🏗️ **Process Management** - Launch, monitor, and control Minecraft processes
- 📝 **Comprehensive Logging** - Detailed logging for debugging

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
minecraft-launcher-lib = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use minecraft_launcher_lib::{
    Launcher, LauncherConfig, AuthenticatorConfig,
    init_logger,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logger();

    // Create launcher configuration
    let config = LauncherConfig::new(PathBuf::from(".minecraft"))
        .with_memory(4096, 8192); // 4GB min, 8GB max

    // Create launcher instance
    let mut launcher = Launcher::new(config).await?;

    // Set up authentication
    let auth_config = AuthenticatorConfig::new("your-client-id".to_string());
    let account = launcher.authenticate_with_code(auth_config, "auth_code".to_string()).await?;

    // Create launch configuration
    let launch_config = launcher.create_launch_config("1.21.4", &account).await?;

    // Launch Minecraft
    let process = launcher.launch(launch_config).await?;
    println!("Minecraft launched with PID: {}", process.get_pid().await?);

    Ok(())
}
```

## Examples

The library includes comprehensive examples demonstrating various features:

### Running Examples

```bash
# Basic launcher usage
cargo run --example basic_launch

# Authentication flow
cargo run --example auth_example

# Mod loader support
cargo run --example mod_loader_example

# Download management
cargo run --example download_example
```

### Example Descriptions

- **`basic_launch.rs`** - Complete authentication and launch flow
- **`auth_example.rs`** - Microsoft authentication and token management
- **`mod_loader_example.rs`** - Using different mod loaders (Forge, Fabric, etc.)
- **`download_example.rs`** - Download progress tracking and file management

## Authentication

### Microsoft OAuth Setup

1. Register your application in Azure AD:
   - Go to [Azure Portal](https://portal.azure.com)
   - Navigate to "Azure Active Directory" > "App registrations"
   - Create a new registration
   - Add redirect URI: `http://localhost:8080/auth/callback`
   - Note your Application (client) ID

2. Use your client ID:

```rust
let auth_config = AuthenticatorConfig::new("your-azure-client-id".to_string())
    .with_redirect_uri("http://localhost:8080/auth/callback".to_string());

let authenticator = Authenticator::new(auth_config)?;
let auth_url = authenticator.get_auth_url()?;

// Direct user to auth_url, then extract authorization code
let account = authenticator.authenticate_with_code(auth_code).await?;
```

### Token Management

```rust
// Check if token is still valid
if authenticator.is_token_valid(&account) {
    println!("Token is valid");
} else {
    // Refresh the token
    let refreshed_account = authenticator.refresh_account(&account).await?;
}
```

## Mod Loaders

### Supported Mod Loaders

- **Forge** - The most popular mod loader
- **Fabric** - Lightweight, modern mod loader  
- **Quilt** - Enhanced fork of Fabric
- **NeoForge** - Modern fork of Forge
- **Legacy Fabric** - Fabric for older Minecraft versions

### Using Mod Loaders

```rust
use minecraft_launcher_lib::ModLoaderType;

// Create launch config with Fabric
let mut launch_config = launcher.create_launch_config("1.21.4", &account).await?;
launch_config = launch_config.with_mod_loader(
    ModLoaderType::Fabric,
    "0.16.10".to_string(),
);

// Launch with custom mod directories
launch_config = launch_config.with_custom_dirs(
    Some(PathBuf::from("mods")),
    Some(PathBuf::from("resourcepacks")),
    Some(PathBuf::from("shaderpacks")),
    Some(PathBuf::from("saves")),
);
```

## Configuration

### Launcher Configuration

```rust
let config = LauncherConfig::new(minecraft_dir)
    .with_java_path(PathBuf::from("/usr/lib/jvm/java-21"))
    .with_memory(6144, 12288) // 6GB min, 12GB max
    .with_jvm_args(vec![
        "-XX:+UseG1GC".to_string(),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
    ])
    .with_download_config(300, 8) // 5min timeout, 8 concurrent downloads
    .with_debug();
```

### Launch Configuration

```rust
let launch_config = LaunchConfig::new(version, instance_name, account)
    .with_mod_loader(ModLoaderType::Forge, "47.3.0".to_string())
    .with_window(1920, 1080, false) // width, height, fullscreen
    .with_additional_args(
        vec!["-Dfml.ignoreInvalidMinecraftCertificates=true".to_string()], // JVM args
        vec!["--server".to_string(), "my-server.com".to_string()], // Game args
    );
```

## Download Management

### Progress Tracking

```rust
use minecraft_launcher_lib::Downloader;

let downloader = Downloader::new(8, 300)?; // 8 concurrent, 5min timeout

downloader.download_file_with_progress(
    "https://example.com/file.jar",
    &PathBuf::from("file.jar"),
    Some("expected_sha1_hash"),
    |downloaded, total| {
        let percentage = (downloaded * 100) / total;
        println!("Progress: {}% ({}/{})", percentage, downloaded, total);
    },
).await?;
```

### Batch Downloads

```rust
let downloads = vec![
    ("https://example.com/file1.jar".to_string(), PathBuf::from("file1.jar"), "hash1".to_string()),
    ("https://example.com/file2.jar".to_string(), PathBuf::from("file2.jar"), "hash2".to_string()),
];

downloader.download_files(downloads).await?;
```

## Process Management

### Managing Minecraft Processes

```rust
// Launch Minecraft
let process = launcher.launch(launch_config).await?;

// Check if running
if process.is_running().await {
    println!("Minecraft is running with PID: {}", process.get_pid().await?);
}

// Read logs
let logs = process.read_logs().await?;
println!("Latest logs:\n{}", logs);

// Kill the process
process.kill().await?;
```

### Multiple Instances

```rust
// Get all active processes
let active_processes = launcher.get_active_processes().await;
println!("Active processes: {}", active_processes.len());

// Kill all processes
let killed_count = launcher.kill_all().await?;
println!("Killed {} processes", killed_count);
```

## Java Management

The library automatically detects and manages Java installations:

```rust
use minecraft_launcher_lib::JavaFinder;

let java_finder = JavaFinder::new();

// Find Java for specific version
let java_path = java_finder.find_java(21).await?; // Java 21

// List all Java installations
let installations = java_finder.list_java_installations().await;
for (version, path) in installations {
    println!("Java {}: {}", version, path.display());
}
```

## Error Handling

The library provides comprehensive error types:

```rust
use minecraft_launcher_lib::{LauncherError, Result};

match launcher.launch(launch_config).await {
    Ok(process) => println!("Launch successful"),
    Err(LauncherError::AuthenticationError(msg)) => println!("Auth failed: {}", msg),
    Err(LauncherError::VersionNotFound(version)) => println!("Version {} not found", version),
    Err(LauncherError::DownloadError(msg)) => println!("Download failed: {}", msg),
    Err(LauncherError::LaunchError(msg)) => println!("Launch failed: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Logging

Enable logging to see detailed information:

```rust
use minecraft_launcher_lib::init_logger;

// Initialize with default settings
init_logger();

// Or use env_logger directly for custom configuration
std::env::set_var("RUST_LOG", "minecraft_launcher_lib=debug");
env_logger::init();
```

## Feature Flags

Enable specific mod loaders:

```toml
[dependencies]
minecraft-launcher-lib = { version = "0.1.0", features = ["forge", "fabric"] }
```

Available features:
- `default` - All mod loaders
- `forge` - Forge support
- `fabric` - Fabric support  
- `quilt` - Quilt support
- `neoforge` - NeoForge support
- `legacy-fabric` - Legacy Fabric support

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-repo/minecraft-launcher-lib
cd minecraft-launcher-lib

# Run tests
cargo test

# Run examples
cargo run --example basic_launch

# Build documentation
cargo doc --open
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the original Node.js launcher library
- Microsoft for providing the authentication APIs
- Mojang/Microsoft for Minecraft and its APIs
- The Rust community for excellent async libraries

## Roadmap

- [ ] Complete mod loader API implementations
- [ ] Resource pack and shader pack management
- [ ] Server list management
- [ ] Crash report analysis
- [ ] Performance profiling
- [ ] GUI framework integration helpers
- [ ] Plugin system for custom mod loaders
