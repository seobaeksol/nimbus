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

/// Copy a file or directory
#[command]
pub async fn copy_item(src_path: String, dst_path: String) -> CommandResult<()> {
    let fs = LocalFileSystem::new();
    let src = Path::new(&src_path);
    let dst = Path::new(&dst_path);
    
    // Check if source exists and get its info
    let src_info = match fs.get_file_info(src).await {
        Ok(info) => info,
        Err(e) => return Err(format!("Source not found: {}", e)),
    };
    
    match src_info.file_type {
        core_engine::FileType::File => {
            match fs.copy_file(src, dst).await {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Failed to copy file: {}", e)),
            }
        },
        core_engine::FileType::Directory => {
            // For now, just create the directory without recursion
            // TODO: Implement proper recursive directory copying
            match fs.create_dir(dst).await {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Failed to copy directory: {}", e)),
            }
        },
        _ => Err("Unsupported file type for copying".to_string()),
    }
}

/// Move/rename a file or directory
#[command]
pub async fn move_item(src_path: String, dst_path: String) -> CommandResult<()> {
    let fs = LocalFileSystem::new();
    let src = Path::new(&src_path);
    let dst = Path::new(&dst_path);
    
    match fs.move_file(src, dst).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to move item: {}", e)),
    }
}

/// Delete a file or directory
#[command]
pub async fn delete_item(path: String) -> CommandResult<()> {
    let fs = LocalFileSystem::new();
    let item_path = Path::new(&path);
    
    // Check if the item exists and get its info
    let item_info = match fs.get_file_info(item_path).await {
        Ok(info) => info,
        Err(e) => return Err(format!("Item not found: {}", e)),
    };
    
    match item_info.file_type {
        core_engine::FileType::File => {
            match fs.delete_file(item_path).await {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Failed to delete file: {}", e)),
            }
        },
        core_engine::FileType::Directory => {
            match fs.delete_dir(item_path).await {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Failed to delete directory: {}", e)),
            }
        },
        _ => Err("Unsupported file type for deletion".to_string()),
    }
}

/// Rename a file or directory
#[command]
pub async fn rename_item(old_path: String, new_name: String) -> CommandResult<()> {
    let old = Path::new(&old_path);
    let parent = old.parent().ok_or("Invalid path: no parent directory")?;
    let new_path = parent.join(&new_name);
    
    let fs = LocalFileSystem::new();
    match fs.move_file(old, &new_path).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to rename item: {}", e)),
    }
}

/// Create a new file
#[command]
pub async fn create_file(path: String, name: String) -> CommandResult<()> {
    let fs = LocalFileSystem::new();
    let full_path = Path::new(&path).join(&name);
    
    match fs.write_file(&full_path, &[]).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to create file: {}", e)),
    }
}

