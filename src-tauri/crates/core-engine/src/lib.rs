//! Core Engine for Nimbus File Manager
//! 
//! Provides the unified FileSystem trait and core file management functionality.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;
use thiserror::Error;

pub mod local_fs;

pub use local_fs::LocalFileSystem;

/// File system error types
#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found: {path}")]
    NotFound { path: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    #[error("File already exists: {path}")]
    AlreadyExists { path: String },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// File type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

/// File permissions structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

/// File information structure matching the API documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub size_formatted: String,
    pub modified: SystemTime,
    pub created: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub file_type: FileType,
    pub extension: Option<String>,
    pub permissions: FilePermissions,
    pub is_hidden: bool,
    pub is_readonly: bool,
}

/// Unified file system trait
#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn list_dir(&self, path: &Path) -> Result<Vec<FileInfo>, FileError>;
    async fn get_file_info(&self, path: &Path) -> Result<FileInfo, FileError>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileError>;
    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), FileError>;
    async fn copy_file(&self, src: &Path, dst: &Path) -> Result<(), FileError>;
    async fn move_file(&self, src: &Path, dst: &Path) -> Result<(), FileError>;
    async fn delete_file(&self, path: &Path) -> Result<(), FileError>;
    async fn create_dir(&self, path: &Path) -> Result<(), FileError>;
    async fn delete_dir(&self, path: &Path) -> Result<(), FileError>;
    async fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<(), FileError>;
}

/// Utility functions
pub mod utils {
    use super::*;
    
    /// Format file size in human-readable format
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    /// Check if a file name is hidden (starts with .)
    pub fn is_hidden(name: &str) -> bool {
        name.starts_with('.')
    }
    
    /// Get file extension
    pub fn get_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }
}