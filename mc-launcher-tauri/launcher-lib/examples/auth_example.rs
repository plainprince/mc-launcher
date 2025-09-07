//! Authentication example
//! 
//! This example demonstrates how to authenticate with Microsoft and manage account tokens.

use minecraft_launcher_lib::{
    AuthenticatorConfig, Authenticator, Account,
    init_logger,
};
use serde_json;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    println!("🔐 Microsoft Authentication Example");
    println!("==================================");

    // Set up authentication configuration
    let auth_config = AuthenticatorConfig::new(
        "your-azure-client-id".to_string() // Replace with your actual client ID
    ).with_redirect_uri("http://localhost:8080/auth/callback".to_string());

    let authenticator = Authenticator::new(auth_config)?;

    // Check if we have a saved account
    let account_file = "account.json";
    let account = if let Ok(account_data) = fs::read_to_string(account_file) {
        let mut account: Account = serde_json::from_str(&account_data)?;
        
        println!("📱 Found saved account: {}", account.name);
        
        // Check if token is still valid
        if authenticator.is_token_valid(&account) {
            println!("✅ Token is still valid");
            account
        } else {
            println!("🔄 Token expired, refreshing...");
            
            // Try to refresh the token
            match authenticator.refresh_account(&account).await {
                Ok(refreshed_account) => {
                    println!("✅ Token refreshed successfully");
                    
                    // Save the refreshed account
                    let account_json = serde_json::to_string_pretty(&refreshed_account)?;
                    fs::write(account_file, account_json)?;
                    
                    refreshed_account
                }
                Err(e) => {
                    println!("❌ Token refresh failed: {}", e);
                    println!("🔄 Starting new authentication flow...");
                    
                    perform_new_authentication(&authenticator, account_file).await?
                }
            }
        }
    } else {
        println!("📝 No saved account found, starting authentication...");
        perform_new_authentication(&authenticator, account_file).await?
    };

    // Display account information
    println!("\n👤 Account Information:");
    println!("   Name: {}", account.name);
    println!("   UUID: {}", account.uuid);
    println!("   Type: {}", account.account_type);
    println!("   Expires: {}", account.expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    // Display profile information
    println!("\n🎮 Profile Information:");
    println!("   Profile ID: {}", account.profile.id);
    println!("   Profile Name: {}", account.profile.name);
    
    if !account.profile.skins.is_empty() {
        println!("   Skins:");
        for skin in &account.profile.skins {
            println!("     - {} ({}): {}", skin.variant, skin.state, skin.url);
        }
    }
    
    if !account.profile.capes.is_empty() {
        println!("   Capes:");
        for cape in &account.profile.capes {
            println!("     - {} ({}): {}", cape.alias, cape.state, cape.url);
        }
    }

    println!("\n✅ Authentication example completed successfully!");
    
    Ok(())
}

async fn perform_new_authentication(
    authenticator: &Authenticator,
    account_file: &str,
) -> Result<Account, Box<dyn std::error::Error>> {
    // Get the authentication URL
    let auth_url = authenticator.get_auth_url()?;
    
    println!("🌐 Please visit the following URL to authenticate:");
    println!("{}", auth_url);
    println!();
    println!("After authentication, you'll be redirected to a URL like:");
    println!("http://localhost:8080/auth/callback?code=AUTHORIZATION_CODE&state=...");
    println!();
    
    // In a real application, you would:
    // 1. Open the URL in a browser (or embedded webview)
    // 2. Start a local server to capture the redirect
    // 3. Extract the authorization code automatically
    
    // For this example, we'll ask the user to manually enter the code
    println!("📝 Please copy the authorization code from the callback URL:");
    print!("Authorization code: ");
    
    let mut auth_code = String::new();
    std::io::stdin().read_line(&mut auth_code)?;
    let auth_code = auth_code.trim().to_string();
    
    if auth_code.is_empty() {
        return Err("No authorization code provided".into());
    }
    
    println!("🔄 Completing authentication with code...");
    
    // Complete the authentication
    let account = authenticator.authenticate_with_code(auth_code).await?;
    
    println!("✅ Authentication successful!");
    
    // Save the account for future use
    let account_json = serde_json::to_string_pretty(&account)?;
    fs::write(account_file, account_json)?;
    println!("💾 Account saved to {}", account_file);
    
    Ok(account)
}

// Additional helper functions that could be useful

/// Check account validity without refreshing
pub async fn check_account_status(account_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let account_data = fs::read_to_string(account_file)?;
    let account: Account = serde_json::from_str(&account_data)?;
    
    let auth_config = AuthenticatorConfig::new("your-client-id".to_string());
    let authenticator = Authenticator::new(auth_config)?;
    
    if authenticator.is_token_valid(&account) {
        println!("✅ Account token is valid until {}", account.expires_at);
    } else {
        println!("❌ Account token has expired");
    }
    
    Ok(())
}

/// Logout (delete saved account)
pub fn logout(account_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if std::path::Path::new(account_file).exists() {
        fs::remove_file(account_file)?;
        println!("🚪 Logged out - account file deleted");
    } else {
        println!("ℹ️  No account file found");
    }
    
    Ok(())
}
