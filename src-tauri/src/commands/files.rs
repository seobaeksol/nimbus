//! File system operation commands

use core_engine::{FileInfo, FileSystem, LocalFileSystem};
use std::path::Path;
use tauri::command;

use super::CommandResult;

/// List the contents of a directory
#[command]
pub async fn list_dir(path: String) -> CommandResult<Vec<FileInfo>> {
    let fs = LocalFileSystem::new();
    let path = Path::new(&path);
    
    match fs.list_dir(path).await {
        Ok(files) => Ok(files),
        Err(e) => Err(format!("Failed to list directory: {}", e)),
    }
}

/// Get detailed information about a file or directory
#[command]
pub async fn get_file_info(path: String) -> CommandResult<FileInfo> {
    let fs = LocalFileSystem::new();
    let path = Path::new(&path);
    
    match fs.get_file_info(path).await {
        Ok(info) => Ok(info),
        Err(e) => Err(format!("Failed to get file info: {}", e)),
    }
}

/// Create a new directory
#[command]
pub async fn create_directory(path: String, name: String) -> CommandResult<()> {
    let fs = LocalFileSystem::new();
    let full_path = Path::new(&path).join(&name);
    
    match fs.create_dir(&full_path).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to create directory: {}", e)),
    }
}