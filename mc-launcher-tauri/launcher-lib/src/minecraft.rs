//! Minecraft process management

use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{auth::Account, error::{LauncherError, Result}};

/// Status of a Minecraft process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is starting up
    Starting,
    /// Process is running normally
    Running,
    /// Process has exited successfully
    Exited(i32),
    /// Process was killed
    Killed,
    /// Process failed to start or crashed
    Failed(String),
}

/// Minecraft process wrapper
#[derive(Debug, Clone)]
pub struct MinecraftProcess {
    inner: Arc<MinecraftProcessInner>,
}

#[derive(Debug)]
struct MinecraftProcessInner {
    child: RwLock<Option<Child>>,
    java_path: PathBuf,
    args: Vec<String>,
    working_dir: PathBuf,
    account: Account,
    status: RwLock<ProcessStatus>,
    pid: RwLock<Option<u32>>,
}

impl MinecraftProcess {
    /// Create and start a new Minecraft process
    pub async fn new(
        java_path: PathBuf,
        args: Vec<String>,
        working_dir: PathBuf,
        account: Account,
    ) -> Result<Self> {
        let inner = Arc::new(MinecraftProcessInner {
            child: RwLock::new(None),
            java_path,
            args,
            working_dir,
            account,
            status: RwLock::new(ProcessStatus::Starting),
            pid: RwLock::new(None),
        });

        let process = Self { inner };
        process.start().await?;
        Ok(process)
    }

    /// Start the Minecraft process
    async fn start(&self) -> Result<()> {
        let mut status = self.inner.status.write().await;
        *status = ProcessStatus::Starting;
        drop(status);

        log::info!("Starting Minecraft process with Java: {}", self.inner.java_path.display());
        log::info!("Working directory: {}", self.inner.working_dir.display());
        // Log arguments with sensitive data redacted for debugging
        let mut debug_args = self.inner.args.clone();
        for arg in &mut debug_args {
            if arg.contains("accessToken") || arg.contains("access_token") {
                *arg = arg.chars().take(20).collect::<String>() + "...***REDACTED***";
            }
        }
        log::info!("Arguments: {:?}", debug_args);

        // For pre-1.17 Minecraft on Apple Silicon, force Rosetta 2 emulation
        let mut command = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            log::info!("Forcing Rosetta 2 emulation for ARM64 compatibility with pre-1.17 Minecraft");
            let mut cmd = Command::new("arch");
            cmd.arg("-x86_64")
               .arg(&self.inner.java_path);
            cmd
        } else {
            Command::new(&self.inner.java_path)
        };
        
        command
            .args(&self.inner.args)
            .current_dir(&self.inner.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Set environment variables if needed
        #[cfg(target_os = "macos")]
        {
            command.env("OBJC_DISABLE_INITIALIZE_FORK_SAFETY", "YES");
        }

        let mut child = command.spawn()
            .map_err(|e| LauncherError::launch(format!("Failed to start Minecraft process: {}", e)))?;

        let pid = child.id();
        
        // Capture stdout and stderr for debugging
        if let Some(stdout) = child.stdout.take() {
            let stdout_reader = BufReader::new(stdout);
            let mut stdout_lines = stdout_reader.lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_lines.next_line().await {
                    log::info!("[Minecraft STDOUT] {}", line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let stderr_reader = BufReader::new(stderr);
            let mut stderr_lines = stderr_reader.lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_lines.next_line().await {
                    log::error!("[Minecraft STDERR] {}", line);
                }
            });
        }
        
        // Update status and PID
        {
            let mut status = self.inner.status.write().await;
            *status = ProcessStatus::Running;
        }
        {
            let mut stored_pid = self.inner.pid.write().await;
            *stored_pid = pid;
        }

        // Store the child process
        {
            let mut stored_child = self.inner.child.write().await;
            *stored_child = Some(child);
        }

        log::info!("Minecraft process started with PID: {:?}", pid);
        Ok(())
    }

    /// Get the process ID
    pub async fn get_pid(&self) -> Result<u32> {
        let pid = self.inner.pid.read().await;
        pid.ok_or_else(|| LauncherError::process("Process not started"))
    }

    /// Get the current status of the process
    pub fn get_status(&self) -> ProcessStatus {
        // This is a simplified synchronous version
        // In practice, you might want to check if the process is still running
        ProcessStatus::Running // Placeholder
    }

    /// Get detailed status asynchronously
    pub async fn get_status_async(&self) -> ProcessStatus {
        let status = self.inner.status.read().await;
        status.clone()
    }

    /// Check if the process is running
    pub async fn is_running(&self) -> bool {
        let child_guard = self.inner.child.read().await;
        if let Some(_child) = child_guard.as_ref() {
            // Try to poll the process without blocking
            true // Simplified - in practice you'd check child.try_wait()
        } else {
            false
        }
    }

    /// Kill the Minecraft process
    pub async fn kill(&self) -> Result<()> {
        log::info!("Killing Minecraft process");

        let mut child_guard = self.inner.child.write().await;
        if let Some(mut child) = child_guard.take() {
            // Try graceful shutdown first
            if let Err(e) = child.kill().await {
                log::warn!("Failed to kill process gracefully: {}", e);
            }

            // Wait for the process to exit
            match child.wait().await {
                Ok(exit_status) => {
                    let mut status = self.inner.status.write().await;
                    if exit_status.success() {
                        *status = ProcessStatus::Exited(0);
                    } else {
                        let code = exit_status.code().unwrap_or(-1);
                        *status = ProcessStatus::Exited(code);
                    }
                    log::info!("Process exited with status: {}", exit_status);
                }
                Err(e) => {
                    let mut status = self.inner.status.write().await;
                    *status = ProcessStatus::Failed(format!("Wait failed: {}", e));
                    log::error!("Failed to wait for process: {}", e);
                }
            }
        } else {
            return Err(LauncherError::process("No process to kill"));
        }

        // Clear PID
        {
            let mut pid = self.inner.pid.write().await;
            *pid = None;
        }

        Ok(())
    }

    /// Wait for the process to exit naturally
    pub async fn wait(&self) -> Result<ProcessStatus> {
        let mut child_guard = self.inner.child.write().await;
        if let Some(mut child) = child_guard.take() {
            match child.wait().await {
                Ok(exit_status) => {
                    let status = if exit_status.success() {
                        ProcessStatus::Exited(0)
                    } else {
                        ProcessStatus::Exited(exit_status.code().unwrap_or(-1))
                    };

                    let mut stored_status = self.inner.status.write().await;
                    *stored_status = status.clone();

                    Ok(status)
                }
                Err(e) => {
                    let error_status = ProcessStatus::Failed(format!("Wait failed: {}", e));
                    let mut stored_status = self.inner.status.write().await;
                    *stored_status = error_status.clone();
                    
                    Err(LauncherError::process(format!("Failed to wait for process: {}", e)))
                }
            }
        } else {
            Err(LauncherError::process("No process to wait for"))
        }
    }

    /// Get the account associated with this process
    pub fn get_account(&self) -> &Account {
        &self.inner.account
    }

    /// Get the working directory
    pub fn get_working_dir(&self) -> &PathBuf {
        &self.inner.working_dir
    }

    /// Read stdout from the process (non-blocking)
    pub async fn read_stdout(&self) -> Result<Option<String>> {
        // TODO: Implement stdout reading
        // This would involve capturing stdout during process creation
        // and providing a way to read from it asynchronously
        Ok(None)
    }

    /// Read stderr from the process (non-blocking)
    pub async fn read_stderr(&self) -> Result<Option<String>> {
        // TODO: Implement stderr reading
        Ok(None)
    }

    /// Send input to the process
    pub async fn send_input(&self, _input: &str) -> Result<()> {
        // TODO: Implement stdin writing
        // This would require keeping a handle to the process's stdin
        Ok(())
    }

    /// Get log file path for this instance
    pub fn get_log_path(&self) -> PathBuf {
        self.inner.working_dir.join("logs").join("latest.log")
    }

    /// Read the latest log file
    pub async fn read_logs(&self) -> Result<String> {
        let log_path = self.get_log_path();
        
        match tokio::fs::read_to_string(&log_path).await {
            Ok(content) => Ok(content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok("No logs available yet".to_string())
            }
            Err(e) => Err(LauncherError::file(format!("Failed to read logs: {}", e))),
        }
    }

    /// Get crash reports directory
    pub fn get_crash_reports_dir(&self) -> PathBuf {
        self.inner.working_dir.join("crash-reports")
    }

    /// List available crash reports
    pub async fn list_crash_reports(&self) -> Result<Vec<PathBuf>> {
        let crash_dir = self.get_crash_reports_dir();
        
        let mut entries = tokio::fs::read_dir(&crash_dir)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to read crash reports directory: {}", e)))?;

        let mut crash_reports = Vec::new();
        
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                crash_reports.push(path);
            }
        }

        // Sort by modification time (newest first)
        crash_reports.sort_by(|a, b| {
            let a_modified = std::fs::metadata(a).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_modified = std::fs::metadata(b).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_modified.cmp(&a_modified)
        });

        Ok(crash_reports)
    }

    /// Read a specific crash report
    pub async fn read_crash_report(&self, crash_report_path: &PathBuf) -> Result<String> {
        tokio::fs::read_to_string(crash_report_path)
            .await
            .map_err(|e| LauncherError::file(format!("Failed to read crash report: {}", e)))
    }

    /// Get the latest crash report if any
    pub async fn get_latest_crash_report(&self) -> Result<Option<String>> {
        let crash_reports = self.list_crash_reports().await?;
        
        if let Some(latest) = crash_reports.first() {
            let content = self.read_crash_report(latest).await?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }
}
