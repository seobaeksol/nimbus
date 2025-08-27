//! System information and utility commands

use serde::{Deserialize, Serialize};
use std::env;
use tauri::command;

use super::CommandResult;

// App constants
mod constants {
    pub const APP_NAME: &str = "Nimbus";
    pub const VERSION: &str = "0.1.0";
}

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub platform: String,
    pub arch: String,
    pub hostname: String,
    pub username: String,
    pub home_dir: Option<String>,
    pub temp_dir: String,
    pub app_name: String,
    pub app_version: String,
}

/// Get system information
#[command]
pub async fn get_system_info() -> CommandResult<SystemInfo> {
    let info = SystemInfo {
        platform: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        username: whoami::username(),
        home_dir: dirs::home_dir().map(|p| p.to_string_lossy().to_string()),
        temp_dir: env::temp_dir().to_string_lossy().to_string(),
        app_name: constants::APP_NAME.to_string(),
        app_version: constants::VERSION.to_string(),
    };
    
    Ok(info)
}

/// Simple greeting command for testing IPC
#[command]
pub async fn greet(name: &str) -> CommandResult<String> {
    Ok(format!("Hello {}, you've been greeted from Rust!", name))
}