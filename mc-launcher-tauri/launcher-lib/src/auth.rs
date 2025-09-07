//! Microsoft authentication for Minecraft


use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::{LauncherError, Result};

/// Microsoft account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account UUID
    pub uuid: String,
    /// Minecraft profile name (username)
    pub name: String,
    /// Access token for Minecraft services
    pub access_token: String,
    /// Refresh token for token renewal
    pub refresh_token: String,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
    /// Account type (typically "msa" for Microsoft)
    pub account_type: String,
    /// Additional profile information
    pub profile: ProfileInfo,
}

/// Minecraft profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Profile skins
    pub skins: Vec<SkinInfo>,
    /// Profile capes
    pub capes: Vec<CapeInfo>,
}

/// Skin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    /// Skin ID
    pub id: String,
    /// Skin state (ACTIVE, etc.)
    pub state: String,
    /// Skin URL
    pub url: String,
    /// Skin variant (classic, slim)
    pub variant: String,
}

/// Cape information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapeInfo {
    /// Cape ID
    pub id: String,
    /// Cape state
    pub state: String,
    /// Cape URL
    pub url: String,
    /// Cape alias
    pub alias: String,
}

/// Configuration for Microsoft authentication
#[derive(Debug, Clone)]
pub struct AuthenticatorConfig {
    /// Microsoft Azure application client ID
    pub client_id: String,
    /// Redirect URI for OAuth flow
    pub redirect_uri: String,
    /// OAuth scopes to request
    pub scopes: Vec<String>,
    /// Custom user agent for HTTP requests
    pub user_agent: Option<String>,
    /// Timeout for authentication requests (seconds)
    pub timeout: u64,
}

impl Default for AuthenticatorConfig {
    fn default() -> Self {
        Self {
            client_id: "00000000-0000-0000-0000-000000000000".to_string(), // Placeholder
            redirect_uri: "http://localhost:8080/auth/callback".to_string(),
            scopes: vec![
                "XboxLive.signin".to_string(),
                "offline_access".to_string(),
            ],
            user_agent: Some(format!("MinecraftLauncher/{}", crate::VERSION)),
            timeout: 300,
        }
    }
}

impl AuthenticatorConfig {
    /// Create a new authenticator configuration with the specified client ID
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }

    /// Set the redirect URI
    pub fn with_redirect_uri(mut self, redirect_uri: String) -> Self {
        self.redirect_uri = redirect_uri;
        self
    }

    /// Set custom scopes
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Set custom user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Microsoft authenticator for Minecraft
pub struct Authenticator {
    config: AuthenticatorConfig,
    client: reqwest::Client,
}

impl Authenticator {
    /// Create a new authenticator with the given configuration
    pub fn new(config: AuthenticatorConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        if let Some(user_agent) = &config.user_agent {
            headers.insert(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_str(user_agent)
                    .map_err(|e| LauncherError::config(format!("Invalid user agent: {}", e)))?
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| LauncherError::auth(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Start the OAuth authentication flow
    /// Returns the authorization URL that the user should visit
    pub fn get_auth_url(&self) -> Result<String> {
        // Use the exact same URL format as the working JavaScript launcher
        let auth_url = format!(
            "https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=XboxLive.signin%20offline_access&cobrandid=8058f65d-ce06-4c30-9559-473c9275a65d&prompt=select_account",
            self.config.client_id
        );

        Ok(auth_url)
    }

    /// Complete the OAuth flow with the authorization code
    pub async fn authenticate_with_code(&self, auth_code: String) -> Result<Account> {
        // Step 1: Exchange authorization code for access token
        let token_response = self.exchange_code_for_token(auth_code).await?;
        
        // Step 2: Authenticate with Xbox Live
        let xbox_token = self.authenticate_xbox_live(&token_response.access_token).await?;
        
        // Step 3: Authenticate with XSTS (Xbox Security Token Service)
        let xsts_token = self.authenticate_xsts(&xbox_token).await?;
        
        // Step 4: Authenticate with Minecraft
        let minecraft_token = self.authenticate_minecraft(&xsts_token).await?;
        
        // Step 5: Get profile information
        let profile = self.get_minecraft_profile(&minecraft_token).await?;
        
        // Step 6: Check game ownership
        self.check_game_ownership(&minecraft_token).await?;

        Ok(Account {
            uuid: profile.id.clone(),
            name: profile.name.clone(),
            access_token: minecraft_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            expires_at: Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64),
            account_type: "msa".to_string(),
            profile,
        })
    }

    /// Refresh an existing account's tokens
    pub async fn refresh_account(&self, account: &Account) -> Result<Account> {
        if account.refresh_token.is_empty() {
            return Err(LauncherError::auth("No refresh token available"));
        }

        // Refresh the Microsoft access token
        let token_response = self.refresh_microsoft_token(&account.refresh_token).await?;
        
        // Re-authenticate with the new token
        let xbox_token = self.authenticate_xbox_live(&token_response.access_token).await?;
        let xsts_token = self.authenticate_xsts(&xbox_token).await?;
        let minecraft_token = self.authenticate_minecraft(&xsts_token).await?;
        let profile = self.get_minecraft_profile(&minecraft_token).await?;

        Ok(Account {
            uuid: profile.id.clone(),
            name: profile.name.clone(),
            access_token: minecraft_token,
            refresh_token: token_response.refresh_token.unwrap_or_else(|| account.refresh_token.clone()),
            expires_at: Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64),
            account_type: account.account_type.clone(),
            profile,
        })
    }

    /// Check if an account's token is still valid
    pub fn is_token_valid(&self, account: &Account) -> bool {
        account.expires_at > Utc::now() + chrono::Duration::minutes(5) // 5-minute buffer
    }

    /// Start device code flow for authentication
    /// Note: Microsoft Live.com doesn't support standard device code flow, so we'll simulate it
    /// by generating a device code locally and using the standard authorization flow
    pub async fn start_device_code_flow(&self) -> Result<DeviceCodeResponse> {
        // Since Live.com doesn't support device code flow, we'll create a simulated response
        // that directs users to the standard OAuth flow
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Generate a simple user code (like "ABCD-EFGH")
        let user_code = format!("{:04X}-{:04X}", 
            (timestamp % 65536) as u16, 
            ((timestamp / 65536) % 65536) as u16
        );
        
        // Create a device code (we'll use this to track the session)
        let device_code = format!("device_{}", timestamp);
        
        // Use the same URL format as your working JavaScript launcher
        let verification_uri = format!(
            "https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=XboxLive.signin%20offline_access&prompt=select_account",
            self.config.client_id
        );

        Ok(DeviceCodeResponse {
            device_code,
            user_code,
            verification_uri,
            expires_in: 900, // 15 minutes
            interval: 5,     // Poll every 5 seconds
            message: Some("Please visit the URL and sign in with your Microsoft account".to_string()),
        })
    }

    /// Poll for device code completion
    pub async fn poll_device_code(&self, device_code: &str) -> Result<Account> {
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", self.config.client_id.as_str()),
            ("device_code", device_code),
        ];

        let response = self.client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Device code poll request failed: {}", e)))?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse token response: {}", e)))?;

        // If we get here, authentication was successful - continue with normal flow
        self.complete_authentication_with_token(token_response).await
    }

    /// Complete authentication with a token response (shared by both flows)
    async fn complete_authentication_with_token(&self, token_response: TokenResponse) -> Result<Account> {
        // Step 2: Authenticate with Xbox Live
        let xbox_token = self.authenticate_xbox_live(&token_response.access_token).await?;
        
        // Step 3: Authenticate with XSTS (Xbox Security Token Service)
        let xsts_token = self.authenticate_xsts(&xbox_token).await?;
        
        // Step 4: Authenticate with Minecraft
        let minecraft_token = self.authenticate_minecraft(&xsts_token).await?;
        
        // Step 5: Get profile information
        let profile = self.get_minecraft_profile(&minecraft_token).await?;
        
        // Step 6: Check game ownership
        self.check_game_ownership(&minecraft_token).await?;

        Ok(Account {
            uuid: profile.id.clone(),
            name: profile.name.clone(),
            access_token: minecraft_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            expires_at: Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64),
            account_type: "msa".to_string(),
            profile,
        })
    }

    // Private helper methods for the authentication flow
    
    async fn exchange_code_for_token(&self, auth_code: String) -> Result<TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("code", &auth_code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        // Use the same token endpoint as the working JavaScript launcher
        let response = self.client
            .post("https://login.live.com/oauth20_token.srf")
            .form(&params)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Token exchange request failed: {}", e)))?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response)
    }

    async fn refresh_microsoft_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("scope", &self.config.scopes.join(" ")),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self.client
            .post("https://login.live.com/oauth20_token.srf")
            .form(&params)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Token refresh request failed: {}", e)))?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse refresh response: {}", e)))?;

        Ok(token_response)
    }

    async fn authenticate_xbox_live(&self, access_token: &str) -> Result<String> {
        let payload = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let response = self.client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Xbox Live authentication failed: {}", e)))?;

        let xbox_response: XboxLiveResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse Xbox Live response: {}", e)))?;

        Ok(xbox_response.token)
    }

    async fn authenticate_xsts(&self, xbox_token: &str) -> Result<XstsResponse> {
        let payload = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let response = self.client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("XSTS authentication failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LauncherError::auth(format!("XSTS authentication failed with status {}: {}", status, error_text)));
        }

        let xsts_response: XstsResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse XSTS response: {}", e)))?;

        Ok(xsts_response)
    }

    async fn authenticate_minecraft(&self, xsts_response: &XstsResponse) -> Result<String> {
        let user_hash = xsts_response.display_claims.xui[0].uhs.clone();
        let payload = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_response.token)
        });

        let response = self.client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Minecraft authentication failed: {}", e)))?;

        let minecraft_response: MinecraftAuthResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse Minecraft auth response: {}", e)))?;

        Ok(minecraft_response.access_token)
    }

    async fn get_minecraft_profile(&self, access_token: &str) -> Result<ProfileInfo> {
        let response = self.client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Profile request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LauncherError::auth(format!("Profile request failed with status {}: {}", status, error_text)));
        }

        let profile: ProfileInfo = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse profile response: {}", e)))?;

        Ok(profile)
    }

    async fn check_game_ownership(&self, access_token: &str) -> Result<()> {
        let response = self.client
            .get("https://api.minecraftservices.com/entitlements/mcstore")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| LauncherError::auth(format!("Ownership check failed: {}", e)))?;

        let ownership: OwnershipResponse = response
            .json()
            .await
            .map_err(|e| LauncherError::auth(format!("Failed to parse ownership response: {}", e)))?;

        if ownership.items.is_empty() {
            return Err(LauncherError::auth("No Minecraft ownership found for this account"));
        }

        Ok(())
    }
}

// Response structures for API calls

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    expires_in: u64,
    #[allow(dead_code)]
    scope: String,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    #[serde(alias = "verification_url")]
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XboxLiveResponse {
    #[serde(rename = "Token")]
    token: String,
}

#[derive(Debug, Deserialize)]
struct XstsResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
struct DisplayClaims {
    xui: Vec<UserInfo>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    uhs: String,
}

#[derive(Debug, Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct OwnershipResponse {
    items: Vec<OwnershipItem>,
}

#[derive(Debug, Deserialize)]
struct OwnershipItem {
    #[allow(dead_code)]
    name: String,
}

// Add urlencoding dependency
