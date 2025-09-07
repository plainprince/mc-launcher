mod launcher;

use launcher::LauncherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Minecraft Launcher Tauri app");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(LauncherState::new())
        .invoke_handler(tauri::generate_handler![
            launcher::initialize_launcher,
            launcher::setup_authenticator,
            launcher::get_auth_url,
            launcher::authenticate_with_code,
            launcher::start_device_code_flow,
            launcher::poll_device_code,
            launcher::refresh_account,
            launcher::get_version_manifest,
            launcher::get_java_runtime,
            launcher::launch_minecraft,
            launcher::get_process_status,
            launcher::kill_minecraft,
            launcher::get_minecraft_logs,
            launcher::list_active_processes,
            launcher::is_process_running,
            launcher::get_home_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
