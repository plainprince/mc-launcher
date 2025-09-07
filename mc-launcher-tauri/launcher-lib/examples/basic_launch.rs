//! Basic launcher example
//! 
//! This example demonstrates the most basic usage of the Minecraft launcher library.
//! It authenticates a user and launches Minecraft.

use minecraft_launcher_lib::{
    Launcher, LauncherConfig, AuthenticatorConfig,
    init_logger,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logger();

    println!("🚀 Basic Minecraft Launcher Example");
    println!("==================================");

    // 1. Create launcher configuration
    let minecraft_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".minecraft");
    
    let config = LauncherConfig::new(minecraft_dir)
        .with_memory(4096, 8192) // 4GB min, 8GB max
        .with_debug();

    println!("📁 Minecraft directory: {}", config.minecraft_dir.display());

    // 2. Create launcher instance
    let mut launcher = Launcher::new(config).await?;
    println!("✅ Launcher initialized");

    // 3. Set up authentication
    // NOTE: You need to register an application with Microsoft Azure AD
    // and get a client ID. This is a placeholder.
    let auth_config = AuthenticatorConfig::new(
        "your-azure-client-id".to_string() // Replace with your actual client ID
    );

    println!("🔐 Starting authentication...");
    
    // In a real application, you would handle the OAuth flow properly.
    // For this example, we'll show how the authentication would work:
    match launcher.authenticate(auth_config.clone()).await {
        Ok(account) => {
            println!("✅ Authentication successful!");
            println!("   Player: {}", account.name);
            println!("   UUID: {}", account.uuid);

            // 4. Create launch configuration
            let launch_config = launcher.create_launch_config("1.21.4", &account).await?;
            println!("📋 Launch configuration created for version {}", launch_config.version);

            // 5. Launch Minecraft
            println!("🎮 Launching Minecraft...");
            let process = launcher.launch(launch_config).await?;
            
            let pid = process.get_pid().await?;
            println!("✅ Minecraft launched successfully! PID: {}", pid);

            // 6. Wait for a bit, then show some process information
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            if process.is_running().await {
                println!("🟢 Minecraft is running");
                
                // Read logs if available
                match process.read_logs().await {
                    Ok(logs) => {
                        let lines: Vec<&str> = logs.lines().collect();
                        if lines.len() > 10 {
                            println!("📄 Last 10 lines of logs:");
                            for line in lines.iter().rev().take(10).rev() {
                                println!("   {}", line);
                            }
                        } else {
                            println!("📄 Logs: {}", logs);
                        }
                    }
                    Err(e) => println!("⚠️  Could not read logs: {}", e),
                }
            } else {
                println!("🔴 Minecraft has already stopped");
            }

            // 7. Example: Wait for user input to kill the process
            println!("\nPress Enter to kill Minecraft...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            println!("🛑 Killing Minecraft...");
            process.kill().await?;
            println!("✅ Minecraft stopped");
        }
        Err(e) => {
            if e.to_string().contains("Please visit this URL") {
                println!("📱 Manual authentication required:");
                println!("{}", e);
                println!("\n💡 To complete this example:");
                println!("   1. Visit the URL above");
                println!("   2. Complete the Microsoft OAuth flow");
                println!("   3. Extract the authorization code from the callback URL");
                println!("   4. Use launcher.authenticate_with_code() method");
            } else {
                println!("❌ Authentication failed: {}", e);
            }
        }
    }

    Ok(())
}
