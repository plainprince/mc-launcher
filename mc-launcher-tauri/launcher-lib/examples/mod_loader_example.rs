//! Mod loader example
//! 
//! This example demonstrates how to launch Minecraft with different mod loaders
//! like Forge, Fabric, Quilt, etc.

use minecraft_launcher_lib::{
    Launcher, LauncherConfig, AuthenticatorConfig, Account,
    ModLoaderType, VersionManager,
    init_logger,
};
use std::path::PathBuf;
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    println!("🔧 Mod Loader Example");
    println!("=====================");

    // Set up launcher
    let minecraft_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".minecraft");
    
    let config = LauncherConfig::new(minecraft_dir)
        .with_memory(6144, 12288) // More memory for modded Minecraft
        .with_debug();

    let mut launcher = Launcher::new(config).await?;
    println!("✅ Launcher initialized");

    // Load account (simplified - in practice you'd handle authentication)
    let account = load_account().await?;
    println!("👤 Loaded account: {}", account.name);

    // Set up version manager to check available mod loaders
    let cache_dir = launcher.get_config().minecraft_dir.join("cache");
    let version_manager = VersionManager::new(cache_dir)?;

    // Example 1: Launch with Fabric
    println!("\n🧵 Example 1: Minecraft with Fabric");
    println!("===================================");
    
    let minecraft_version = "1.21.4";
    println!("🔍 Checking available Fabric versions for {}...", minecraft_version);
    
    let fabric_versions = version_manager.get_mod_loader_versions(
        ModLoaderType::Fabric,
        minecraft_version,
    ).await?;

    if !fabric_versions.is_empty() {
        let latest_fabric = &fabric_versions[0];
        println!("✅ Latest Fabric version: {}", latest_fabric.version);
        
        let mut launch_config = launcher.create_launch_config(minecraft_version, &account).await?;
        launch_config = launch_config.with_mod_loader(
            ModLoaderType::Fabric,
            latest_fabric.version.clone(),
        );

        println!("🚀 Launching Minecraft {} with Fabric {}...", minecraft_version, latest_fabric.version);
        
        // Note: In a complete implementation, this would actually launch
        println!("   (Launch simulation - mod loader setup would happen here)");
    } else {
        println!("⚠️  No Fabric versions found for {}", minecraft_version);
    }

    // Example 2: Launch with Forge
    println!("\n⚒️ Example 2: Minecraft with Forge");
    println!("==================================");
    
    let forge_versions = version_manager.get_mod_loader_versions(
        ModLoaderType::Forge,
        minecraft_version,
    ).await?;

    if !forge_versions.is_empty() {
        let latest_forge = &forge_versions[0];
        println!("✅ Latest Forge version: {}", latest_forge.version);
        
        let mut launch_config = launcher.create_launch_config(minecraft_version, &account).await?;
        launch_config = launch_config.with_mod_loader(
            ModLoaderType::Forge,
            latest_forge.version.clone(),
        );

        println!("🚀 Launching Minecraft {} with Forge {}...", minecraft_version, latest_forge.version);
        println!("   (Launch simulation - mod loader setup would happen here)");
    } else {
        println!("⚠️  No Forge versions found for {}", minecraft_version);
    }

    // Example 3: Custom mod configuration
    println!("\n📦 Example 3: Custom Mod Configuration");
    println!("=====================================");
    
    let mods_dir = launcher.get_config().minecraft_dir.join("mods");
    let resource_packs_dir = launcher.get_config().minecraft_dir.join("resourcepacks");
    let shader_packs_dir = launcher.get_config().minecraft_dir.join("shaderpacks");
    let saves_dir = launcher.get_config().minecraft_dir.join("saves");

    let mut custom_launch_config = launcher.create_launch_config(minecraft_version, &account).await?;
    custom_launch_config = custom_launch_config
        .with_mod_loader(ModLoaderType::Fabric, "0.16.10".to_string())
        .with_custom_dirs(
            Some(mods_dir.clone()),
            Some(resource_packs_dir),
            Some(shader_packs_dir),
            Some(saves_dir),
        )
        .with_window(1920, 1080, false)
        .with_additional_args(
            vec![
                "-Dfml.ignoreInvalidMinecraftCertificates=true".to_string(),
                "-Dfml.ignorePatchDiscrepancies=true".to_string(),
            ],
            vec!["--quickPlayMultiplayer".to_string(), "my-server.com".to_string()],
        );

    println!("📁 Custom mods directory: {}", mods_dir.display());
    println!("🖼️  Window size: 1920x1080");
    println!("⚡ Additional JVM args: 2");
    println!("🎮 Additional game args: 2");

    // Example 4: Show all supported mod loaders
    println!("\n🔧 Supported Mod Loaders");
    println!("========================");
    
    let mod_loaders = [
        ModLoaderType::Forge,
        ModLoaderType::Fabric,
        ModLoaderType::Quilt,
        ModLoaderType::NeoForge,
        ModLoaderType::LegacyFabric,
    ];

    for loader_type in &mod_loaders {
        println!("🔌 {}", loader_type);
        
        // In a complete implementation, you would check available versions
        match loader_type {
            ModLoaderType::Forge => println!("   - Most popular mod loader, supports many mods"),
            ModLoaderType::Fabric => println!("   - Lightweight, modern mod loader"),
            ModLoaderType::Quilt => println!("   - Fork of Fabric with additional features"),
            ModLoaderType::NeoForge => println!("   - Modern fork of Forge"),
            ModLoaderType::LegacyFabric => println!("   - Fabric for older Minecraft versions"),
        }
    }

    // Example 5: Mod management tips
    println!("\n💡 Mod Management Tips");
    println!("=====================");
    println!("1. 📦 Download mods to: {}", mods_dir.display());
    println!("2. 🔄 Different mod loaders require different mod files");
    println!("3. 📋 Check mod compatibility with your Minecraft version");
    println!("4. 🧪 Test mods in a separate instance first");
    println!("5. 💾 Keep backups of your worlds before adding new mods");
    
    println!("\n✅ Mod loader example completed!");

    Ok(())
}

// Helper function to load account (simplified)
async fn load_account() -> Result<Account, Box<dyn std::error::Error>> {
    // In practice, this would load from a file or perform authentication
    // For this example, we'll create a dummy account
    use minecraft_launcher_lib::{ProfileInfo, SkinInfo, CapeInfo};
    use chrono::Utc;

    Ok(Account {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        name: "ExamplePlayer".to_string(),
        access_token: "example_token".to_string(),
        refresh_token: "example_refresh_token".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
        account_type: "msa".to_string(),
        profile: ProfileInfo {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            name: "ExamplePlayer".to_string(),
            skins: vec![
                SkinInfo {
                    id: "skin1".to_string(),
                    state: "ACTIVE".to_string(),
                    url: "https://textures.minecraft.net/texture/example".to_string(),
                    variant: "classic".to_string(),
                }
            ],
            capes: vec![],
        },
    })
}

// Additional helper functions for mod management

/// List installed mods in a directory
pub async fn list_installed_mods(mods_dir: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut mods = Vec::new();
    
    if mods_dir.exists() {
        let mut entries = tokio::fs::read_dir(mods_dir).await?;
        
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "jar" {
                    if let Some(filename) = path.file_name() {
                        mods.push(filename.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    mods.sort();
    Ok(mods)
}

/// Check mod loader compatibility
pub fn check_mod_compatibility(mod_loader: &ModLoaderType, minecraft_version: &str) -> bool {
    // Simplified compatibility check
    match mod_loader {
        ModLoaderType::Forge => {
            // Forge supports most versions from 1.2.5 onwards
            true
        }
        ModLoaderType::Fabric => {
            // Fabric supports versions from 1.14 onwards primarily
            let version_parts: Vec<&str> = minecraft_version.split('.').collect();
            if version_parts.len() >= 2 {
                if let (Ok(major), Ok(minor)) = (version_parts[0].parse::<i32>(), version_parts[1].parse::<i32>()) {
                    major > 1 || (major == 1 && minor >= 14)
                } else {
                    false
                }
            } else {
                false
            }
        }
        ModLoaderType::Quilt => {
            // Quilt is based on Fabric, similar compatibility
            let version_parts: Vec<&str> = minecraft_version.split('.').collect();
            if version_parts.len() >= 2 {
                if let (Ok(major), Ok(minor)) = (version_parts[0].parse::<i32>(), version_parts[1].parse::<i32>()) {
                    major > 1 || (major == 1 && minor >= 17)
                } else {
                    false
                }
            } else {
                false
            }
        }
        ModLoaderType::NeoForge => {
            // NeoForge is newer, supports 1.20.1+
            let version_parts: Vec<&str> = minecraft_version.split('.').collect();
            if version_parts.len() >= 3 {
                if let (Ok(major), Ok(minor), Ok(patch)) = (
                    version_parts[0].parse::<i32>(),
                    version_parts[1].parse::<i32>(),
                    version_parts[2].parse::<i32>()
                ) {
                    major > 1 || (major == 1 && minor > 20) || (major == 1 && minor == 20 && patch >= 1)
                } else {
                    false
                }
            } else {
                false
            }
        }
        ModLoaderType::LegacyFabric => {
            // Legacy Fabric for older versions
            let version_parts: Vec<&str> = minecraft_version.split('.').collect();
            if version_parts.len() >= 2 {
                if let (Ok(major), Ok(minor)) = (version_parts[0].parse::<i32>(), version_parts[1].parse::<i32>()) {
                    major == 1 && minor < 14
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}
