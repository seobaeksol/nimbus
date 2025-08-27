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
    
    // Validate paths
    if src == dst {
        return Err("Source and destination cannot be the same".to_string());
    }
    
    // Check if source exists and get its info
    let src_info = match fs.get_file_info(src).await {
        Ok(info) => info,
        Err(core_engine::FileError::NotFound { path }) => {
            return Err(format!("Source file not found: {}", path));
        },
        Err(core_engine::FileError::PermissionDenied { path }) => {
            return Err(format!("Permission denied accessing source: {}", path));
        },
        Err(e) => return Err(format!("Failed to access source: {}", e)),
    };
    
    // Check if destination already exists
    if let Ok(_) = fs.get_file_info(dst).await {
        return Err(format!("Destination already exists: {}", dst.display()));
    }
    
    // Ensure destination directory exists
    if let Some(parent) = dst.parent() {
        if let Err(e) = fs.create_dir(parent).await {
            match e {
                core_engine::FileError::AlreadyExists { .. } => {
                    // Parent directory already exists, that's fine
                },
                _ => return Err(format!("Failed to create destination directory: {}", e)),
            }
        }
    }
    
    match src_info.file_type {
        core_engine::FileType::File => {
            match fs.copy_file(src, dst).await {
                Ok(()) => Ok(()),
                Err(core_engine::FileError::PermissionDenied { path }) => {
                    Err(format!("Permission denied copying file: {}", path))
                },
                Err(e) => Err(format!("Failed to copy file '{}': {}", src.display(), e)),
            }
        },
        core_engine::FileType::Directory => {
            match fs.copy_dir_recursive(src, dst).await {
                Ok(()) => Ok(()),
                Err(core_engine::FileError::PermissionDenied { path }) => {
                    Err(format!("Permission denied copying directory: {}", path))
                },
                Err(e) => Err(format!("Failed to copy directory '{}': {}", src.display(), e)),
            }
        },
        core_engine::FileType::Symlink => {
            Err("Copying symlinks is not currently supported".to_string())
        }
    }
}

/// Move/rename a file or directory
#[command]
pub async fn move_item(src_path: String, dst_path: String) -> CommandResult<()> {
    let fs = LocalFileSystem::new();
    let src = Path::new(&src_path);
    let dst = Path::new(&dst_path);
    
    // Validate paths
    if src == dst {
        return Err("Source and destination cannot be the same".to_string());
    }
    
    // Check if source exists
    if let Err(e) = fs.get_file_info(src).await {
        return match e {
            core_engine::FileError::NotFound { path } => {
                Err(format!("Source file not found: {}", path))
            },
            core_engine::FileError::PermissionDenied { path } => {
                Err(format!("Permission denied accessing source: {}", path))
            },
            _ => Err(format!("Failed to access source: {}", e)),
        };
    }
    
    // Check if destination already exists
    if let Ok(_) = fs.get_file_info(dst).await {
        return Err(format!("Destination already exists: {}", dst.display()));
    }
    
    // Ensure destination directory exists
    if let Some(parent) = dst.parent() {
        if let Err(e) = fs.create_dir(parent).await {
            match e {
                core_engine::FileError::AlreadyExists { .. } => {
                    // Parent directory already exists, that's fine
                },
                _ => return Err(format!("Failed to create destination directory: {}", e)),
            }
        }
    }
    
    match fs.move_file(src, dst).await {
        Ok(()) => Ok(()),
        Err(core_engine::FileError::PermissionDenied { path }) => {
            Err(format!("Permission denied moving item: {}", path))
        },
        Err(e) => Err(format!("Failed to move '{}' to '{}': {}", src.display(), dst.display(), e)),
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
        Err(core_engine::FileError::NotFound { path }) => {
            return Err(format!("Item not found: {}", path));
        },
        Err(core_engine::FileError::PermissionDenied { path }) => {
            return Err(format!("Permission denied accessing item: {}", path));
        },
        Err(e) => return Err(format!("Failed to access item: {}", e)),
    };
    
    // Check if item is read-only
    if item_info.is_readonly {
        return Err(format!("Cannot delete read-only item: {}", path));
    }
    
    match item_info.file_type {
        core_engine::FileType::File => {
            match fs.delete_file(item_path).await {
                Ok(()) => Ok(()),
                Err(core_engine::FileError::PermissionDenied { path }) => {
                    Err(format!("Permission denied deleting file: {}", path))
                },
                Err(e) => Err(format!("Failed to delete file '{}': {}", item_path.display(), e)),
            }
        },
        core_engine::FileType::Directory => {
            match fs.delete_dir(item_path).await {
                Ok(()) => Ok(()),
                Err(core_engine::FileError::PermissionDenied { path }) => {
                    Err(format!("Permission denied deleting directory: {}", path))
                },
                Err(e) => Err(format!("Failed to delete directory '{}': {}", item_path.display(), e)),
            }
        },
        core_engine::FileType::Symlink => {
            match fs.delete_file(item_path).await {
                Ok(()) => Ok(()),
                Err(core_engine::FileError::PermissionDenied { path }) => {
                    Err(format!("Permission denied deleting symlink: {}", path))
                },
                Err(e) => Err(format!("Failed to delete symlink '{}': {}", item_path.display(), e)),
            }
        }
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

