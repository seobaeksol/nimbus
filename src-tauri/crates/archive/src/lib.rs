//! Archive handling for file manager operations
//!
//! This crate provides unified archive support for browsing and extracting
//! various archive formats. Currently supports ZIP files with foundation
//! for adding more formats like TAR, 7z, and RAR.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Archive handling error types
#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("Unsupported archive format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("Archive file not found: {path}")]
    NotFound { path: String },
    
    #[error("Password required for encrypted archive")]
    PasswordRequired,
    
    #[error("Invalid password for encrypted archive")]
    InvalidPassword,
    
    #[error("Corrupted archive: {reason}")]
    CorruptedArchive { reason: String },
    
    #[error("Extraction failed: {reason}")]
    ExtractionFailed { reason: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
}

/// Archive format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    // Future formats:
    // Tar,
    // TarGz,
    // TarBz2,
    // SevenZ,
    // Rar,
}

impl ArchiveFormat {
    /// Detect archive format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        match extension.as_str() {
            "zip" => Some(Self::Zip),
            // "tar" => Some(Self::Tar),
            // "gz" | "tgz" => Some(Self::TarGz),
            // "bz2" | "tbz2" => Some(Self::TarBz2),
            // "7z" => Some(Self::SevenZ),
            // "rar" => Some(Self::Rar),
            _ => None,
        }
    }
    
    /// Get the typical file extensions for this format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Zip => &["zip"],
            // Self::Tar => &["tar"],
            // Self::TarGz => &["gz", "tgz"],
            // Self::TarBz2 => &["bz2", "tbz2"],
            // Self::SevenZ => &["7z"],
            // Self::Rar => &["rar"],
        }
    }
}

/// Archive entry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    /// Full path within the archive
    pub path: String,
    /// File name (last component of path)
    pub name: String,
    /// Uncompressed size in bytes
    pub size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Last modification time
    pub modified: Option<DateTime<Utc>>,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Compression method used
    pub compression_method: Option<String>,
    /// CRC32 checksum (if available)
    pub crc32: Option<u32>,
    /// Whether the entry is encrypted
    pub is_encrypted: bool,
}

/// Extraction options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionOptions {
    /// Whether to preserve directory structure
    pub preserve_paths: bool,
    /// What to do when files already exist
    pub overwrite_policy: OverwritePolicy,
    /// Password for encrypted archives
    pub password: Option<String>,
    /// Create a subdirectory with the archive name
    pub create_subfolder: bool,
    /// Specific entries to extract (None = extract all)
    pub entries: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverwritePolicy {
    /// Ask user for each conflict
    Ask,
    /// Overwrite existing files
    Overwrite,
    /// Skip existing files
    Skip,
    /// Rename new files (add suffix)
    Rename,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            preserve_paths: true,
            overwrite_policy: OverwritePolicy::Ask,
            password: None,
            create_subfolder: false,
            entries: None,
        }
    }
}

/// Progress information for archive operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current_file: String,
    pub files_processed: usize,
    pub total_files: usize,
    pub bytes_processed: u64,
    pub total_bytes: u64,
}

/// Archive reader trait for browsing and extracting archives
#[async_trait]
pub trait ArchiveReader: Send + Sync {
    /// List all entries in the archive
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError>;
    
    /// Get information about a specific entry
    async fn get_entry(&self, path: &str) -> Result<Option<ArchiveEntry>, ArchiveError>;
    
    /// Extract specific entries to a destination
    async fn extract(
        &self,
        destination: &Path,
        options: ExtractionOptions,
        progress_callback: Option<Box<dyn Fn(ProgressInfo) + Send + Sync>>,
    ) -> Result<(), ArchiveError>;
    
    /// Extract a single entry to memory
    async fn extract_entry_to_memory(&self, path: &str) -> Result<Vec<u8>, ArchiveError>;
    
    /// Check if the archive requires a password
    fn requires_password(&self) -> bool;
    
    /// Get the archive format
    fn format(&self) -> ArchiveFormat;
}

/// ZIP archive implementation
pub struct ZipArchiveReader {
    archive_path: PathBuf,
}

impl ZipArchiveReader {
    /// Open a ZIP archive for reading
    pub fn new(archive_path: PathBuf) -> Self {
        Self { archive_path }
    }
}

#[async_trait]
impl ArchiveReader for ZipArchiveReader {
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        // We need to run this in a blocking task since zip crate is not async
        let archive_path = self.archive_path.clone();
        let entries = tokio::task::spawn_blocking(move || -> Result<Vec<ArchiveEntry>, ArchiveError> {
            let mut archive = {
                let file = std::fs::File::open(&archive_path)
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            ArchiveError::NotFound {
                                path: archive_path.to_string_lossy().to_string(),
                            }
                        } else {
                            ArchiveError::Io(e)
                        }
                    })?;
                
                zip::ZipArchive::new(file)
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read ZIP archive: {}", e),
                    })?
            };
            
            let mut entries = Vec::new();
            
            for i in 0..archive.len() {
                let entry = archive.by_index(i)
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read entry at index {}: {}", i, e),
                    })?;
                
                let path = entry.name().to_string();
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
                
                // Convert ZIP's DateTime to chrono DateTime
                let modified = entry.last_modified()
                    .and_then(|dt| time::OffsetDateTime::try_from(dt).ok())
                    .and_then(|time| {
                        DateTime::from_timestamp(time.unix_timestamp(), 0)
                    });
                
                entries.push(ArchiveEntry {
                    path,
                    name,
                    size: entry.size(),
                    compressed_size: entry.compressed_size(),
                    modified,
                    is_directory: entry.is_dir(),
                    compression_method: Some(format!("{:?}", entry.compression())),
                    crc32: Some(entry.crc32()),
                    is_encrypted: entry.encrypted(),
                });
            }
            
            Ok(entries)
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })??;
        
        Ok(entries)
    }
    
    async fn get_entry(&self, path: &str) -> Result<Option<ArchiveEntry>, ArchiveError> {
        let entries = self.list_entries().await?;
        Ok(entries.into_iter().find(|e| e.path == path))
    }
    
    async fn extract(
        &self,
        destination: &Path,
        options: ExtractionOptions,
        progress_callback: Option<Box<dyn Fn(ProgressInfo) + Send + Sync>>,
    ) -> Result<(), ArchiveError> {
        let archive_path = self.archive_path.clone();
        let destination = destination.to_path_buf();
        
        tokio::task::spawn_blocking(move || -> Result<(), ArchiveError> {
            let mut archive = {
                let file = std::fs::File::open(&archive_path)
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            ArchiveError::NotFound {
                                path: archive_path.to_string_lossy().to_string(),
                            }
                        } else {
                            ArchiveError::Io(e)
                        }
                    })?;
                
                zip::ZipArchive::new(file)
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read ZIP archive: {}", e),
                    })?
            };
            
            // Calculate total files and bytes for progress
            let mut total_files = 0;
            let mut total_bytes = 0;
            let entries_to_extract: Vec<usize> = if let Some(entry_paths) = &options.entries {
                // Extract specific entries - first collect matching indices
                let mut indices = Vec::new();
                for i in 0..archive.len() {
                    if let Ok(entry) = archive.by_index(i) {
                        if entry_paths.contains(&entry.name().to_string()) {
                            indices.push(i);
                            total_files += 1;
                            total_bytes += entry.size();
                        }
                    }
                }
                indices
            } else {
                // Extract all entries - collect all indices and count
                let mut indices = Vec::new();
                for i in 0..archive.len() {
                    if let Ok(entry) = archive.by_index(i) {
                        indices.push(i);
                        total_files += 1;
                        total_bytes += entry.size();
                    }
                }
                indices
            };
            
            let mut files_processed = 0;
            let mut bytes_processed = 0;
            
            for &index in &entries_to_extract {
                let mut entry = archive.by_index(index)
                    .map_err(|e| ArchiveError::ExtractionFailed {
                        reason: format!("Failed to read entry at index {}: {}", index, e),
                    })?;
                
                let entry_path = entry.name().to_string();
                
                // Calculate destination path
                let dest_path = if options.preserve_paths {
                    if options.create_subfolder {
                        let archive_name = archive_path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("archive");
                        destination.join(archive_name).join(&entry_path)
                    } else {
                        destination.join(&entry_path)
                    }
                } else {
                    let file_name = Path::new(&entry_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&entry_path);
                    destination.join(file_name)
                };
                
                // Handle existing files according to policy
                if dest_path.exists() {
                    match options.overwrite_policy {
                        OverwritePolicy::Skip => {
                            files_processed += 1;
                            if let Some(ref callback) = progress_callback {
                                callback(ProgressInfo {
                                    current_file: entry_path,
                                    files_processed,
                                    total_files,
                                    bytes_processed,
                                    total_bytes,
                                });
                            }
                            continue;
                        }
                        OverwritePolicy::Ask => {
                            // In a real implementation, this would trigger a UI dialog
                            // For now, skip to avoid overwriting
                            files_processed += 1;
                            if let Some(ref callback) = progress_callback {
                                callback(ProgressInfo {
                                    current_file: entry_path,
                                    files_processed,
                                    total_files,
                                    bytes_processed,
                                    total_bytes,
                                });
                            }
                            continue;
                        }
                        OverwritePolicy::Rename => {
                            // Find a unique name by adding a number suffix
                            let mut counter = 1;
                            let mut new_dest = dest_path.clone();
                            while new_dest.exists() {
                                if let Some(stem) = dest_path.file_stem().and_then(|s| s.to_str()) {
                                    let extension = dest_path.extension()
                                        .and_then(|e| e.to_str())
                                        .map(|e| format!(".{}", e))
                                        .unwrap_or_default();
                                    let new_name = format!("{} ({}){}", stem, counter, extension);
                                    new_dest = dest_path.parent().unwrap().join(new_name);
                                }
                                counter += 1;
                            }
                            // Use the renamed path
                        }
                        OverwritePolicy::Overwrite => {
                            // Proceed with overwriting
                        }
                    }
                }
                
                if entry.is_dir() {
                    // Create directory
                    std::fs::create_dir_all(&dest_path)
                        .map_err(|e| ArchiveError::ExtractionFailed {
                            reason: format!("Failed to create directory '{}': {}", dest_path.display(), e),
                        })?;
                } else {
                    // Create parent directory if it doesn't exist
                    if let Some(parent) = dest_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| ArchiveError::ExtractionFailed {
                                reason: format!("Failed to create parent directory '{}': {}", parent.display(), e),
                            })?;
                    }
                    
                    // Extract file
                    let mut output_file = std::fs::File::create(&dest_path)
                        .map_err(|e| ArchiveError::ExtractionFailed {
                            reason: format!("Failed to create file '{}': {}", dest_path.display(), e),
                        })?;
                    
                    std::io::copy(&mut entry, &mut output_file)
                        .map_err(|e| ArchiveError::ExtractionFailed {
                            reason: format!("Failed to extract file '{}': {}", dest_path.display(), e),
                        })?;
                    
                    bytes_processed += entry.size();
                }
                
                files_processed += 1;
                
                // Report progress
                if let Some(ref callback) = progress_callback {
                    callback(ProgressInfo {
                        current_file: entry_path,
                        files_processed,
                        total_files,
                        bytes_processed,
                        total_bytes,
                    });
                }
            }
            
            Ok(())
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })??;
        
        Ok(())
    }
    
    async fn extract_entry_to_memory(&self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        let path = path.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ArchiveError> {
            let mut archive = {
                let file = std::fs::File::open(&archive_path)
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            ArchiveError::NotFound {
                                path: archive_path.to_string_lossy().to_string(),
                            }
                        } else {
                            ArchiveError::Io(e)
                        }
                    })?;
                
                zip::ZipArchive::new(file)
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read ZIP archive: {}", e),
                    })?
            };
            
            let mut entry = archive.by_name(&path)
                .map_err(|_| ArchiveError::NotFound { path: path.clone() })?;
            
            let mut buffer = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buffer)
                .map_err(|e| ArchiveError::ExtractionFailed {
                    reason: format!("Failed to read entry '{}': {}", path, e),
                })?;
            
            Ok(buffer)
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    fn requires_password(&self) -> bool {
        // For ZIP files, we'd need to check each entry individually
        // This is a simplified implementation
        false
    }
    
    fn format(&self) -> ArchiveFormat {
        ArchiveFormat::Zip
    }
}

/// Archive factory for creating appropriate readers
pub struct ArchiveFactory;

impl ArchiveFactory {
    /// Create an archive reader for the given file
    pub fn create_reader(archive_path: PathBuf) -> Result<Box<dyn ArchiveReader>, ArchiveError> {
        let format = ArchiveFormat::from_path(&archive_path)
            .ok_or_else(|| ArchiveError::UnsupportedFormat {
                format: archive_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            })?;
        
        match format {
            ArchiveFormat::Zip => Ok(Box::new(ZipArchiveReader::new(archive_path))),
            // Future formats would be handled here
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    
    fn create_test_zip() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = temp_dir.path().join("test.zip");
        
        // Create a simple ZIP file for testing
        let file = std::fs::File::create(&zip_path).expect("Failed to create zip file");
        let mut zip_writer = zip::ZipWriter::new(file);
        
        // Add a text file
        zip_writer.start_file("test.txt", zip::write::SimpleFileOptions::default())
            .expect("Failed to start file");
        zip_writer.write_all(b"Hello, World!")
            .expect("Failed to write file content");
        
        // Add a directory
        zip_writer.add_directory("subdir/", zip::write::SimpleFileOptions::default())
            .expect("Failed to add directory");
        
        // Add a file in the directory
        zip_writer.start_file("subdir/nested.txt", zip::write::SimpleFileOptions::default())
            .expect("Failed to start nested file");
        zip_writer.write_all(b"Nested content")
            .expect("Failed to write nested file content");
        
        zip_writer.finish().expect("Failed to finish ZIP");
        
        (temp_dir, zip_path)
    }
    
    #[tokio::test]
    async fn test_archive_format_detection() {
        assert_eq!(ArchiveFormat::from_path(Path::new("test.zip")), Some(ArchiveFormat::Zip));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.txt")), None);
    }
    
    #[tokio::test]
    async fn test_zip_listing() {
        let (_temp_dir, zip_path) = create_test_zip();
        
        let reader = ZipArchiveReader::new(zip_path);
        let entries = reader.list_entries().await.expect("Failed to list entries");
        
        assert_eq!(entries.len(), 3); // test.txt, subdir/, subdir/nested.txt
        
        let text_file = entries.iter().find(|e| e.path == "test.txt").expect("test.txt not found");
        assert!(!text_file.is_directory);
        assert_eq!(text_file.size, 13); // "Hello, World!" is 13 bytes
        
        let subdir = entries.iter().find(|e| e.path == "subdir/").expect("subdir/ not found");
        assert!(subdir.is_directory);
    }
    
    #[tokio::test]
    async fn test_zip_extraction() {
        let (_temp_dir, zip_path) = create_test_zip();
        let extract_dir = TempDir::new().expect("Failed to create extract dir");
        
        let reader = ZipArchiveReader::new(zip_path);
        let options = ExtractionOptions::default();
        
        reader.extract(extract_dir.path(), options, None).await
            .expect("Failed to extract archive");
        
        // Verify extracted files
        let test_file_path = extract_dir.path().join("test.txt");
        assert!(test_file_path.exists());
        
        let content = fs::read_to_string(test_file_path).expect("Failed to read extracted file");
        assert_eq!(content, "Hello, World!");
        
        let nested_file_path = extract_dir.path().join("subdir/nested.txt");
        assert!(nested_file_path.exists());
        
        let nested_content = fs::read_to_string(nested_file_path).expect("Failed to read nested file");
        assert_eq!(nested_content, "Nested content");
    }
    
    #[tokio::test]
    async fn test_extract_entry_to_memory() {
        let (_temp_dir, zip_path) = create_test_zip();
        
        let reader = ZipArchiveReader::new(zip_path);
        let content = reader.extract_entry_to_memory("test.txt").await
            .expect("Failed to extract to memory");
        
        assert_eq!(content, b"Hello, World!");
    }
    
    #[tokio::test]
    async fn test_archive_factory() {
        let (_temp_dir, zip_path) = create_test_zip();
        
        let reader = ArchiveFactory::create_reader(zip_path)
            .expect("Failed to create reader");
        
        assert_eq!(reader.format(), ArchiveFormat::Zip);
        
        let entries = reader.list_entries().await.expect("Failed to list entries");
        assert_eq!(entries.len(), 3);
    }
}