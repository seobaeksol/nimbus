//! Local file system implementation

use super::{FileError, FileInfo, FilePermissions, FileSystem, FileType, utils, PerformanceOptimizer, PerformanceConfig};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs;

/// Local file system implementation with performance optimization
pub struct LocalFileSystem {
    performance_optimizer: Option<Arc<PerformanceOptimizer>>,
}

impl Default for LocalFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalFileSystem {
    pub fn new() -> Self {
        Self {
            performance_optimizer: None,
        }
    }
    
    /// Create LocalFileSystem with performance optimization enabled
    pub fn with_performance_optimization(config: PerformanceConfig) -> Self {
        Self {
            performance_optimizer: Some(Arc::new(PerformanceOptimizer::new(config))),
        }
    }
    
    /// Create LocalFileSystem with default performance optimization
    pub fn with_default_performance() -> Self {
        Self::with_performance_optimization(PerformanceConfig::default())
    }
    
    /// Invalidate cache for a specific path
    pub fn invalidate_cache(&self, path: &Path) {
        if let Some(ref optimizer) = self.performance_optimizer {
            optimizer.invalidate_cache(path);
        }
    }
    
    /// Get performance statistics if available
    pub async fn get_performance_stats(&self) -> Option<super::PerformanceStats> {
        if let Some(ref optimizer) = self.performance_optimizer {
            Some(optimizer.get_stats().await)
        } else {
            None
        }
    }
    
    /// Get cache hit ratio if performance optimization is enabled
    pub async fn get_cache_hit_ratio(&self) -> f64 {
        if let Some(ref optimizer) = self.performance_optimizer {
            optimizer.get_cache_hit_ratio().await
        } else {
            0.0
        }
    }
    
    /// Convert std::fs::Metadata to FileInfo
    async fn metadata_to_file_info(&self, path: &Path, metadata: &std::fs::Metadata) -> Result<FileInfo, FileError> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
            
        let file_type = if metadata.is_file() {
            FileType::File
        } else if metadata.is_dir() {
            FileType::Directory
        } else {
            FileType::Symlink
        };
        
        let permissions = FilePermissions {
            read: !metadata.permissions().readonly(),
            write: !metadata.permissions().readonly(),
            execute: false, // Simplified for now
        };
        
        let size = if file_type == FileType::File {
            metadata.len()
        } else {
            0
        };
        
        Ok(FileInfo {
            name: file_name.clone(),
            path: dunce::canonicalize(path)
                .unwrap_or_else(|_| path.to_path_buf())
                .to_string_lossy()
                .to_string(),
            size,
            size_formatted: utils::format_file_size(size),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            created: metadata.created().ok(),
            accessed: metadata.accessed().ok(),
            file_type,
            extension: utils::get_extension(path),
            permissions,
            is_hidden: utils::is_hidden(&file_name),
            is_readonly: metadata.permissions().readonly(),
        })
    }
}

#[async_trait]
impl FileSystem for LocalFileSystem {
    async fn list_dir(&self, path: &Path) -> Result<Vec<FileInfo>, FileError> {
        // Try to use cached result if performance optimization is enabled
        if let Some(ref optimizer) = self.performance_optimizer {
            if let Some(cached_files) = optimizer.get_cached_directory(path).await {
                return Ok(cached_files);
            }
        }
        
        // Execute the directory listing operation
        let operation = async {
            let mut entries = fs::read_dir(path).await.map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => FileError::NotFound {
                    path: path.to_string_lossy().to_string(),
                },
                std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                    path: path.to_string_lossy().to_string(),
                },
                _ => FileError::Io(e),
            })?;
            
            let mut files = Vec::new();
            
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let metadata = entry.metadata().await?;
                
                match self.metadata_to_file_info(&entry_path, &metadata).await {
                    Ok(file_info) => files.push(file_info),
                    Err(_) => continue, // Skip files we can't read
                }
            }
            
            // Sort by name for consistent ordering
            files.sort_by(|a, b| {
                // Directories first, then files
                match (&a.file_type, &b.file_type) {
                    (FileType::Directory, FileType::Directory) => a.name.cmp(&b.name),
                    (FileType::Directory, _) => std::cmp::Ordering::Less,
                    (_, FileType::Directory) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });
            
            Ok::<Vec<FileInfo>, FileError>(files)
        };
        
        // Execute with or without performance optimization
        let result = if let Some(ref optimizer) = self.performance_optimizer {
            optimizer.execute_with_limits(operation).await
        } else {
            operation.await
        };
        
        // If successful and performance optimization is enabled, cache the result
        if let (Ok(ref files), Some(ref optimizer)) = (&result, &self.performance_optimizer) {
            optimizer.cache_directory(path, files.clone()).await;
        }
        
        result
    }
    
    async fn get_file_info(&self, path: &Path) -> Result<FileInfo, FileError> {
        let metadata = fs::metadata(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound {
                path: path.to_string_lossy().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            _ => FileError::Io(e),
        })?;
        
        self.metadata_to_file_info(path, &metadata).await
    }
    
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileError> {
        fs::read(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound {
                path: path.to_string_lossy().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            _ => FileError::Io(e),
        })
    }
    
    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), FileError> {
        fs::write(path, data).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            _ => FileError::Io(e),
        })?;
        
        // Invalidate cache for parent directory (in case it's a new file)
        if let Some(parent) = path.parent() {
            self.invalidate_cache(parent);
        }
        
        Ok(())
    }
    
    async fn copy_file(&self, src: &Path, dst: &Path) -> Result<(), FileError> {
        fs::copy(src, dst).await?;
        
        // Invalidate cache for destination directory
        if let Some(parent) = dst.parent() {
            self.invalidate_cache(parent);
        }
        
        Ok(())
    }
    
    async fn move_file(&self, src: &Path, dst: &Path) -> Result<(), FileError> {
        fs::rename(src, dst).await.map_err(FileError::Io)?;
        
        // Invalidate cache for both source and destination directories
        if let Some(src_parent) = src.parent() {
            self.invalidate_cache(src_parent);
        }
        if let Some(dst_parent) = dst.parent() {
            self.invalidate_cache(dst_parent);
        }
        
        Ok(())
    }
    
    async fn delete_file(&self, path: &Path) -> Result<(), FileError> {
        fs::remove_file(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound {
                path: path.to_string_lossy().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            _ => FileError::Io(e),
        })?;
        
        // Invalidate cache for parent directory
        if let Some(parent) = path.parent() {
            self.invalidate_cache(parent);
        }
        
        Ok(())
    }
    
    async fn create_dir(&self, path: &Path) -> Result<(), FileError> {
        fs::create_dir_all(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => FileError::AlreadyExists {
                path: path.to_string_lossy().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            _ => FileError::Io(e),
        })?;
        
        // Invalidate cache for parent directory
        if let Some(parent) = path.parent() {
            self.invalidate_cache(parent);
        }
        
        Ok(())
    }
    
    async fn delete_dir(&self, path: &Path) -> Result<(), FileError> {
        fs::remove_dir_all(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound {
                path: path.to_string_lossy().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            _ => FileError::Io(e),
        })?;
        
        // Invalidate cache for the directory itself and its parent
        self.invalidate_cache(path);
        if let Some(parent) = path.parent() {
            self.invalidate_cache(parent);
        }
        
        Ok(())
    }

    async fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<(), FileError> {
        // Create the destination directory
        self.create_dir(dst).await?;
        
        // List all entries in the source directory
        let entries = self.list_dir(src).await?;
        
        for entry in entries {
            let src_path = Path::new(&entry.path);
            let dst_path = dst.join(&entry.name);
            
            match entry.file_type {
                FileType::File => {
                    self.copy_file(src_path, &dst_path).await?;
                },
                FileType::Directory => {
                    // Recursively copy subdirectory
                    Box::pin(self.copy_dir_recursive(src_path, &dst_path)).await?;
                },
                FileType::Symlink => {
                    // For now, skip symlinks - could add support later
                    continue;
                }
            }
        }
        
        Ok(())
    }
}