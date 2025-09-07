//! Download example
//! 
//! This example demonstrates the download capabilities of the launcher,
//! including progress tracking and parallel downloads.

use minecraft_launcher_lib::{
    Launcher, LauncherConfig, VersionManager,
    init_logger,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    println!("📥 Download Example");
    println!("==================");

    // Set up directories
    let minecraft_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".minecraft");
    
    let config = LauncherConfig::new(minecraft_dir.clone())
        .with_download_config(300, 8) // 5 min timeout, 8 concurrent downloads
        .with_debug();

    println!("📁 Minecraft directory: {}", config.minecraft_dir.display());

    // Create launcher
    let launcher = Launcher::new(config).await?;
    println!("✅ Launcher initialized");

    // Set up version manager
    let cache_dir = minecraft_dir.join("cache");
    let version_manager = VersionManager::new(cache_dir)?;

    // Example 1: Download version manifest
    println!("\n📋 Example 1: Downloading Version Manifest");
    println!("==========================================");

    let manifest = version_manager.fetch_version_manifest().await?;
    println!("✅ Version manifest downloaded");
    println!("   Latest release: {}", manifest.latest.release);
    println!("   Latest snapshot: {}", manifest.latest.snapshot);
    println!("   Total versions: {}", manifest.versions.len());

    // Show some version information
    println!("\n📦 Recent versions:");
    for version in manifest.versions.iter().take(10) {
        println!("   {} ({}) - {}", version.id, version.version_type, version.release_time.format("%Y-%m-%d"));
    }

    // Example 2: Download specific version info
    println!("\n🎯 Example 2: Downloading Specific Version Info");
    println!("===============================================");

    let version_id = "1.21.4";
    println!("🔍 Finding version {}...", version_id);

    let version_entry = version_manager.find_version(version_id).await?;
    println!("✅ Found version: {}", version_entry.id);

    println!("📥 Downloading version info...");
    let version_info = version_manager.fetch_version_info(&version_entry).await?;
    println!("✅ Version info downloaded");
    println!("   Main class: {}", version_info.main_class);
    println!("   Asset index: {}", version_info.asset_index.id);
    println!("   Libraries: {}", version_info.libraries.len());

    // Java version requirements
    if let Some(java_version) = &version_info.java_version {
        println!("   Java version required: {}", java_version.major_version);
        println!("   Java component: {}", java_version.component);
    }

    // Example 3: Simulate download with progress tracking
    println!("\n📊 Example 3: Download Progress Tracking");
    println!("========================================");

    // Count total download size
    let mut total_size = 0u64;
    let mut download_count = 0;

    // Client jar
    total_size += version_info.downloads.client.size;
    download_count += 1;

    // Libraries
    for library in &version_info.libraries {
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                total_size += artifact.size;
                download_count += 1;
            }
            if let Some(classifiers) = &downloads.classifiers {
                for (_, download_info) in classifiers {
                    total_size += download_info.size;
                    download_count += 1;
                }
            }
        }
    }

    // Asset index
    total_size += version_info.asset_index.size;
    download_count += 1;

    println!("📈 Download statistics:");
    println!("   Files to download: {}", download_count);
    println!("   Total size: {}", format_size(total_size));
    println!("   Estimated time: {}", estimate_download_time(total_size));

    // Example 4: Show download progress simulation
    println!("\n⏳ Example 4: Simulated Download Progress");
    println!("========================================");

    let progress = Arc::new(Mutex::new((0u64, total_size)));
    let progress_clone = progress.clone();

    // Simulate download progress
    let progress_task = tokio::spawn(async move {
        for i in 0..=100 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            
            let downloaded = (total_size * i) / 100;
            
            {
                let mut p = progress_clone.lock().unwrap();
                p.0 = downloaded;
            }
            
            print_progress(downloaded, total_size);
        }
    });

    progress_task.await?;
    println!("\n✅ Simulated download completed!");

    // Example 5: File verification example
    println!("\n🔍 Example 5: File Verification");
    println!("===============================");

    println!("📄 Client jar verification:");
    println!("   Expected SHA1: {}", version_info.downloads.client.sha1);
    println!("   File size: {}", format_size(version_info.downloads.client.size));
    println!("   Download URL: {}", version_info.downloads.client.url);

    // Show some library verification info
    println!("\n📚 Library verification examples:");
    for (i, library) in version_info.libraries.iter().take(5).enumerate() {
        println!("   {}. {}", i + 1, library.name);
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                println!("      SHA1: {}", artifact.sha1);
                println!("      Size: {}", format_size(artifact.size));
            }
        }
    }

    // Example 6: Download optimization tips
    println!("\n💡 Download Optimization Tips");
    println!("=============================");
    println!("1. 🚀 Use concurrent downloads (current: 8 parallel)");
    println!("2. ✅ Enable file verification to ensure integrity");
    println!("3. 💾 Cache downloads to avoid re-downloading");
    println!("4. 🔄 Use resume capability for large files");
    println!("5. 🌐 Choose appropriate timeout values");
    println!("6. 📊 Track progress for better user experience");

    // Example 7: Asset download simulation
    println!("\n🎨 Example 7: Asset Download Information");
    println!("=======================================");

    println!("🖼️  Asset index: {}", version_info.asset_index.id);
    println!("📥 Asset index URL: {}", version_info.asset_index.url);
    println!("📊 Asset index size: {}", format_size(version_info.asset_index.size));

    if let Some(total_size) = version_info.asset_index.total_size {
        println!("📦 Total asset size: {}", format_size(total_size));
        println!("⏱️  Estimated download time: {}", estimate_download_time(total_size));
    }

    println!("\n✅ Download example completed!");

    Ok(())
}

fn print_progress(downloaded: u64, total: u64) {
    let percentage = if total > 0 {
        (downloaded * 100) / total
    } else {
        0
    };

    let bar_length = 30;
    let filled_length = (percentage * bar_length as u64) / 100;
    
    let mut bar = String::new();
    for i in 0..bar_length {
        if i < filled_length as usize {
            bar.push('█');
        } else {
            bar.push('░');
        }
    }

    print!("\r[{}] {}% ({}/{})", 
           bar, 
           percentage, 
           format_size(downloaded), 
           format_size(total));
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if size == 0 {
        return "0 B".to_string();
    }

    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn estimate_download_time(size: u64) -> String {
    // Assume average download speed of 10 MB/s
    let speed_mbps = 10.0;
    let size_mb = size as f64 / (1024.0 * 1024.0);
    let time_seconds = size_mb / speed_mbps;

    if time_seconds < 60.0 {
        format!("{:.0}s", time_seconds)
    } else if time_seconds < 3600.0 {
        format!("{:.1}m", time_seconds / 60.0)
    } else {
        format!("{:.1}h", time_seconds / 3600.0)
    }
}

// Additional utility functions for download management

/// Calculate the total download size for a version
pub async fn calculate_version_download_size(
    version_info: &minecraft_launcher_lib::VersionInfo,
) -> u64 {
    let mut total_size = 0u64;

    // Client jar
    total_size += version_info.downloads.client.size;

    // Libraries
    for library in &version_info.libraries {
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                total_size += artifact.size;
            }
            if let Some(classifiers) = &downloads.classifiers {
                for (_, download_info) in classifiers {
                    total_size += download_info.size;
                }
            }
        }
    }

    // Asset index
    total_size += version_info.asset_index.size;

    // Assets (if total size is available)
    if let Some(asset_total) = version_info.asset_index.total_size {
        total_size += asset_total;
    }

    total_size
}

/// Get download statistics for multiple versions
pub async fn get_download_statistics(
    version_manager: &VersionManager,
    version_ids: &[&str],
) -> Result<Vec<(String, u64, usize)>, Box<dyn std::error::Error>> {
    let mut statistics = Vec::new();

    for version_id in version_ids {
        let version_entry = version_manager.find_version(version_id).await?;
        let version_info = version_manager.fetch_version_info(&version_entry).await?;
        
        let total_size = calculate_version_download_size(&version_info).await;
        let file_count = version_info.libraries.len() + 1; // +1 for client jar

        statistics.push((version_id.to_string(), total_size, file_count));
    }

    Ok(statistics)
}
