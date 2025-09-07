//! Main launcher implementation

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{
    auth::{Authenticator, AuthenticatorConfig, Account},
    config::{LauncherConfig, LaunchConfig},
    downloader::Downloader,
    error::{LauncherError, Result},
    minecraft::{MinecraftProcess, ProcessStatus},
    version::{VersionManager, VersionInfo},
    utils::JavaFinder,
};

/// Main launcher instance
pub struct Launcher {
    config: LauncherConfig,
    version_manager: VersionManager,
    downloader: Downloader,
    java_finder: JavaFinder,
    active_processes: Arc<Mutex<Vec<MinecraftProcess>>>,
}

impl Launcher {
    /// Create a new launcher instance
    pub async fn new(config: LauncherConfig) -> Result<Self> {
        // Ensure minecraft directory exists
        tokio::fs::create_dir_all(&config.minecraft_dir)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to create minecraft directory: {}", e)))?;

        let cache_dir = config.minecraft_dir.join("cache");
        tokio::fs::create_dir_all(&cache_dir)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to create cache directory: {}", e)))?;

        let version_manager = VersionManager::new(cache_dir.clone())?;
        let downloader = Downloader::new(config.concurrent_downloads, config.download_timeout)?;
        let java_finder = JavaFinder::new();

        Ok(Self {
            config,
            version_manager,
            downloader,
            java_finder,
            active_processes: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create an authenticator with the given configuration
    pub fn create_authenticator(&self, auth_config: AuthenticatorConfig) -> Result<Authenticator> {
        Authenticator::new(auth_config)
    }

    /// Authenticate using the built-in authenticator
    pub async fn authenticate(&mut self, auth_config: AuthenticatorConfig) -> Result<Account> {
        let authenticator = self.create_authenticator(auth_config)?;
        let auth_url = authenticator.get_auth_url()?;
        
        // In a real implementation, you would:
        // 1. Open the auth URL in a browser or embedded webview
        // 2. Start a local server to capture the redirect
        // 3. Extract the authorization code from the callback
        // 4. Complete the authentication flow
        
        // For now, return an error indicating manual intervention is needed
        Err(LauncherError::auth(format!(
            "Please visit this URL to authenticate: {}\nThen extract the authorization code and use authenticate_with_code()",
            auth_url
        )))
    }

    /// Complete authentication with an authorization code
    pub async fn authenticate_with_code(&mut self, auth_config: AuthenticatorConfig, auth_code: String) -> Result<Account> {
        let authenticator = self.create_authenticator(auth_config)?;
        authenticator.authenticate_with_code(auth_code).await
    }

    /// Refresh an existing account
    pub async fn refresh_account(&mut self, auth_config: AuthenticatorConfig, account: &Account) -> Result<Account> {
        let authenticator = self.create_authenticator(auth_config)?;
        authenticator.refresh_account(account).await
    }

    /// Create a launch configuration for a specific version
    pub async fn create_launch_config(&mut self, version: &str, account: &Account) -> Result<LaunchConfig> {
        // Validate that the version exists
        let _version_entry = self.version_manager.find_version(version).await?;
        
        let launch_config = LaunchConfig::new(
            version.to_string(),
            format!("instance-{}", version), // Default instance name
            account.clone(),
        );

        Ok(launch_config)
    }

    /// Launch Minecraft with the given configuration
    pub async fn launch(&mut self, launch_config: LaunchConfig) -> Result<MinecraftProcess> {
        log::info!("Starting Minecraft launch for version {}", launch_config.version);

        // 1. Get version information
        let version_entry = self.version_manager.find_version(&launch_config.version).await?;
        let version_info = self.version_manager.fetch_version_info(&version_entry).await?;

        // 2. Set up directories
        let instance_dir = self.get_instance_dir(&launch_config.instance_name);
        self.setup_instance_directories(&instance_dir).await?;

        // 3. Download required files
        if launch_config.download_libraries {
            self.download_libraries(&version_info, &instance_dir).await?;
        }
        
        if launch_config.download_assets {
            self.download_assets(&version_info, &instance_dir).await?;
        }

        // 4. Setup mod loader if specified
        if let Some(mod_loader_config) = &launch_config.mod_loader {
            self.setup_mod_loader(mod_loader_config, &version_info, &instance_dir).await?;
        }

        // 5. Find Java executable
        let java_path = self.get_java_path(&version_info).await?;

        // 6. Build launch arguments
        let launch_args = self.build_launch_arguments(&launch_config, &version_info, &instance_dir, &java_path)?;

        // 7. Start the process
        let process = MinecraftProcess::new(
            java_path,
            launch_args,
            instance_dir,
            launch_config.account.clone(),
        ).await?;

        // 8. Track the process
        {
            let mut processes = self.active_processes.lock().await;
            processes.push(process.clone());
        }

        log::info!("Minecraft launched successfully with PID {}", process.get_pid().await?);
        Ok(process)
    }

    /// Get all active Minecraft processes
    pub async fn get_active_processes(&self) -> Vec<MinecraftProcess> {
        let mut processes = self.active_processes.lock().await;
        
        // Remove finished processes
        processes.retain(|process| {
            match process.get_status() {
                ProcessStatus::Running => true,
                _ => false,
            }
        });

        processes.clone()
    }

    /// Kill all active Minecraft processes
    pub async fn kill_all(&mut self) -> Result<usize> {
        let processes = {
            let mut processes = self.active_processes.lock().await;
            let current_processes = processes.clone();
            processes.clear();
            current_processes
        };

        let mut killed = 0;
        for process in processes {
            if process.kill().await.is_ok() {
                killed += 1;
            }
        }

        Ok(killed)
    }

    /// Get launcher configuration
    pub fn get_config(&self) -> &LauncherConfig {
        &self.config
    }

    /// Update launcher configuration
    pub fn update_config(&mut self, config: LauncherConfig) {
        self.config = config;
    }

    // Private helper methods

    fn get_instance_dir(&self, instance_name: &str) -> PathBuf {
        self.config.minecraft_dir.join("instances").join(instance_name)
    }

    async fn setup_instance_directories(&self, instance_dir: &PathBuf) -> Result<()> {
        let directories = [
            instance_dir.clone(),
            instance_dir.join("libraries"),
            instance_dir.join("assets"),
            instance_dir.join("versions"),
            instance_dir.join("mods"),
            instance_dir.join("resourcepacks"),
            instance_dir.join("shaderpacks"),
            instance_dir.join("saves"),
            instance_dir.join("logs"),
            instance_dir.join("crash-reports"),
        ];

        for dir in &directories {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| LauncherError::file(format!("Failed to create directory {}: {}", dir.display(), e)))?;
        }

        Ok(())
    }

    async fn download_libraries(&mut self, version_info: &VersionInfo, instance_dir: &PathBuf) -> Result<()> {
        log::info!("Downloading libraries for version {}", version_info.id);
        
        let libraries_dir = instance_dir.join("libraries");
        let mut download_tasks = Vec::new();

        // First, add the main Minecraft client JAR to download tasks
        let client_download = &version_info.downloads.client;
        let versions_dir = instance_dir.join("versions").join(&version_info.id);
        let client_jar_path = versions_dir.join(format!("{}.jar", version_info.id));
        
        // Create versions directory if it doesn't exist
        if let Some(parent) = client_jar_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LauncherError::file(format!("Failed to create versions directory: {}", e)))?;
            }
        }
        
        log::info!("Adding main client JAR to download: {}", client_jar_path.display());
        download_tasks.push((client_download.url.clone(), client_jar_path, client_download.sha1.clone()));

        for library in &version_info.libraries {
            // Check if library applies to current OS
            if let Some(rules) = &library.rules {
                if !self.evaluate_rules(rules) {
                    continue;
                }
            }

            // LWJGL libraries will work via Rosetta 2 emulation on ARM64

            if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let library_path = self.get_library_path(&library.name, &libraries_dir);
                    download_tasks.push((artifact.url.clone(), library_path, artifact.sha1.clone()));
                }

                // Handle native libraries
                if let Some(classifiers) = &downloads.classifiers {
                    for (classifier, download_info) in classifiers {
                        if self.is_native_for_current_os(classifier) {
                            let native_path = self.get_native_path(&library.name, classifier, &libraries_dir);
                            download_tasks.push((download_info.url.clone(), native_path, download_info.sha1.clone()));
                        }
                    }
                }
            }
        }

        // Download all libraries and the main client JAR
        self.downloader.download_files(download_tasks).await?;
        
        // ARM compatibility is handled via JVM flags and Rosetta 2
        
        // Extract native libraries after downloading
        self.extract_native_libraries(version_info, instance_dir).await?;
        
        log::info!("Libraries and main client JAR downloaded successfully");
        Ok(())
    }

    async fn download_assets(&mut self, version_info: &VersionInfo, instance_dir: &PathBuf) -> Result<()> {
        log::info!("Downloading assets for version {}", version_info.id);
        
        // Download asset index
        let assets_dir = instance_dir.join("assets");
        let asset_index_path = assets_dir.join("indexes").join(format!("{}.json", version_info.asset_index.id));
        
        tokio::fs::create_dir_all(asset_index_path.parent().unwrap())
            .await
            .map_err(|e| LauncherError::file(format!("Failed to create asset index directory: {}", e)))?;

        self.downloader.download_file(
            &version_info.asset_index.url,
            &asset_index_path,
            Some(&version_info.asset_index.sha1),
        ).await?;

        // Parse asset index and download assets
        let asset_index_content = tokio::fs::read_to_string(&asset_index_path)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to read asset index: {}", e)))?;

        let asset_index: serde_json::Value = serde_json::from_str(&asset_index_content)
            .map_err(|e| LauncherError::json(format!("Failed to parse asset index: {}", e)))?;

        if let Some(objects) = asset_index.get("objects").and_then(|o| o.as_object()) {
            let mut download_tasks = Vec::new();
            
            for (_asset_name, asset_info) in objects {
                if let (Some(hash), Some(_size)) = (
                    asset_info.get("hash").and_then(|h| h.as_str()),
                    asset_info.get("size").and_then(|s| s.as_u64()),
                ) {
                    let asset_url = format!("https://resources.download.minecraft.net/{}/{}", &hash[0..2], hash);
                    let asset_path = assets_dir.join("objects").join(&hash[0..2]).join(hash);
                    
                    download_tasks.push((asset_url, asset_path, hash.to_string()));
                }
            }

            self.downloader.download_files(download_tasks).await?;
        }

        log::info!("Assets downloaded successfully");
        Ok(())
    }

    async fn setup_mod_loader(
        &mut self,
        _mod_loader_config: &crate::config::ModLoaderConfig,
        _version_info: &VersionInfo,
        _instance_dir: &PathBuf,
    ) -> Result<()> {
        // TODO: Implement mod loader setup
        log::info!("Mod loader setup not yet implemented");
        Ok(())
    }

    async fn get_java_path(&self, version_info: &VersionInfo) -> Result<PathBuf> {
        if let Some(java_path) = &self.config.java_path {
            return Ok(java_path.clone());
        }

        // Determine required Java version
        let required_java_version = version_info
            .java_version
            .as_ref()
            .map(|jv| jv.major_version)
            .unwrap_or(8); // Default to Java 8 for older versions

        self.java_finder.find_java(required_java_version).await
    }

    fn build_launch_arguments(
        &self,
        launch_config: &LaunchConfig,
        version_info: &VersionInfo,
        instance_dir: &PathBuf,
        _java_path: &PathBuf,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Add JVM arguments
        args.extend(self.config.jvm_args.clone());
        args.extend(launch_config.additional_jvm_args.clone());

        // Add memory settings
        args.push(format!("-Xms{}m", self.config.memory_min));
        args.push(format!("-Xmx{}m", self.config.memory_max));

        // Add native library path arguments
        let natives_dir = instance_dir.join("versions").join(&version_info.id).join("natives");
        if natives_dir.exists() {
            let natives_path = natives_dir.to_string_lossy();
            args.push(format!("-Djava.library.path={}", natives_path));
            args.push(format!("-Djna.tmpdir={}", natives_path));
            args.push(format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={}", natives_path));
            args.push(format!("-Dio.netty.native.workdir={}", natives_path));
        }

        // ARM64 compatibility is handled by Rosetta 2 emulation at the process level

        // Add library path
        let libraries_dir = instance_dir.join("libraries");
        let classpath = self.build_classpath(version_info, &libraries_dir, instance_dir)?;
        args.push("-cp".to_string());
        args.push(classpath);

        // Add main class
        args.push(version_info.main_class.clone());

        // Add game arguments
        let game_args = self.build_game_arguments(launch_config, version_info, instance_dir)?;
        args.extend(game_args);

        Ok(args)
    }

    fn build_classpath(&self, version_info: &VersionInfo, libraries_dir: &PathBuf, instance_dir: &PathBuf) -> Result<String> {
        let mut classpath_entries = Vec::new();

        // Add libraries first
        for library in &version_info.libraries {
            if let Some(rules) = &library.rules {
                if !self.evaluate_rules(rules) {
                    continue;
                }
            }

            // All libraries work normally via Rosetta 2 emulation on ARM64

            let library_path = self.get_library_path(&library.name, libraries_dir);
            classpath_entries.push(library_path.to_string_lossy().to_string());
        }

        // ARM compatibility is handled via JVM flags, not separate libraries

        // Add main client jar (this contains the main class)
        // The client jar should be in instance_dir/versions/{version_id}/{version_id}.jar
        let versions_dir = instance_dir.join("versions").join(&version_info.id);
        let client_jar = versions_dir.join(format!("{}.jar", version_info.id));
        classpath_entries.push(client_jar.to_string_lossy().to_string());

        log::info!("Built classpath with {} entries", classpath_entries.len());
        log::debug!("Client jar path: {}", client_jar.display());
        
        Ok(classpath_entries.join(if cfg!(windows) { ";" } else { ":" }))
    }

    fn build_game_arguments(
        &self,
        launch_config: &LaunchConfig,
        version_info: &VersionInfo,
        instance_dir: &PathBuf,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Handle modern argument format
        if let Some(arguments) = &version_info.arguments {
            for arg in &arguments.game {
                match arg {
                    crate::version::ArgumentValue::String(s) => {
                        args.push(self.substitute_argument_variables(s, launch_config, instance_dir));
                    }
                    crate::version::ArgumentValue::Conditional { rules, value } => {
                        if self.evaluate_rules(rules) {
                            for v in value {
                                args.push(self.substitute_argument_variables(v, launch_config, instance_dir));
                            }
                        }
                    }
                }
            }
        } 
        // Handle legacy argument format
        else if let Some(minecraft_arguments) = &version_info.minecraft_arguments {
            let legacy_args: Vec<&str> = minecraft_arguments.split_whitespace().collect();
            for arg in legacy_args {
                args.push(self.substitute_argument_variables(arg, launch_config, instance_dir));
            }
        }

        // Add additional game arguments
        args.extend(launch_config.additional_game_args.clone());

        Ok(args)
    }

    fn substitute_argument_variables(&self, arg: &str, launch_config: &LaunchConfig, instance_dir: &PathBuf) -> String {
        // Validate authentication data to prevent JSON parsing errors
        let safe_player_name = if launch_config.account.name.is_empty() {
            log::warn!("Empty player name detected, using placeholder");
            "Player".to_string()
        } else {
            launch_config.account.name.clone()
        };
        
        let safe_uuid = if launch_config.account.uuid.is_empty() {
            log::warn!("Empty UUID detected, using placeholder");
            "00000000-0000-0000-0000-000000000000".to_string()
        } else {
            launch_config.account.uuid.clone()
        };
        
        let safe_access_token = if launch_config.account.access_token.is_empty() {
            log::warn!("Empty access token detected, using placeholder");
            "placeholder_token".to_string()
        } else {
            launch_config.account.access_token.clone()
        };
        
        let safe_user_type = if launch_config.account.account_type.is_empty() {
            log::warn!("Empty user type detected, using 'msa' as default");
            "msa".to_string()
        } else {
            launch_config.account.account_type.clone()
        };

        arg.replace("${auth_player_name}", &safe_player_name)
            .replace("${version_name}", &launch_config.version)
            .replace("${game_directory}", &instance_dir.to_string_lossy())
            .replace("${assets_root}", &instance_dir.join("assets").to_string_lossy())
            .replace("${game_assets}", &instance_dir.join("assets").to_string_lossy())
            .replace("${auth_uuid}", &safe_uuid)
            .replace("${auth_access_token}", &safe_access_token)
            .replace("${user_type}", &safe_user_type)
            .replace("${version_type}", "release")
            .replace("${resolution_width}", &launch_config.window_config.width.to_string())
            .replace("${resolution_height}", &launch_config.window_config.height.to_string())
    }

    fn evaluate_rules(&self, rules: &[crate::version::Rule]) -> bool {
        for rule in rules {
            let mut matches = true;

            if let Some(os_rule) = &rule.os {
                matches &= self.evaluate_os_rule(os_rule);
            }

            if let Some(_features) = &rule.features {
                // Evaluate feature rules (not implemented for now)
                matches &= true;
            }

            if rule.action == "allow" && matches {
                return true;
            } else if rule.action == "disallow" && matches {
                return false;
            }
        }

        true // Default to allow
    }

    fn evaluate_os_rule(&self, os_rule: &crate::version::OsRule) -> bool {
        if let Some(os_name) = &os_rule.name {
            let current_os = if cfg!(windows) {
                "windows"
            } else if cfg!(target_os = "macos") {
                "osx"
            } else {
                "linux"
            };

            if os_name != current_os {
                return false;
            }
        }

        // TODO: Implement version and arch matching
        true
    }

    fn get_library_path(&self, library_name: &str, libraries_dir: &PathBuf) -> PathBuf {
        // Parse Maven coordinate: group:artifact:version[:classifier]
        let parts: Vec<&str> = library_name.split(':').collect();
        if parts.len() >= 3 {
            let group = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];
            let classifier = if parts.len() > 3 { format!("-{}", parts[3]) } else { String::new() };
            
            libraries_dir
                .join(group)
                .join(artifact)
                .join(version)
                .join(format!("{}-{}{}.jar", artifact, version, classifier))
        } else {
            libraries_dir.join(library_name)
        }
    }

    fn get_native_path(&self, library_name: &str, classifier: &str, libraries_dir: &PathBuf) -> PathBuf {
        let parts: Vec<&str> = library_name.split(':').collect();
        if parts.len() >= 3 {
            let group = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];
            
            libraries_dir
                .join(group)
                .join(artifact)
                .join(version)
                .join(format!("{}-{}-{}.jar", artifact, version, classifier))
        } else {
            libraries_dir.join(format!("{}-{}.jar", library_name, classifier))
        }
    }

    fn is_native_for_current_os(&self, classifier: &str) -> bool {
        if cfg!(windows) {
            classifier.contains("natives-windows")
        } else if cfg!(target_os = "macos") {
            classifier.contains("natives-osx") || classifier.contains("natives-macos")
        } else {
            classifier.contains("natives-linux")
        }
    }


    async fn extract_native_libraries(&self, version_info: &VersionInfo, instance_dir: &PathBuf) -> Result<()> {
        log::info!("Extracting native libraries for version {}", version_info.id);
        
        let libraries_dir = instance_dir.join("libraries");
        let natives_dir = instance_dir.join("versions").join(&version_info.id).join("natives");
        
        // Create natives directory
        if !natives_dir.exists() {
            std::fs::create_dir_all(&natives_dir)
                .map_err(|e| LauncherError::file(format!("Failed to create natives directory: {}", e)))?;
        }

        for library in &version_info.libraries {
            // Check if library applies to current OS
            if let Some(rules) = &library.rules {
                if !self.evaluate_rules(rules) {
                    continue;
                }
            }

            if let Some(downloads) = &library.downloads {
                if let Some(classifiers) = &downloads.classifiers {
                    for (classifier, _download_info) in classifiers {
                        if self.is_native_for_current_os(classifier) {
                            let native_jar_path = self.get_native_path(&library.name, classifier, &libraries_dir);
                            
                            if native_jar_path.exists() {
                                log::info!("Extracting native library: {}", native_jar_path.display());
                                self.extract_native_jar(&native_jar_path, &natives_dir).await?;
                            }
                        }
                    }
                }
            }
        }

        log::info!("Native libraries extracted to: {}", natives_dir.display());
        Ok(())
    }

    async fn extract_native_jar(&self, jar_path: &PathBuf, natives_dir: &PathBuf) -> Result<()> {
        
        let file = std::fs::File::open(jar_path)
            .map_err(|e| LauncherError::file(format!("Failed to open native JAR: {}", e)))?;
        
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| LauncherError::file(format!("Failed to read ZIP archive: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| LauncherError::file(format!("Failed to read ZIP entry: {}", e)))?;
            
            let file_path = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };

            // Skip META-INF directory
            if file_path.starts_with("META-INF") {
                continue;
            }

            let output_path = natives_dir.join(file_path);

            if file.is_dir() {
                std::fs::create_dir_all(&output_path)
                    .map_err(|e| LauncherError::file(format!("Failed to create directory: {}", e)))?;
            } else {
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| LauncherError::file(format!("Failed to create parent directory: {}", e)))?;
                }

                let mut output_file = std::fs::File::create(&output_path)
                    .map_err(|e| LauncherError::file(format!("Failed to create output file: {}", e)))?;
                
                std::io::copy(&mut file, &mut output_file)
                    .map_err(|e| LauncherError::file(format!("Failed to extract file: {}", e)))?;

                // Set executable permissions on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = output_file.metadata()
                        .map_err(|e| LauncherError::file(format!("Failed to get file metadata: {}", e)))?
                        .permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&output_path, perms)
                        .map_err(|e| LauncherError::file(format!("Failed to set file permissions: {}", e)))?;
                }
            }
        }

        Ok(())
    }

}
