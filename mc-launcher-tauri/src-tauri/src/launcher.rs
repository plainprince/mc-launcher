//! Tauri launcher backend integration

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, Emitter};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use minecraft_launcher_lib::{
    Launcher as MLLauncher,
    LauncherConfig,
    AuthenticatorConfig,
    Authenticator,
    Account,
    MinecraftProcess,
    ModLoaderType,
    LauncherError,
    version::{VersionManager, VersionManifest},
    java::JavaManager,
};

/// Tauri launcher state
pub struct LauncherState {
    pub launcher: Arc<Mutex<Option<MLLauncher>>>,
    pub authenticator: Arc<Mutex<Option<Authenticator>>>,
    pub active_processes: Arc<Mutex<HashMap<String, MinecraftProcess>>>,
    pub current_account: Arc<Mutex<Option<Account>>>,
}

impl LauncherState {
    pub fn new() -> Self {
        Self {
            launcher: Arc::new(Mutex::new(None)),
            authenticator: Arc::new(Mutex::new(None)),
            active_processes: Arc::new(Mutex::new(HashMap::new())),
            current_account: Arc::new(Mutex::new(None)),
        }
    }
}

// Tauri command structs
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeLauncherRequest {
    pub minecraft_dir: String,
    pub memory_min: Option<u32>,
    pub memory_max: Option<u32>,
    pub java_path: Option<String>,
    pub concurrent_downloads: Option<usize>,
    pub download_timeout: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LaunchRequest {
    pub version: String,
    pub instance_name: Option<String>,
    pub mod_loader: Option<ModLoaderRequest>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub fullscreen: Option<bool>,
    pub additional_jvm_args: Option<Vec<String>>,
    pub additional_game_args: Option<Vec<String>>,
    pub account: Account,
    pub java_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModLoaderRequest {
    pub loader_type: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> LauncherResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaRuntimeResponse {
    pub path: String,
}


// Tauri commands

#[tauri::command]
pub async fn initialize_launcher(
    request: InitializeLauncherRequest,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<String>, String> {
    log::info!("Initializing launcher with directory: {}", request.minecraft_dir);
    
    let minecraft_dir = PathBuf::from(&request.minecraft_dir);
    
    let mut config = LauncherConfig::new(minecraft_dir);
    
    if let (Some(min), Some(max)) = (request.memory_min, request.memory_max) {
        config = config.with_memory(min, max);
    }
    
    if let Some(java_path) = request.java_path {
        config = config.with_java_path(PathBuf::from(java_path));
    }
    
    if let (Some(concurrent), Some(timeout)) = (request.concurrent_downloads, request.download_timeout) {
        config = config.with_download_config(timeout, concurrent);
    }
    
    config = config.with_debug();
    
    match MLLauncher::new(config).await {
        Ok(launcher) => {
            let mut launcher_guard = state.launcher.lock().await;
            *launcher_guard = Some(launcher);
            
            log::info!("Launcher initialized successfully");
            Ok(LauncherResponse::success("Launcher initialized successfully".to_string()))
        }
        Err(e) => {
            log::error!("Failed to initialize launcher: {}", e);
            Ok(LauncherResponse::error(format!("Failed to initialize launcher: {}", e)))
        }
    }
}

#[tauri::command]
pub async fn setup_authenticator(
    auth_config: AuthConfig,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<String>, String> {
    log::info!("Setting up authenticator with client ID: {}", auth_config.client_id);
    
    let mut config = AuthenticatorConfig::new(auth_config.client_id);
    
    if let Some(redirect_uri) = auth_config.redirect_uri {
        config = config.with_redirect_uri(redirect_uri);
    }
    
    match Authenticator::new(config) {
        Ok(authenticator) => {
            let mut auth_guard = state.authenticator.lock().await;
            *auth_guard = Some(authenticator);
            
            log::info!("Authenticator setup successfully");
            Ok(LauncherResponse::success("Authenticator setup successfully".to_string()))
        }
        Err(e) => {
            log::error!("Failed to setup authenticator: {}", e);
            Ok(LauncherResponse::error(format!("Failed to setup authenticator: {}", e)))
        }
    }
}

#[tauri::command]
pub async fn get_auth_url(
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<String>, String> {
    let auth_guard = state.authenticator.lock().await;
    
    if let Some(authenticator) = auth_guard.as_ref() {
        match authenticator.get_auth_url() {
            Ok(url) => {
                log::info!("Generated auth URL");
                Ok(LauncherResponse::success(url))
            }
            Err(e) => {
                log::error!("Failed to get auth URL: {}", e);
                Ok(LauncherResponse::error(format!("Failed to get auth URL: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Authenticator not initialized".to_string()))
    }
}

#[tauri::command]
pub async fn authenticate_with_code(
    auth_code: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<Account>, String> {
    log::info!("Authenticating with code");
    
    let auth_guard = state.authenticator.lock().await;
    
    if let Some(authenticator) = auth_guard.as_ref() {
        match authenticator.authenticate_with_code(auth_code).await {
            Ok(account) => {
                log::info!("Authentication successful for user: {}", account.name);
                
                // Store the account
                let mut account_guard = state.current_account.lock().await;
                *account_guard = Some(account.clone());
                
                Ok(LauncherResponse::success(account))
            }
            Err(e) => {
                log::error!("Authentication failed: {}", e);
                Ok(LauncherResponse::error(format!("Authentication failed: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Authenticator not initialized".to_string()))
    }
}

#[tauri::command]
pub async fn start_device_code_flow(
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<minecraft_launcher_lib::auth::DeviceCodeResponse>, String> {
    log::info!("Starting device code flow");
    
    let auth_guard = state.authenticator.lock().await;
    
    if let Some(authenticator) = auth_guard.as_ref() {
        match authenticator.start_device_code_flow().await {
            Ok(device_response) => {
                log::info!("Device code flow started successfully");
                Ok(LauncherResponse::success(device_response))
            }
            Err(e) => {
                log::error!("Device code flow failed: {}", e);
                Ok(LauncherResponse::error(format!("Device code flow failed: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Authenticator not initialized".to_string()))
    }
}

#[tauri::command]
pub async fn poll_device_code(
    device_code: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<Account>, String> {
    log::info!("Polling device code");
    
    let auth_guard = state.authenticator.lock().await;
    
    if let Some(authenticator) = auth_guard.as_ref() {
        match authenticator.poll_device_code(&device_code).await {
            Ok(account) => {
                log::info!("Device code authentication successful for user: {}", account.name);
                
                // Store the account
                let mut account_guard = state.current_account.lock().await;
                *account_guard = Some(account.clone());
                
                Ok(LauncherResponse::success(account))
            }
            Err(e) => {
                let error_msg = e.to_string();
                log::debug!("Device code polling result: {}", error_msg);
                
                // Check for specific OAuth errors that should be passed through
                if error_msg.contains("authorization_pending") || 
                   error_msg.contains("slow_down") || 
                   error_msg.contains("expired_token") {
                    Ok(LauncherResponse::error(error_msg))
                } else {
                    log::error!("Device code polling failed: {}", e);
                    Ok(LauncherResponse::error(format!("Device code polling failed: {}", e)))
                }
            }
        }
    } else {
        Ok(LauncherResponse::error("Authenticator not initialized".to_string()))
    }
}

#[tauri::command]
pub async fn refresh_account(
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<Account>, String> {
    log::info!("Refreshing account");
    
    let auth_guard = state.authenticator.lock().await;
    let account_guard = state.current_account.lock().await;
    
    if let (Some(authenticator), Some(account)) = (auth_guard.as_ref(), account_guard.as_ref()) {
        match authenticator.refresh_account(account).await {
            Ok(refreshed_account) => {
                log::info!("Account refreshed successfully");
                
                // Update stored account
                drop(account_guard);
                let mut account_guard = state.current_account.lock().await;
                *account_guard = Some(refreshed_account.clone());
                
                Ok(LauncherResponse::success(refreshed_account))
            }
            Err(e) => {
                log::error!("Failed to refresh account: {}", e);
                Ok(LauncherResponse::error(format!("Failed to refresh account: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Authenticator or account not available".to_string()))
    }
}

#[tauri::command]
pub async fn get_version_manifest(
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<VersionManifest>, String> {
    log::info!("Fetching version manifest");
    
    let launcher_guard = state.launcher.lock().await;
    
    if let Some(launcher) = launcher_guard.as_ref() {
        let cache_dir = launcher.get_config().minecraft_dir.join("cache");
        
        match VersionManager::new(cache_dir) {
            Ok(version_manager) => {
                match version_manager.fetch_version_manifest().await {
                    Ok(manifest) => {
                        log::info!("Version manifest fetched successfully");
                        Ok(LauncherResponse::success(manifest))
                    }
                    Err(e) => {
                        log::error!("Failed to fetch version manifest: {}", e);
                        Ok(LauncherResponse::error(format!("Failed to fetch version manifest: {}", e)))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create version manager: {}", e);
                Ok(LauncherResponse::error(format!("Failed to create version manager: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Launcher not initialized".to_string()))
    }
}

#[tauri::command]
pub async fn get_java_runtime(
    version: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<JavaRuntimeResponse>, String> {
    log::info!("Checking for Java runtime for version: {}", version);
    let launcher_guard = state.launcher.lock().await;

    if let Some(launcher) = launcher_guard.as_ref() {
        let minecraft_dir = launcher.get_config().minecraft_dir.clone();
        let java_manager = JavaManager::new(minecraft_dir.join("runtime"));

        match java_manager.get_java_runtime(&version).await {
            Ok(Some(java_path)) => {
                log::info!("Found existing Java runtime at: {:?}", java_path);
                Ok(LauncherResponse::success(JavaRuntimeResponse {
                    path: java_path.to_string_lossy().to_string(),
                }))
            }
            Ok(None) => {
                log::info!("No suitable Java runtime found, downloading Java {}...", version);
                match java_manager.download_java_runtime(&version).await {
                    Ok(java_path) => {
                        log::info!("Successfully downloaded Java runtime to: {:?}", java_path);
                        Ok(LauncherResponse::success(JavaRuntimeResponse {
                            path: java_path.to_string_lossy().to_string(),
                        }))
                    }
                    Err(e) => {
                        log::error!("Failed to download Java runtime: {}", e);
                        Ok(LauncherResponse::error(format!(
                            "Failed to download Java runtime: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get Java runtime: {}", e);
                Ok(LauncherResponse::error(format!(
                    "Failed to get Java runtime: {}",
                    e
                )))
            }
        }
    } else {
        Ok(LauncherResponse::error(
            "Launcher not initialized".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn launch_minecraft(
    request: LaunchRequest,
    state: State<'_, LauncherState>,
    app: tauri::AppHandle,
) -> Result<LauncherResponse<String>, String> {
    log::info!("Launching Minecraft version: {} for user: {}", request.version, request.account.name);
    
    // Emit log to frontend
    let _ = app.emit("launcher-log", serde_json::json!({
        "level": "info",
        "message": format!("🚀 Launching Minecraft {} for {}...", request.version, request.account.name)
    }));
    
    let mut launcher_guard = state.launcher.lock().await;
    
    if let Some(launcher) = launcher_guard.as_mut() {
        // Use the account from the request instead of the state
        match launcher.create_launch_config(&request.version, &request.account).await {
            Ok(mut launch_config) => {
                // Apply custom configuration
                if let Some(instance_name) = request.instance_name {
                    launch_config.instance_name = instance_name;
                }
                
                if let Some(mod_loader_req) = request.mod_loader {
                    if let Ok(loader_type) = parse_mod_loader_type(&mod_loader_req.loader_type) {
                        launch_config = launch_config.with_mod_loader(loader_type, mod_loader_req.version);
                    }
                }
                
                if let (Some(width), Some(height)) = (request.window_width, request.window_height) {
                    let fullscreen = request.fullscreen.unwrap_or(false);
                    launch_config = launch_config.with_window(width, height, fullscreen);
                }
                
                if let Some(jvm_args) = request.additional_jvm_args {
                    if let Some(game_args) = request.additional_game_args {
                        launch_config = launch_config.with_additional_args(jvm_args, game_args);
                    }
                }
                
                // Set the Java path in the launcher config if provided
                if let Some(java_path) = request.java_path {
                    log::info!("Using custom Java path: {}", java_path);
                    // Update the launcher's Java path configuration
                    let mut config = launcher.get_config().clone();
                    config.java_path = Some(PathBuf::from(java_path));
                    launcher.update_config(config);
                }
                
                // Launch Minecraft
                match launcher.launch(launch_config).await {
                    Ok(process) => {
                        let pid = process.get_pid().await.unwrap_or(0);
                        let process_id = uuid::Uuid::new_v4().to_string();
                        
                        // Store the process
                        let mut processes_guard = state.active_processes.lock().await;
                        processes_guard.insert(process_id.clone(), process);
                        
                        log::info!("Minecraft launched successfully with PID: {} (Internal ID: {})", pid, process_id);
                        
                        // Emit success log to frontend
                        let _ = app.emit("launcher-log", serde_json::json!({
                            "level": "success",
                            "message": format!("✅ Minecraft {} launched successfully (PID: {})", request.version, pid)
                        }));
                        
                        Ok(LauncherResponse::success(process_id))
                    }
                    Err(e) => {
                        log::error!("Failed to launch Minecraft: {}", e);
                        
                        // Emit error log to frontend
                        let _ = app.emit("launcher-log", serde_json::json!({
                            "level": "error",
                            "message": format!("❌ Failed to launch Minecraft: {}", e)
                        }));
                        
                        Ok(LauncherResponse::error(format!("Failed to launch Minecraft: {}", e)))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create launch config: {}", e);
                Ok(LauncherResponse::error(format!("Configuration error: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Launcher not available".to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessStatusResponse {
    pub is_running: bool,
    pub pid: Option<u32>,
}

#[tauri::command]
pub async fn get_process_status(
    process_id: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<ProcessStatusResponse>, String> {
    let processes_guard = state.active_processes.lock().await;
    
    if let Some(process) = processes_guard.get(&process_id) {
        let is_running = process.is_running().await;
        let pid = if is_running {
            process.get_pid().await.ok()
        } else {
            None
        };
        
        Ok(LauncherResponse::success(ProcessStatusResponse {
            is_running,
            pid,
        }))
    } else {
        // Process not found, assume it's not running
        Ok(LauncherResponse::success(ProcessStatusResponse {
            is_running: false,
            pid: None,
        }))
    }
}

#[tauri::command]
pub async fn kill_minecraft(
    process_id: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<String>, String> {
    log::info!("Killing Minecraft process: {}", process_id);
    
    let mut processes_guard = state.active_processes.lock().await;
    
    if let Some(process) = processes_guard.remove(&process_id) {
        match process.kill().await {
            Ok(()) => {
                log::info!("Minecraft process killed successfully");
                Ok(LauncherResponse::success("Process killed successfully".to_string()))
            }
            Err(e) => {
                log::error!("Failed to kill process: {}", e);
                Ok(LauncherResponse::error(format!("Failed to kill process: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Process not found".to_string()))
    }
}

#[tauri::command]
pub async fn get_minecraft_logs(
    process_id: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<String>, String> {
    let processes_guard = state.active_processes.lock().await;
    
    if let Some(process) = processes_guard.get(&process_id) {
        match process.read_logs().await {
            Ok(logs) => Ok(LauncherResponse::success(logs)),
            Err(e) => {
                log::error!("Failed to read logs: {}", e);
                Ok(LauncherResponse::error(format!("Failed to read logs: {}", e)))
            }
        }
    } else {
        Ok(LauncherResponse::error("Process not found".to_string()))
    }
}

#[tauri::command]
pub async fn list_active_processes(
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<Vec<String>>, String> {
    let processes_guard = state.active_processes.lock().await;
    let process_ids: Vec<String> = processes_guard.keys().cloned().collect();
    
    Ok(LauncherResponse::success(process_ids))
}

#[tauri::command]
pub async fn is_process_running(
    process_id: String,
    state: State<'_, LauncherState>,
) -> Result<LauncherResponse<bool>, String> {
    let processes_guard = state.active_processes.lock().await;
    
    if let Some(process) = processes_guard.get(&process_id) {
        let is_running = process.is_running().await;
        Ok(LauncherResponse::success(is_running))
    } else {
        Ok(LauncherResponse::success(false))
    }
}

#[tauri::command]
pub async fn get_home_directory() -> Result<LauncherResponse<String>, String> {
    match dirs::home_dir() {
        Some(home) => {
            let home_str = home.to_string_lossy().to_string();
            Ok(LauncherResponse::success(home_str))
        }
        None => Ok(LauncherResponse::error("Could not determine home directory".to_string()))
    }
}

// Helper functions

fn parse_mod_loader_type(loader_type: &str) -> Result<ModLoaderType, LauncherError> {
    match loader_type.to_lowercase().as_str() {
        "forge" => Ok(ModLoaderType::Forge),
        "fabric" => Ok(ModLoaderType::Fabric),
        "quilt" => Ok(ModLoaderType::Quilt),
        "neoforge" => Ok(ModLoaderType::NeoForge),
        "legacy-fabric" => Ok(ModLoaderType::LegacyFabric),
        _ => Err(LauncherError::config(format!("Unknown mod loader type: {}", loader_type))),
    }
}
