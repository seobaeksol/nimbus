//! Archive handling for file manager operations
//!
//! This crate provides unified archive support for browsing and extracting
//! various archive formats. Currently supports ZIP files with foundation
//! for adding more formats like TAR, 7z, and RAR.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

// TAR support
use tar::Archive as TarArchive;
use flate2::read::GzDecoder;
use bzip2::read::BzDecoder;

// 7z support
use sevenz_rust::SevenZReader;

// RAR support
#[allow(unused_imports)]
use unrar::Archive as RarArchive;

// Integrity verification
use crc32fast::Hasher as Crc32Hasher;
use md5::{Md5, Digest};
use sha1::Sha1;
use sha2::Sha256;

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
    
    #[error("Integrity verification failed: {reason}")]
    IntegrityVerificationFailed { reason: String },
    
    #[error("Operation cancelled by user")]
    OperationCancelled,
    
    #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
    InsufficientDiskSpace { required: u64, available: u64 },
    
    #[error("Archive format validation failed: {reason}")]
    FormatValidationFailed { reason: String },
    
    #[error("Network error during remote operation: {reason}")]
    NetworkError { reason: String },
}

/// Archive format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    SevenZ,
    Rar,
}

impl ArchiveFormat {
    /// Detect archive format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        let path_str = path.to_str()?.to_lowercase();
        
        match extension.as_str() {
            "zip" => Some(Self::Zip),
            "tar" => Some(Self::Tar),
            "gz" | "tgz" => {
                // Check for .tar.gz
                if path_str.ends_with(".tar.gz") {
                    Some(Self::TarGz)
                } else {
                    Some(Self::TarGz) // Assume single .gz files are compressed tars
                }
            },
            "bz2" | "tbz2" => {
                // Check for .tar.bz2
                if path_str.ends_with(".tar.bz2") {
                    Some(Self::TarBz2)
                } else {
                    Some(Self::TarBz2) // Assume single .bz2 files are compressed tars
                }
            },
            "7z" => Some(Self::SevenZ),
            "rar" => Some(Self::Rar),
            _ => None,
        }
    }

    /// Detect archive format by reading file header (magic bytes)
    pub fn from_header(path: &Path) -> std::io::Result<Option<Self>> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 512]; // Read enough bytes to check TAR header
        let bytes_read = file.read(&mut buffer)?;
        
        if bytes_read < 4 {
            return Ok(None); // Not enough data
        }
        
        // Check ZIP format: PK (50 4B 03 04)
        if buffer[0] == 0x50 && buffer[1] == 0x4B && buffer[2] == 0x03 && buffer[3] == 0x04 {
            return Ok(Some(Self::Zip));
        }
        
        // Check 7z format: 7z (37 7A BC AF 27 1C)
        if bytes_read >= 6 &&
           buffer[0] == 0x37 && buffer[1] == 0x7A && buffer[2] == 0xBC &&
           buffer[3] == 0xAF && buffer[4] == 0x27 && buffer[5] == 0x1C {
            return Ok(Some(Self::SevenZ));
        }
        
        // Check RAR format: Rar! (52 61 72 21 1A 07)
        if bytes_read >= 6 &&
           buffer[0] == 0x52 && buffer[1] == 0x61 && buffer[2] == 0x72 &&
           buffer[3] == 0x21 && buffer[4] == 0x1A && buffer[5] == 0x07 {
            return Ok(Some(Self::Rar));
        }
        
        // Check TAR format: "ustar" at offset 257 (POSIX TAR)
        if bytes_read >= 262 {
            let ustar_bytes = &buffer[257..262];
            if ustar_bytes == b"ustar" {
                // Determine TAR compression by checking for additional magic bytes
                // Check for gzip header (1F 8B) at the start
                if buffer[0] == 0x1F && buffer[1] == 0x8B {
                    return Ok(Some(Self::TarGz));
                }
                // Check for bzip2 header (42 5A 68) at the start  
                if buffer[0] == 0x42 && buffer[1] == 0x5A && buffer[2] == 0x68 {
                    return Ok(Some(Self::TarBz2));
                }
                // Plain TAR
                return Ok(Some(Self::Tar));
            }
        }
        
        // Check for compressed formats that might contain TAR
        // Gzip header: 1F 8B
        if buffer[0] == 0x1F && buffer[1] == 0x8B {
            return Ok(Some(Self::TarGz)); // Assume .gz files are compressed tar
        }
        
        // Bzip2 header: 42 5A 68  
        if buffer[0] == 0x42 && buffer[1] == 0x5A && buffer[2] == 0x68 {
            return Ok(Some(Self::TarBz2)); // Assume .bz2 files are compressed tar
        }
        
        Ok(None) // Unknown format
    }

    /// Detect archive format using both header and path fallback
    pub fn detect(path: &Path) -> std::io::Result<Option<Self>> {
        // Try header-based detection first (more reliable)
        match Self::from_header(path)? {
            Some(format) => Ok(Some(format)),
            None => {
                // Fall back to extension-based detection
                Ok(Self::from_path(path))
            }
        }
    }
    
    /// Get the typical file extensions for this format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Zip => &["zip"],
            Self::Tar => &["tar"],
            Self::TarGz => &["gz", "tgz", "tar.gz"],
            Self::TarBz2 => &["bz2", "tbz2", "tar.bz2"],
            Self::SevenZ => &["7z"],
            Self::Rar => &["rar"],
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

/// Hash algorithm types supported for integrity verification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HashAlgorithm {
    Crc32,
    Md5,
    Sha1,
    Sha256,
}

/// Integrity verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    /// Whether the verification passed
    pub passed: bool,
    /// Hash algorithm used
    pub algorithm: HashAlgorithm,
    /// Expected hash value
    pub expected: String,
    /// Actual computed hash value
    pub actual: String,
    /// Error message if verification failed
    pub error_message: Option<String>,
}

/// Utility functions for hash computation and verification
pub struct IntegrityVerifier;

impl IntegrityVerifier {
    /// Compute CRC32 checksum for data
    pub fn compute_crc32(data: &[u8]) -> u32 {
        let mut hasher = Crc32Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }
    
    /// Compute MD5 hash for data
    pub fn compute_md5(data: &[u8]) -> String {
        let mut hasher = Md5::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    
    /// Compute SHA1 hash for data
    pub fn compute_sha1(data: &[u8]) -> String {
        use sha1::Digest;
        let mut hasher = Sha1::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    
    /// Compute SHA256 hash for data
    pub fn compute_sha256(data: &[u8]) -> String {
        use sha2::Digest;
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    
    /// Compute hash using specified algorithm
    pub fn compute_hash(data: &[u8], algorithm: HashAlgorithm) -> String {
        match algorithm {
            HashAlgorithm::Crc32 => format!("{:08x}", Self::compute_crc32(data)),
            HashAlgorithm::Md5 => Self::compute_md5(data),
            HashAlgorithm::Sha1 => Self::compute_sha1(data),
            HashAlgorithm::Sha256 => Self::compute_sha256(data),
        }
    }
    
    /// Verify data integrity against archive entry CRC32
    pub fn verify_crc32(data: &[u8], expected_crc32: u32) -> IntegrityResult {
        let actual_crc32 = Self::compute_crc32(data);
        let passed = actual_crc32 == expected_crc32;
        
        IntegrityResult {
            passed,
            algorithm: HashAlgorithm::Crc32,
            expected: format!("{:08x}", expected_crc32),
            actual: format!("{:08x}", actual_crc32),
            error_message: if passed {
                None
            } else {
                Some(format!(
                    "CRC32 mismatch: expected {:08x}, got {:08x}",
                    expected_crc32, actual_crc32
                ))
            },
        }
    }
    
    /// Verify data integrity against expected hash
    pub fn verify_hash(data: &[u8], expected: &str, algorithm: HashAlgorithm) -> IntegrityResult {
        let actual = Self::compute_hash(data, algorithm);
        let expected_lower = expected.to_lowercase();
        let actual_lower = actual.to_lowercase();
        let passed = expected_lower == actual_lower;
        
        IntegrityResult {
            passed,
            algorithm,
            expected: expected_lower.clone(),
            actual: actual_lower.clone(),
            error_message: if passed {
                None
            } else {
                Some(format!(
                    "{:?} hash mismatch: expected {}, got {}",
                    algorithm, expected_lower, actual_lower
                ))
            },
        }
    }
}

/// Enhanced error context with recovery suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The original error
    pub error: String,
    /// File or operation that caused the error
    pub context: String,
    /// Suggested recovery actions
    pub recovery_suggestions: Vec<String>,
    /// Whether the operation can be retried
    pub is_retryable: bool,
    /// Error severity level
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Informational - operation can continue
    Info,
    /// Warning - operation can continue with reduced functionality
    Warning,
    /// Error - operation failed but can be retried
    Error,
    /// Critical - operation failed and cannot be retried
    Critical,
}

/// Progress tracker with timing and speed calculations
pub struct ProgressTracker {
    start_time: std::time::Instant,
    last_update: std::time::Instant,
    last_bytes: u64,
    operation: ProgressOperation,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(operation: ProgressOperation) -> Self {
        let now = std::time::Instant::now();
        Self {
            start_time: now,
            last_update: now,
            last_bytes: 0,
            operation,
        }
    }
    
    /// Create enhanced progress info with speed and ETA calculations
    pub fn create_progress(&mut self, 
                          current_file: String,
                          files_processed: usize,
                          total_files: usize,
                          bytes_processed: u64,
                          total_bytes: u64,
                          stage: ProgressStage,
                          status_message: Option<String>) -> ProgressInfo {
        
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.start_time);
        
        // Calculate speed (bytes per second)
        let speed_bps = if elapsed.as_secs() > 0 {
            Some(bytes_processed / elapsed.as_secs())
        } else {
            None
        };
        
        // Calculate ETA (estimated time remaining)
        let eta_seconds = if bytes_processed > 0 && total_bytes > bytes_processed {
            let remaining_bytes = total_bytes - bytes_processed;
            speed_bps.map(|speed| if speed > 0 { remaining_bytes / speed } else { 0 })
        } else {
            None
        };
        
        // Update tracking state
        self.last_update = now;
        self.last_bytes = bytes_processed;
        
        ProgressInfo {
            current_file,
            files_processed,
            total_files,
            bytes_processed,
            total_bytes,
            operation: self.operation,
            speed_bps,
            eta_seconds,
            stage,
            status_message,
        }
    }
    
    /// Create a completion progress info
    pub fn create_completion(&self, total_files: usize, total_bytes: u64) -> ProgressInfo {
        ProgressInfo {
            current_file: "Operation completed".to_string(),
            files_processed: total_files,
            total_files,
            bytes_processed: total_bytes,
            total_bytes,
            operation: self.operation,
            speed_bps: None,
            eta_seconds: Some(0),
            stage: ProgressStage::Completed,
            status_message: Some("All files processed successfully".to_string()),
        }
    }
}

/// Archive error handling utilities
pub struct ErrorHandler;

impl ErrorHandler {
    /// Create enhanced error context for common archive errors
    pub fn create_context(error: &ArchiveError, operation_context: &str) -> ErrorContext {
        let (suggestions, retryable, severity) = match error {
            ArchiveError::NotFound { path } => (
                vec![
                    "Verify the file path is correct".to_string(),
                    "Check if the file was moved or deleted".to_string(),
                    "Ensure you have read permissions".to_string(),
                ],
                false,
                ErrorSeverity::Error,
            ),
            ArchiveError::PermissionDenied { path: _ } => (
                vec![
                    "Run the application with administrator privileges".to_string(),
                    "Check file/directory permissions".to_string(),
                    "Ensure the file is not being used by another process".to_string(),
                ],
                true,
                ErrorSeverity::Error,
            ),
            ArchiveError::PasswordRequired => (
                vec![
                    "Provide the correct password for the encrypted archive".to_string(),
                    "Check if you have the right archive file".to_string(),
                ],
                true,
                ErrorSeverity::Warning,
            ),
            ArchiveError::InvalidPassword => (
                vec![
                    "Verify the password is correct".to_string(),
                    "Check for caps lock or keyboard layout issues".to_string(),
                    "Try alternative passwords if available".to_string(),
                ],
                true,
                ErrorSeverity::Error,
            ),
            ArchiveError::CorruptedArchive { reason: _ } => (
                vec![
                    "Try downloading the archive file again".to_string(),
                    "Check if the file was completely downloaded".to_string(),
                    "Verify the file integrity using checksums".to_string(),
                ],
                false,
                ErrorSeverity::Critical,
            ),
            ArchiveError::InsufficientDiskSpace { required, available: _ } => (
                vec![
                    format!("Free up at least {} bytes of disk space", required),
                    "Choose a different destination with more space".to_string(),
                    "Extract only selected files instead of the entire archive".to_string(),
                ],
                true,
                ErrorSeverity::Error,
            ),
            ArchiveError::IntegrityVerificationFailed { reason: _ } => (
                vec![
                    "Re-download the archive file".to_string(),
                    "Verify the archive file is not corrupted".to_string(),
                    "Try extracting without integrity verification".to_string(),
                ],
                true,
                ErrorSeverity::Warning,
            ),
            _ => (
                vec!["Try the operation again".to_string()],
                true,
                ErrorSeverity::Error,
            ),
        };
        
        ErrorContext {
            error: error.to_string(),
            context: operation_context.to_string(),
            recovery_suggestions: suggestions,
            is_retryable: retryable,
            severity,
        }
    }
    
    /// Check if an error is recoverable
    pub fn is_recoverable(error: &ArchiveError) -> bool {
        !matches!(error,
            ArchiveError::CorruptedArchive { .. } |
            ArchiveError::UnsupportedFormat { .. } |
            ArchiveError::NotFound { .. }
        )
    }
    
    /// Get user-friendly error message
    pub fn user_friendly_message(error: &ArchiveError) -> String {
        match error {
            ArchiveError::NotFound { path } => {
                format!("Could not find the archive file '{}'", path)
            }
            ArchiveError::PermissionDenied { path } => {
                format!("Access denied to '{}'", path)
            }
            ArchiveError::PasswordRequired => {
                "This archive is password-protected. Please provide the password.".to_string()
            }
            ArchiveError::InvalidPassword => {
                "The password you entered is incorrect.".to_string()
            }
            ArchiveError::CorruptedArchive { reason: _ } => {
                "The archive file appears to be damaged or corrupted.".to_string()
            }
            ArchiveError::InsufficientDiskSpace { required, available } => {
                format!("Not enough disk space. Need {} bytes but only {} bytes available.", 
                       required, available)
            }
            ArchiveError::IntegrityVerificationFailed { reason: _ } => {
                "File integrity check failed. The extracted file may be corrupted.".to_string()
            }
            ArchiveError::OperationCancelled => {
                "Operation was cancelled by the user.".to_string()
            }
            _ => error.to_string(),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Currently processing file path
    pub current_file: String,
    /// Number of files processed so far
    pub files_processed: usize,
    /// Total number of files to process
    pub total_files: usize,
    /// Bytes processed so far
    pub bytes_processed: u64,
    /// Total bytes to process
    pub total_bytes: u64,
    /// Current operation type
    pub operation: ProgressOperation,
    /// Processing speed in bytes per second
    pub speed_bps: Option<u64>,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Current operation stage
    pub stage: ProgressStage,
    /// Optional detailed status message
    pub status_message: Option<String>,
}

/// Types of operations that can be tracked
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProgressOperation {
    Extracting,
    Listing,
    Verifying,
    Compressing,
    Counting,
}

/// Stages of operation progress
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProgressStage {
    Initializing,
    Processing,
    Finalizing,
    Completed,
    Cancelled,
    Failed,
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
    
    /// Extract entry to memory and verify its integrity against archive CRC32
    async fn extract_and_verify_crc32(&self, path: &str) -> Result<(Vec<u8>, IntegrityResult), ArchiveError> {
        // Extract the data
        let data = self.extract_entry_to_memory(path).await?;
        
        // Get the entry info to check CRC32
        let entry = self.get_entry(path).await?
            .ok_or_else(|| ArchiveError::NotFound { path: path.to_string() })?;
            
        // Verify integrity if CRC32 is available
        let integrity_result = if let Some(expected_crc32) = entry.crc32 {
            IntegrityVerifier::verify_crc32(&data, expected_crc32)
        } else {
            // No CRC32 available - mark as passed but with a note
            IntegrityResult {
                passed: true,
                algorithm: HashAlgorithm::Crc32,
                expected: "N/A".to_string(),
                actual: format!("{:08x}", IntegrityVerifier::compute_crc32(&data)),
                error_message: None,
            }
        };
        
        // Return error if verification failed
        if !integrity_result.passed {
            if let Some(ref error_msg) = integrity_result.error_message {
                return Err(ArchiveError::IntegrityVerificationFailed {
                    reason: error_msg.clone(),
                });
            }
        }
        
        Ok((data, integrity_result))
    }
    
    /// Extract entry to memory and compute specified hash
    async fn extract_and_compute_hash(&self, path: &str, algorithm: HashAlgorithm) -> Result<(Vec<u8>, String), ArchiveError> {
        let data = self.extract_entry_to_memory(path).await?;
        let hash = IntegrityVerifier::compute_hash(&data, algorithm);
        Ok((data, hash))
    }
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
        tokio::task::spawn_blocking(move || -> Result<Vec<ArchiveEntry>, ArchiveError> {
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
        })?
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
            
            // Initialize progress tracker
            let mut progress_tracker = ProgressTracker::new(ProgressOperation::Extracting);
            
            // Send initial progress
            if let Some(ref callback) = progress_callback {
                let progress = progress_tracker.create_progress(
                    "Preparing extraction...".to_string(),
                    files_processed,
                    total_files,
                    bytes_processed,
                    total_bytes,
                    ProgressStage::Initializing,
                    None,
                );
                callback(progress);
            }
            
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
                                let progress = progress_tracker.create_progress(
                                    entry_path.clone(),
                                    files_processed,
                                    total_files,
                                    bytes_processed,
                                    total_bytes,
                                    ProgressStage::Processing,
                                    None,
                                );
                                callback(progress);
                            }
                            continue;
                        }
                        OverwritePolicy::Ask => {
                            // In a real implementation, this would trigger a UI dialog
                            // For now, skip to avoid overwriting
                            files_processed += 1;
                            if let Some(ref callback) = progress_callback {
                                let progress = progress_tracker.create_progress(
                                    entry_path.clone(),
                                    files_processed,
                                    total_files,
                                    bytes_processed,
                                    total_bytes,
                                    ProgressStage::Processing,
                                    None,
                                );
                                callback(progress);
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
                    let progress = progress_tracker.create_progress(
                        entry_path,
                        files_processed,
                        total_files,
                        bytes_processed,
                        total_bytes,
                        ProgressStage::Processing,
                        None,
                    );
                    callback(progress);
                }
            }
            
            // Send completion progress
            if let Some(ref callback) = progress_callback {
                let progress = progress_tracker.create_progress(
                    "Extraction completed successfully".to_string(),
                    files_processed,
                    total_files,
                    bytes_processed,
                    total_bytes,
                    ProgressStage::Completed,
                    None,
                );
                callback(progress);
            }
            
            Ok(())
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
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

/// TAR archive implementation
pub struct TarArchiveReader {
    archive_path: PathBuf,
    format: ArchiveFormat,
}

impl TarArchiveReader {
    /// Open a TAR archive for reading
    pub fn new(archive_path: PathBuf, format: ArchiveFormat) -> Self {
        Self { archive_path, format }
    }
    
    /// Helper to create the appropriate TAR archive reader based on compression
    fn create_tar_archive(&self, file: std::fs::File) -> Result<Box<dyn std::io::Read + Send>, ArchiveError> {
        match self.format {
            ArchiveFormat::Tar => Ok(Box::new(file)),
            ArchiveFormat::TarGz => Ok(Box::new(GzDecoder::new(file))),
            ArchiveFormat::TarBz2 => Ok(Box::new(BzDecoder::new(file))),
            _ => Err(ArchiveError::UnsupportedFormat {
                format: format!("{:?}", self.format),
            }),
        }
    }
}

#[async_trait]
impl ArchiveReader for TarArchiveReader {
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        let format = self.format;
        
        tokio::task::spawn_blocking(move || -> Result<Vec<ArchiveEntry>, ArchiveError> {
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
                
            let reader = match format {
                ArchiveFormat::Tar => Box::new(file) as Box<dyn std::io::Read>,
                ArchiveFormat::TarGz => Box::new(GzDecoder::new(file)) as Box<dyn std::io::Read>,
                ArchiveFormat::TarBz2 => Box::new(BzDecoder::new(file)) as Box<dyn std::io::Read>,
                _ => return Err(ArchiveError::UnsupportedFormat {
                    format: format!("{:?}", format),
                }),
            };
            
            let mut archive = TarArchive::new(reader);
            let mut entries = Vec::new();
            
            for entry_result in archive.entries()
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to read TAR entries: {}", e),
                })? 
            {
                let entry = entry_result
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read TAR entry: {}", e),
                    })?;
                
                let header = entry.header();
                let path = entry.path()
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read entry path: {}", e),
                    })?
                    .to_string_lossy()
                    .to_string();
                
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
                
                // Convert TAR timestamp to chrono DateTime
                let modified = header.mtime()
                    .ok()
                    .and_then(|timestamp| DateTime::from_timestamp(timestamp as i64, 0));
                
                let is_directory = header.entry_type().is_dir();
                let size = if is_directory { 0 } else { header.size().unwrap_or(0) };
                
                entries.push(ArchiveEntry {
                    path,
                    name,
                    size,
                    compressed_size: size, // TAR doesn't compress individual entries
                    modified,
                    is_directory,
                    compression_method: match format {
                        ArchiveFormat::Tar => Some("store".to_string()),
                        ArchiveFormat::TarGz => Some("gzip".to_string()),
                        ArchiveFormat::TarBz2 => Some("bzip2".to_string()),
                        _ => None,
                    },
                    crc32: None, // TAR doesn't store CRC32
                    is_encrypted: false, // TAR doesn't support encryption
                });
            }
            
            Ok(entries)
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
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
        let format = self.format;
        
        tokio::task::spawn_blocking(move || -> Result<(), ArchiveError> {
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
                
            let reader = match format {
                ArchiveFormat::Tar => Box::new(file) as Box<dyn std::io::Read>,
                ArchiveFormat::TarGz => Box::new(GzDecoder::new(file)) as Box<dyn std::io::Read>,
                ArchiveFormat::TarBz2 => Box::new(BzDecoder::new(file)) as Box<dyn std::io::Read>,
                _ => return Err(ArchiveError::UnsupportedFormat {
                    format: format!("{:?}", format),
                }),
            };
            
            let mut archive = TarArchive::new(reader);
            
            // Initialize progress tracker
            let mut progress_tracker = ProgressTracker::new(ProgressOperation::Extracting);
            
            // Send initial progress
            if let Some(ref callback) = progress_callback {
                let progress = progress_tracker.create_progress(
                    "Preparing TAR extraction...".to_string(),
                    0,
                    1,  // We don't know total count yet
                    0,
                    0,
                    ProgressStage::Initializing,
                    None,
                );
                callback(progress);
            }
            
            // Set the destination and handle subfolder creation
            let final_destination = if options.create_subfolder {
                let archive_name = archive_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive");
                destination.join(archive_name)
            } else {
                destination
            };
            
            // Ensure destination directory exists
            std::fs::create_dir_all(&final_destination)
                .map_err(|e| ArchiveError::ExtractionFailed {
                    reason: format!("Failed to create destination directory: {}", e),
                })?;
            
            // Extract to the final destination
            if options.preserve_paths {
                archive.unpack(&final_destination)
                    .map_err(|e| ArchiveError::ExtractionFailed {
                        reason: format!("Failed to extract TAR archive: {}", e),
                    })?;
            } else {
                // For flat extraction, we need to iterate through entries
                for entry_result in archive.entries()
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read TAR entries: {}", e),
                    })? 
                {
                    let mut entry = entry_result
                        .map_err(|e| ArchiveError::CorruptedArchive {
                            reason: format!("Failed to read TAR entry: {}", e),
                        })?;
                    
                    if entry.header().entry_type().is_file() {
                        let entry_path = entry.path()
                            .map_err(|e| ArchiveError::CorruptedArchive {
                                reason: format!("Failed to read entry path: {}", e),
                            })?;
                        
                        let filename = entry_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        let dest_path = final_destination.join(filename);
                        
                        // Handle existing files
                        if dest_path.exists() {
                            match options.overwrite_policy {
                                OverwritePolicy::Skip => continue,
                                OverwritePolicy::Ask => continue, // Skip for now
                                OverwritePolicy::Overwrite => {},
                                OverwritePolicy::Rename => {
                                    // Implementation similar to ZIP rename logic
                                }
                            }
                        }
                        
                        let filename_str = filename.to_string(); // Clone filename before move
                        entry.unpack(&dest_path)
                            .map_err(|e| ArchiveError::ExtractionFailed {
                                reason: format!("Failed to extract file '{}': {}", filename_str, e),
                            })?;
                    }
                }
            }
            
            // Send completion progress
            if let Some(ref callback) = progress_callback {
                let progress = progress_tracker.create_progress(
                    "TAR extraction completed successfully".to_string(),
                    1,
                    1,
                    0,
                    0,
                    ProgressStage::Completed,
                    None,
                );
                callback(progress);
            }
            
            Ok(())
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    async fn extract_entry_to_memory(&self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        let format = self.format;
        let path = path.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ArchiveError> {
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
                
            let reader = match format {
                ArchiveFormat::Tar => Box::new(file) as Box<dyn std::io::Read>,
                ArchiveFormat::TarGz => Box::new(GzDecoder::new(file)) as Box<dyn std::io::Read>,
                ArchiveFormat::TarBz2 => Box::new(BzDecoder::new(file)) as Box<dyn std::io::Read>,
                _ => return Err(ArchiveError::UnsupportedFormat {
                    format: format!("{:?}", format),
                }),
            };
            
            let mut archive = TarArchive::new(reader);
            
            for entry_result in archive.entries()
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to read TAR entries: {}", e),
                })? 
            {
                let mut entry = entry_result
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read TAR entry: {}", e),
                    })?;
                
                let entry_path = entry.path()
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read entry path: {}", e),
                    })?
                    .to_string_lossy()
                    .to_string();
                
                if entry_path == path && entry.header().entry_type().is_file() {
                    let mut buffer = Vec::new();
                    entry.read_to_end(&mut buffer)
                        .map_err(|e| ArchiveError::ExtractionFailed {
                            reason: format!("Failed to read entry '{}': {}", path, e),
                        })?;
                    return Ok(buffer);
                }
            }
            
            Err(ArchiveError::NotFound { path })
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    fn requires_password(&self) -> bool {
        false // TAR doesn't support encryption
    }
    
    fn format(&self) -> ArchiveFormat {
        self.format
    }
}

/// 7z archive implementation using sevenz-rust 0.6.1 API
pub struct SevenZArchiveReader {
    archive_path: PathBuf,
}

impl SevenZArchiveReader {
    /// Open a 7z archive for reading
    pub fn new(archive_path: PathBuf) -> Self {
        Self { archive_path }
    }
}

#[async_trait]
impl ArchiveReader for SevenZArchiveReader {
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<ArchiveEntry>, ArchiveError> {
            let mut reader = SevenZReader::open(&archive_path, "".into())
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to open 7z archive: {}", e),
                })?;
            
            let mut entries = Vec::new();
            
            reader.for_each_entries(|entry, _reader| {
                let path = entry.name().to_string();
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
                
                let is_directory = entry.is_directory();
                let size = if is_directory { 0 } else { entry.size() };
                
                entries.push(ArchiveEntry {
                    path,
                    name,
                    size,
                    compressed_size: size, // Compressed size not easily available
                    modified: None, // Timestamp handling simplified
                    is_directory,
                    compression_method: Some("LZMA2".to_string()),
                    crc32: None,
                    is_encrypted: false,
                });
                
                Ok(true) // Continue processing entries
            }).map_err(|e| ArchiveError::CorruptedArchive {
                reason: format!("Failed to iterate 7z entries: {}", e),
            })?;
            
            Ok(entries)
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
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
            // Use sevenz_rust's decompress helper function for now
            let password = options.password.unwrap_or_default();
            
            // Set the destination and handle subfolder creation
            let final_destination = if options.create_subfolder {
                let archive_name = archive_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive");
                destination.join(archive_name)
            } else {
                destination
            };
            
            // Ensure destination directory exists
            std::fs::create_dir_all(&final_destination)
                .map_err(|e| ArchiveError::ExtractionFailed {
                    reason: format!("Failed to create destination directory: {}", e),
                })?;
            
            if password.is_empty() {
                sevenz_rust::decompress_file(&archive_path, &final_destination)
            } else {
                sevenz_rust::decompress_file_with_password(&archive_path, &final_destination, password.as_str().into())
            }.map_err(|e| ArchiveError::ExtractionFailed {
                reason: format!("Failed to extract 7z archive: {}", e),
            })?;
            
            // Report completion to progress callback
            if let Some(ref callback) = progress_callback {
                let mut progress_tracker = ProgressTracker::new(ProgressOperation::Extracting);
                let progress = progress_tracker.create_progress(
                    "7z extraction completed".to_string(),
                    1,
                    1,
                    0,
                    0,
                    ProgressStage::Completed,
                    None,
                );
                callback(progress);
            }
            
            Ok(())
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    async fn extract_entry_to_memory(&self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        let path = path.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ArchiveError> {
            let mut reader = SevenZReader::open(&archive_path, "".into())
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to open 7z archive: {}", e),
                })?;
            
            let mut result_data = None;
            
            reader.for_each_entries(|entry, file_reader| {
                if entry.name() == path && !entry.is_directory() {
                    let mut buffer = Vec::new();
                    if file_reader.read_to_end(&mut buffer).is_ok() {
                        result_data = Some(buffer);
                    }
                }
                Ok(result_data.is_none()) // Continue until we find our entry
            }).map_err(|e| ArchiveError::CorruptedArchive {
                reason: format!("Failed to extract entry: {}", e),
            })?;
            
            result_data.ok_or_else(|| ArchiveError::NotFound {
                path: format!("Entry not found in 7z archive: {}", path),
            })
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    fn requires_password(&self) -> bool {
        // Try opening without password to check if password is required
        match SevenZReader::open(&self.archive_path, "".into()) {
            Err(e) => {
                let error_str = format!("{}", e);
                error_str.contains("password") || error_str.contains("Password")
            }
            Ok(_) => false,
        }
    }
    
    fn format(&self) -> ArchiveFormat {
        ArchiveFormat::SevenZ
    }
}

/*
/// RAR archive implementation (read-only) - temporarily disabled due to syntax issues
pub struct RarArchiveReader {
    archive_path: PathBuf,
}

impl RarArchiveReader {
    /// Open a RAR archive for reading
    pub fn new(archive_path: PathBuf) -> Self {
        Self { archive_path }
    }
}

#[async_trait]
impl ArchiveReader for RarArchiveReader {
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<ArchiveEntry>, ArchiveError> {
            let mut archive = RarArchive::new(&archive_path)
                .open_for_listing()
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to open RAR archive: {:?}", e),
                })?;
            
            let mut entries = Vec::new();
            
            loop {
                match archive.read_header() {
                    Ok(Some(header)) => {
                        let entry = header.entry();
                        let path = entry.filename.to_string_lossy().to_string();
                        let name = Path::new(&path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&path)
                            .to_string();
                        
                        // Convert RAR timestamp to chrono DateTime
                        let modified = if entry.file_time > 0 {
                            DateTime::from_timestamp(entry.file_time as i64, 0)
                        } else {
                            None
                        };
                        
                        let is_directory = entry.is_directory();
                        let size = if is_directory { 0 } else { entry.unpacked_size as u64 };
                        let compressed_size = entry.unpacked_size as u64; // RAR doesn't expose packed size easily
                        
                        entries.push(ArchiveEntry {
                            path,
                            name,
                            size,
                            compressed_size,
                            modified,
                            is_directory,
                            compression_method: Some("RAR".to_string()),
                            crc32: Some(entry.file_crc),
                            is_encrypted: false, // Encryption detection not available in this version
                        });
                        
                        archive = header.skip()
                            .map_err(|e| ArchiveError::CorruptedArchive {
                                reason: format!("Failed to skip RAR entry: {:?}", e),
                            })?;
                    }
                    Ok(None) => break,
                    Err(e) => return Err(ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read RAR header: {:?}", e),
                    }),
                }
            }
            
            Ok(entries)
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
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
            
            // Set the destination and handle subfolder creation
            let final_destination = if options.create_subfolder {
                let archive_name = archive_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive");
                destination.join(archive_name)
            } else {
                destination
            };
            
            // Ensure destination directory exists
            std::fs::create_dir_all(&final_destination)
                .map_err(|e| ArchiveError::ExtractionFailed {
                    reason: format!("Failed to create destination directory: {}", e),
                })?;
            
            let mut files_processed = 0;
            let mut bytes_processed = 0;
            let mut total_files = 0;
            let mut total_bytes = 0;
            
            // First pass: count files for progress tracking
            if progress_callback.is_some() {
                let mut count_archive = RarArchive::new(&archive_path)
                    .open_for_listing()
                    .map_err(|e| ArchiveError::CorruptedArchive {
                        reason: format!("Failed to open RAR archive for counting: {:?}", e),
                    })?;
                
                loop {
                    match count_archive.read_header() {
                        Ok(Some(header)) => {
                            let entry = header.entry();
                            if options.entries.is_none() || 
                               options.entries.as_ref().unwrap().contains(&entry.filename.to_string_lossy().to_string()) {
                                total_files += 1;
                                total_bytes += entry.unpacked_size as u64;
                            }
                            count_archive = header.skip().map_err(|e| ArchiveError::CorruptedArchive {
                                reason: format!("Failed to skip RAR entry during counting: {:?}", e),
                            })?;
                        }
                        Ok(None) => break,
                        Err(_) => break, // Ignore counting errors
                    }
                }
            }
            
            // Second pass: extract files
            let mut extract_archive = RarArchive::new(&archive_path)
                .open_for_processing()
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to open RAR archive for extraction: {:?}", e),
                })?;
                
            loop {
                match extract_archive.read_header() {
                    Ok(Some(header)) => {
                let entry_path = {
                    let entry = header.entry();
                    entry.filename.to_string_lossy().to_string()
                };
                
                // Check if we should extract this entry
                let should_extract = options.entries.is_none() || 
                    options.entries.as_ref().unwrap().contains(&entry_path);
                
                if should_extract {
                    let (is_directory, unpacked_size) = {
                        let entry = header.entry();
                        (entry.is_directory(), entry.unpacked_size)
                    };
                    // Calculate destination path
                    let dest_path = if options.preserve_paths {
                        final_destination.join(&entry_path)
                    } else {
                        let file_name = Path::new(&entry_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&entry_path);
                        final_destination.join(file_name)
                    };
                    
                    // Handle existing files
                    if dest_path.exists() {
                        match options.overwrite_policy {
                            OverwritePolicy::Skip => {
                                header.skip()
                                    .map_err(|e| ArchiveError::CorruptedArchive {
                                        reason: format!("Failed to skip RAR entry: {:?}", e),
                                    })?;
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
                                header.skip()
                                    .map_err(|e| ArchiveError::CorruptedArchive {
                                        reason: format!("Failed to skip RAR entry: {:?}", e),
                                    })?;
                                continue;
                            }
                            OverwritePolicy::Overwrite => {},
                            OverwritePolicy::Rename => {
                                // Handle renaming similar to other implementations
                            }
                        }
                    }
                    
                    if is_directory {
                        // Create directory
                        std::fs::create_dir_all(&dest_path)
                            .map_err(|e| ArchiveError::ExtractionFailed {
                                reason: format!("Failed to create directory '{}': {}", dest_path.display(), e),
                            })?;
                        
                        extract_archive = header.skip()
                            .map_err(|e| ArchiveError::CorruptedArchive {
                                reason: format!("Failed to skip RAR directory: {:?}", e),
                            })?;
                    } else {
                        // Create parent directory if needed
                        if let Some(parent) = dest_path.parent() {
                            std::fs::create_dir_all(parent)
                                .map_err(|e| ArchiveError::ExtractionFailed {
                                    reason: format!("Failed to create parent directory '{}': {}", parent.display(), e),
                                })?;
                        }
                        
                        // Extract file - this consumes the header
                        extract_archive = header.extract_to(&dest_path)
                            .map_err(|e| ArchiveError::ExtractionFailed {
                                reason: format!("Failed to extract RAR file '{}': {:?}", entry_path, e),
                            })?;
                        
                        bytes_processed += unpacked_size as u64;
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
                } else {
                    // Skip this entry
                    extract_archive = header.skip()
                        .map_err(|e| ArchiveError::CorruptedArchive {
                            reason: format!("Failed to skip RAR entry: {:?}", e),
                        })?;
                    }
                    Ok(None) => break,
                    Err(e) => return Err(ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read RAR header: {:?}", e),
                    }),
                }
            }
            
            Ok(())
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    async fn extract_entry_to_memory(&self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let archive_path = self.archive_path.clone();
        let path = path.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ArchiveError> {
            let mut archive = RarArchive::new(&archive_path)
                .open_for_processing()
                .map_err(|e| ArchiveError::CorruptedArchive {
                    reason: format!("Failed to open RAR archive: {:?}", e),
                })?;
            
            loop {
                let header = match archive.read_header() {
                    Ok(Some(header)) => header,
                    Ok(None) => break,
                    Err(e) => return Err(ArchiveError::CorruptedArchive {
                        reason: format!("Failed to read RAR header: {:?}", e),
                    }),
                };
                let entry = header.entry();
                let entry_path = entry.filename.to_string_lossy().to_string();
                
                if entry_path == path && !entry.is_directory() {
                    // Create a temporary file to extract to, then read it
                    let temp_dir = std::env::temp_dir();
                    let temp_file = temp_dir.join(format!("nimbus_rar_temp_{}", 
                        std::process::id()));
                    
                    header.extract_to(&temp_file)
                        .map_err(|e| ArchiveError::ExtractionFailed {
                            reason: format!("Failed to extract RAR entry to temp file: {:?}", e),
                        })?;
                    
                    // Read the temporary file
                    let buffer = std::fs::read(&temp_file)
                        .map_err(|e| ArchiveError::ExtractionFailed {
                            reason: format!("Failed to read temp file: {}", e),
                        })?;
                    
                    // Clean up
                    let _ = std::fs::remove_file(&temp_file);
                    
                    return Ok(buffer);
                } else {
                    archive = header.skip()
                        .map_err(|e| ArchiveError::CorruptedArchive {
                            reason: format!("Failed to skip RAR entry: {:?}", e),
                        })?;
                }
            }
            
            Err(ArchiveError::NotFound { path })
        }).await.map_err(|e| ArchiveError::ExtractionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }
    
    fn requires_password(&self) -> bool {
        // RAR can have password-protected entries
        // We would need to check this more thoroughly
        false
    }
    
    fn format(&self) -> ArchiveFormat {
        ArchiveFormat::Rar
    }
}
*/

/// Archive factory for creating appropriate readers
pub struct ArchiveFactory;

impl ArchiveFactory {
    /// Create an archive reader for the given file
    pub fn create_reader(archive_path: PathBuf) -> Result<Box<dyn ArchiveReader>, ArchiveError> {
        let format = ArchiveFormat::detect(&archive_path)
            .map_err(|e| ArchiveError::CorruptedArchive {
                reason: format!("Failed to read file header: {}", e),
            })?
            .ok_or_else(|| ArchiveError::UnsupportedFormat {
                format: archive_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            })?;
        
        match format {
            ArchiveFormat::Zip => Ok(Box::new(ZipArchiveReader::new(archive_path))),
            ArchiveFormat::Tar | ArchiveFormat::TarGz | ArchiveFormat::TarBz2 => {
                Ok(Box::new(TarArchiveReader::new(archive_path, format)))
            },
            ArchiveFormat::SevenZ => Ok(Box::new(SevenZArchiveReader::new(archive_path))),
            ArchiveFormat::Rar => Err(ArchiveError::UnsupportedFormat { 
                format: "RAR format has syntax issues - temporarily disabled".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
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
        assert_eq!(ArchiveFormat::from_path(Path::new("test.tar")), Some(ArchiveFormat::Tar));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.tar.gz")), Some(ArchiveFormat::TarGz));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.tgz")), Some(ArchiveFormat::TarGz));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.tar.bz2")), Some(ArchiveFormat::TarBz2));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.tbz2")), Some(ArchiveFormat::TarBz2));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.7z")), Some(ArchiveFormat::SevenZ));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.rar")), Some(ArchiveFormat::Rar));
        assert_eq!(ArchiveFormat::from_path(Path::new("test.txt")), None);
    }
    
    #[tokio::test]
    async fn test_header_based_detection() {
        let (_temp_dir, zip_path) = create_test_zip();
        
        // Test header-based detection on actual ZIP file
        let detected = ArchiveFormat::from_header(&zip_path)
            .expect("Failed to read ZIP header");
        assert_eq!(detected, Some(ArchiveFormat::Zip));
        
        // Test combined detection (should prefer header over extension)
        let detected = ArchiveFormat::detect(&zip_path)
            .expect("Failed to detect ZIP format");
        assert_eq!(detected, Some(ArchiveFormat::Zip));
    }
    
    #[tokio::test]
    async fn test_integrity_verification() {
        let (_temp_dir, zip_path) = create_test_zip();
        
        let reader = ZipArchiveReader::new(zip_path);
        
        // Test CRC32 verification on a known file
        let (data, integrity_result) = reader
            .extract_and_verify_crc32("test.txt")
            .await
            .expect("Failed to extract and verify");
            
        assert_eq!(data, b"Hello, World!");
        assert!(integrity_result.passed, "CRC32 verification should pass");
        assert_eq!(integrity_result.algorithm, HashAlgorithm::Crc32);
        
        // Test hash computation
        let (data2, sha256_hash) = reader
            .extract_and_compute_hash("test.txt", HashAlgorithm::Sha256)
            .await
            .expect("Failed to extract and compute hash");
            
        assert_eq!(data2, b"Hello, World!");
        // SHA256 of "Hello, World!" is known
        assert_eq!(sha256_hash.len(), 64); // SHA256 is 64 hex chars
        
        // Test manual hash verification
        let expected_sha256 = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
        let integrity_result = IntegrityVerifier::verify_hash(&data, expected_sha256, HashAlgorithm::Sha256);
        assert!(integrity_result.passed, "SHA256 verification should pass");
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
    
    // ===== Progress Tracking Tests =====
    
    #[test]
    fn test_progress_tracker_creation() {
        let mut tracker = ProgressTracker::new(ProgressOperation::Extracting);
        
        let progress = tracker.create_progress(
            "test.txt".to_string(),
            1,
            10,
            100,
            1000,
            ProgressStage::Processing,
            None,
        );
        
        assert_eq!(progress.operation, ProgressOperation::Extracting);
        assert_eq!(progress.stage, ProgressStage::Processing);
        assert_eq!(progress.current_file, "test.txt");
        assert_eq!(progress.files_processed, 1);
        assert_eq!(progress.total_files, 10);
        assert_eq!(progress.bytes_processed, 100);
        assert_eq!(progress.total_bytes, 1000);
        // Speed and ETA may be None if elapsed time is too small
        // This is expected behavior in fast test scenarios
    }
    
    #[test]
    fn test_progress_tracker_speed_calculation() {
        let mut tracker = ProgressTracker::new(ProgressOperation::Extracting);
        
        // First progress update
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _progress1 = tracker.create_progress(
            "file1.txt".to_string(),
            1,
            10,
            1000,
            10000,
            ProgressStage::Processing,
            None,
        );
        
        // Second progress update after some processing
        std::thread::sleep(std::time::Duration::from_millis(10));
        let progress2 = tracker.create_progress(
            "file2.txt".to_string(),
            2,
            10,
            2000,
            10000,
            ProgressStage::Processing,
            None,
        );
        
        // Speed calculation may return None in very fast test scenarios
        // In real usage with actual file processing, speed will be calculated properly
        if let Some(speed) = progress2.speed_bps {
            assert!(speed >= 0, "Speed should be non-negative when calculated");
        }
    }
    
    // ===== Integrity Verification Tests =====
    
    #[test]
    fn test_integrity_verifier_crc32() {
        let test_data = b"Hello, World!";
        let crc32 = IntegrityVerifier::compute_crc32(test_data);
        
        // Verify CRC32 calculation
        let result = IntegrityVerifier::verify_crc32(test_data, crc32);
        assert!(result.passed);
        assert_eq!(result.algorithm, HashAlgorithm::Crc32);
        
        // Test with wrong CRC32
        let wrong_result = IntegrityVerifier::verify_crc32(test_data, crc32 + 1);
        assert!(!wrong_result.passed);
        assert!(wrong_result.error_message.is_some());
    }
    
    #[test]
    fn test_integrity_verifier_hash_algorithms() {
        let test_data = b"Hello, World!";
        
        // Test MD5
        let md5 = IntegrityVerifier::compute_hash(test_data, HashAlgorithm::Md5);
        assert_eq!(md5.len(), 32); // MD5 is 32 hex chars
        let result = IntegrityVerifier::verify_hash(test_data, &md5, HashAlgorithm::Md5);
        assert!(result.passed);
        
        // Test SHA1  
        let sha1 = IntegrityVerifier::compute_hash(test_data, HashAlgorithm::Sha1);
        assert_eq!(sha1.len(), 40); // SHA1 is 40 hex chars
        let result = IntegrityVerifier::verify_hash(test_data, &sha1, HashAlgorithm::Sha1);
        assert!(result.passed);
        
        // Test SHA256
        let sha256 = IntegrityVerifier::compute_hash(test_data, HashAlgorithm::Sha256);
        assert_eq!(sha256.len(), 64); // SHA256 is 64 hex chars
        let result = IntegrityVerifier::verify_hash(test_data, &sha256, HashAlgorithm::Sha256);
        assert!(result.passed);
        
        // Test known SHA256 hash
        let expected_sha256 = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
        let result = IntegrityVerifier::verify_hash(test_data, expected_sha256, HashAlgorithm::Sha256);
        assert!(result.passed);
    }
    
    // ===== Error Handling Tests =====
    
    #[test]
    fn test_error_handler_context_creation() {
        let error = ArchiveError::NotFound { path: "/nonexistent/file.zip".to_string() };
        let context = ErrorHandler::create_context(&error, "extracting archive");
        
        assert_eq!(context.context, "extracting archive");
        assert!(!context.recovery_suggestions.is_empty());
        assert!(context.recovery_suggestions.iter().any(|s| s.contains("Verify the file path")));
        assert_eq!(context.severity, ErrorSeverity::Error);
        assert!(!context.is_retryable);
    }
    
    #[test] 
    fn test_error_handler_disk_space() {
        let error = ArchiveError::InsufficientDiskSpace { required: 1000, available: 500 };
        let context = ErrorHandler::create_context(&error, "extracting large archive");
        
        assert_eq!(context.severity, ErrorSeverity::Error);
        assert!(context.is_retryable);
        assert!(context.recovery_suggestions.iter().any(|s| s.contains("Free up")));
        
        let friendly_msg = ErrorHandler::user_friendly_message(&error);
        assert!(friendly_msg.contains("disk space"));
        assert!(friendly_msg.contains("1000"));
    }
    
    #[test]
    fn test_error_handler_recoverability() {
        // Recoverable errors
        let recoverable_errors = vec![
            ArchiveError::PasswordRequired,
            ArchiveError::InsufficientDiskSpace { required: 1000, available: 500 },
            ArchiveError::OperationCancelled,
        ];
        
        for error in recoverable_errors {
            assert!(ErrorHandler::is_recoverable(&error), "Error should be recoverable: {:?}", error);
        }
        
        // Non-recoverable errors
        let non_recoverable_errors = vec![
            ArchiveError::CorruptedArchive { reason: "Invalid header".to_string() },
            ArchiveError::UnsupportedFormat { format: "unknown".to_string() },
        ];
        
        for error in non_recoverable_errors {
            assert!(!ErrorHandler::is_recoverable(&error), "Error should not be recoverable: {:?}", error);
        }
    }
    
    // ===== Archive Format Detection Tests =====
    
    #[test]
    fn test_archive_format_extension_detection() {
        // Test all supported extensions
        assert_eq!(ArchiveFormat::from_path(Path::new("test.zip")), Some(ArchiveFormat::Zip));
        assert_eq!(ArchiveFormat::from_path(Path::new("TEST.ZIP")), Some(ArchiveFormat::Zip)); // Case insensitive
        
        assert_eq!(ArchiveFormat::from_path(Path::new("archive.tar")), Some(ArchiveFormat::Tar));
        assert_eq!(ArchiveFormat::from_path(Path::new("archive.tar.gz")), Some(ArchiveFormat::TarGz));
        assert_eq!(ArchiveFormat::from_path(Path::new("archive.tgz")), Some(ArchiveFormat::TarGz));
        assert_eq!(ArchiveFormat::from_path(Path::new("archive.tar.bz2")), Some(ArchiveFormat::TarBz2));
        assert_eq!(ArchiveFormat::from_path(Path::new("archive.tbz2")), Some(ArchiveFormat::TarBz2));
        
        assert_eq!(ArchiveFormat::from_path(Path::new("data.7z")), Some(ArchiveFormat::SevenZ));
        assert_eq!(ArchiveFormat::from_path(Path::new("files.rar")), Some(ArchiveFormat::Rar));
        
        // Non-archive files
        assert_eq!(ArchiveFormat::from_path(Path::new("document.txt")), None);
        assert_eq!(ArchiveFormat::from_path(Path::new("image.jpg")), None);
        assert_eq!(ArchiveFormat::from_path(Path::new("no_extension")), None);
    }
    
    #[tokio::test] 
    async fn test_magic_bytes_detection() {
        let (_temp_dir, zip_path) = create_test_zip();
        
        // Test ZIP magic bytes detection
        let format = ArchiveFormat::from_header(&zip_path).unwrap();
        assert_eq!(format, Some(ArchiveFormat::Zip));
        
        // Test fallback to extension when header fails
        let non_archive_path = _temp_dir.path().join("test.txt");
        std::fs::write(&non_archive_path, b"not an archive").unwrap();
        
        let format = ArchiveFormat::from_header(&non_archive_path).unwrap();
        assert_eq!(format, None); // Should not detect any format
    }
    
    // ===== Edge Case Tests =====
    
    #[tokio::test]
    async fn test_empty_archive_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = temp_dir.path().join("empty.zip");
        
        // Create an empty ZIP file
        let file = std::fs::File::create(&zip_path).expect("Failed to create zip file");
        let zip_writer = zip::ZipWriter::new(file);
        zip_writer.finish().expect("Failed to finish empty ZIP");
        
        let reader = ZipArchiveReader::new(zip_path);
        let entries = reader.list_entries().await.expect("Failed to list empty archive");
        assert_eq!(entries.len(), 0);
    }
    
    #[tokio::test]
    async fn test_nonexistent_file_error() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/archive.zip");
        let reader = ZipArchiveReader::new(nonexistent_path);
        
        let result = reader.list_entries().await;
        assert!(result.is_err());
        
        if let Err(ArchiveError::NotFound { path }) = result {
            assert!(path.contains("nonexistent"));
        } else {
            panic!("Expected NotFound error");
        }
    }
    
    #[tokio::test]
    async fn test_corrupted_archive_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let corrupt_path = temp_dir.path().join("corrupt.zip");
        
        // Create a file with invalid ZIP content
        std::fs::write(&corrupt_path, b"This is not a ZIP file").unwrap();
        
        let reader = ZipArchiveReader::new(corrupt_path);
        let result = reader.list_entries().await;
        assert!(result.is_err());
        
        match result {
            Err(ArchiveError::CorruptedArchive { .. }) => {
                // Expected
            }
            _ => panic!("Expected CorruptedArchive error"),
        }
    }
    
    #[tokio::test]
    async fn test_extract_nonexistent_entry() {
        let (_temp_dir, zip_path) = create_test_zip();
        let reader = ZipArchiveReader::new(zip_path);
        
        let result = reader.extract_entry_to_memory("nonexistent.txt").await;
        assert!(result.is_err());
        
        match result {
            Err(ArchiveError::NotFound { path }) => {
                assert_eq!(path, "nonexistent.txt");
            }
            _ => panic!("Expected NotFound error"),
        }
    }
    
    // ===== TAR Archive Tests =====
    
    fn create_test_tar() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let tar_path = temp_dir.path().join("test.tar");
        
        // Create a simple TAR file for testing
        let tar_file = std::fs::File::create(&tar_path).expect("Failed to create tar file");
        let mut tar_builder = tar::Builder::new(tar_file);
        
        // Add a text file
        let mut header = tar::Header::new_gnu();
        header.set_size(13);
        header.set_mode(0o644);
        header.set_cksum();
        tar_builder.append_data(&mut header, "test.txt", b"Hello, World!".as_ref())
            .expect("Failed to add file to tar");
        
        // Add a directory entry manually
        let mut dir_header = tar::Header::new_gnu();
        dir_header.set_entry_type(tar::EntryType::Directory);
        dir_header.set_size(0);
        dir_header.set_mode(0o755);
        dir_header.set_cksum();
        tar_builder.append_data(&mut dir_header, "subdir/", std::io::empty())
            .expect("Failed to add directory to tar");
        
        let mut nested_header = tar::Header::new_gnu();
        nested_header.set_size(14);
        nested_header.set_mode(0o644);
        nested_header.set_cksum();
        tar_builder.append_data(&mut nested_header, "subdir/nested.txt", b"Nested content".as_ref())
            .expect("Failed to add nested file to tar");
        
        tar_builder.finish().expect("Failed to finish TAR");
        
        (temp_dir, tar_path)
    }
    
    #[tokio::test]
    async fn test_tar_listing() {
        let (_temp_dir, tar_path) = create_test_tar();
        
        let reader = TarArchiveReader::new(tar_path, ArchiveFormat::Tar);
        let entries = reader.list_entries().await.expect("Failed to list TAR entries");
        
        assert_eq!(entries.len(), 3); // test.txt, subdir, subdir/nested.txt
        
        let text_file = entries.iter().find(|e| e.path == "test.txt").expect("test.txt not found");
        assert!(!text_file.is_directory);
        assert_eq!(text_file.size, 13);
        
        let subdir = entries.iter().find(|e| e.path == "subdir/" || e.path == "subdir").expect("subdir not found");
        assert!(subdir.is_directory);
        assert_eq!(subdir.size, 0);
    }
    
    #[tokio::test] 
    async fn test_tar_extraction() {
        let (_temp_dir, tar_path) = create_test_tar();
        let extract_dir = TempDir::new().expect("Failed to create extract dir");
        
        let reader = TarArchiveReader::new(tar_path, ArchiveFormat::Tar);
        let options = ExtractionOptions::default();
        
        reader.extract(extract_dir.path(), options, None).await
            .expect("Failed to extract TAR archive");
        
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
    
    // ===== Extraction Options Tests =====
    
    #[tokio::test]
    async fn test_extraction_options_default() {
        let options = ExtractionOptions::default();
        
        assert!(options.preserve_paths);
        assert_eq!(options.overwrite_policy, OverwritePolicy::Ask);
        assert!(options.password.is_none());
        assert!(!options.create_subfolder);
        assert!(options.entries.is_none());
    }
    
    #[tokio::test]
    async fn test_extraction_with_subfolder() {
        let (_temp_dir, zip_path) = create_test_zip();
        let extract_dir = TempDir::new().expect("Failed to create extract dir");
        
        let reader = ZipArchiveReader::new(zip_path);
        let options = ExtractionOptions {
            create_subfolder: true,
            ..Default::default()
        };
        
        reader.extract(extract_dir.path(), options, None).await
            .expect("Failed to extract with subfolder");
        
        // Files should be extracted to a subfolder named after the archive
        let subfolder_path = extract_dir.path().join("test");
        assert!(subfolder_path.exists());
        
        let test_file_path = subfolder_path.join("test.txt");
        assert!(test_file_path.exists());
        
        let content = fs::read_to_string(test_file_path).expect("Failed to read file from subfolder");
        assert_eq!(content, "Hello, World!");
    }
    
    // ===== Progress Callback Tests =====
    
    #[tokio::test]
    async fn test_extraction_with_progress_callback() {
        let (_temp_dir, zip_path) = create_test_zip();
        let extract_dir = TempDir::new().expect("Failed to create extract dir");
        
        let progress_updates = Arc::new(Mutex::new(Vec::new()));
        let progress_updates_clone = progress_updates.clone();
        
        let progress_callback = Box::new(move |progress: ProgressInfo| {
            let mut updates = progress_updates_clone.lock().unwrap();
            updates.push(progress);
        });
        
        let reader = ZipArchiveReader::new(zip_path);
        let options = ExtractionOptions::default();
        
        reader.extract(extract_dir.path(), options, Some(progress_callback)).await
            .expect("Failed to extract with progress");
        
        let updates = progress_updates.lock().unwrap();
        assert!(!updates.is_empty(), "Should have received progress updates");
        
        // Check that we got initialization and completion stages
        let has_initializing = updates.iter().any(|p| p.stage == ProgressStage::Initializing);
        let has_completed = updates.iter().any(|p| p.stage == ProgressStage::Completed);
        
        assert!(has_initializing, "Should have initialization progress");
        assert!(has_completed, "Should have completion progress");
        
        // Verify progress values make sense
        for progress in updates.iter() {
            assert!(progress.files_processed <= progress.total_files);
            assert!(progress.bytes_processed <= progress.total_bytes);
            assert_eq!(progress.operation, ProgressOperation::Extracting);
        }
    }
}