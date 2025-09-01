//! Remote File System Support for Nimbus
//!
//! Provides unified access to remote file systems including SFTP, FTP, and WebDAV.
//! Supports connection pooling, credential management, and async operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use thiserror::Error;
use chrono::{DateTime, Utc};

/// Remote file system error types
#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    #[error("Transfer failed: {message}")]
    TransferFailed { message: String },

    #[error("Timeout: {message}")]
    Timeout { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Other error: {message}")]
    Other { message: String },
}

/// Remote file system connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub protocol: RemoteProtocol,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub private_key_passphrase: Option<String>,
    pub timeout: Option<u64>, // seconds
    pub connection_name: String,
    pub use_passive_ftp: bool,
    pub verify_ssl: bool,
    pub base_path: Option<String>,
}

/// Supported remote protocols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemoteProtocol {
    Sftp,
    Ftp,
    Ftps, // FTP over SSL/TLS
    WebDav,
    WebDavs, // WebDAV over HTTPS
}

/// Remote file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub created: Option<DateTime<Utc>>,
    pub file_type: RemoteFileType,
    pub permissions: Option<RemotePermissions>,
    pub mime_type: Option<String>,
    pub is_hidden: bool,
    pub owner: Option<String>,
    pub group: Option<String>,
}

/// Remote file types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemoteFileType {
    File,
    Directory,
    Symlink,
    Other,
}

/// Remote file permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub mode: Option<u32>, // Unix-style permissions
}

/// Transfer options for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOptions {
    pub overwrite: bool,
    pub preserve_timestamps: bool,
    pub verify_integrity: bool,
    pub resume: bool,
    pub create_directories: bool,
    pub buffer_size: Option<usize>,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            preserve_timestamps: true,
            verify_integrity: false,
            resume: false,
            create_directories: true,
            buffer_size: Some(8192), // 8KB default
        }
    }
}

/// Transfer progress callback
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Remote file system trait
#[async_trait]
pub trait RemoteFileSystem: Send + Sync {
    /// Get connection information
    fn config(&self) -> &RemoteConfig;

    /// Get current connection status
    async fn status(&self) -> ConnectionStatus;

    /// Connect to the remote server
    async fn connect(&mut self) -> Result<(), RemoteError>;

    /// Disconnect from the remote server
    async fn disconnect(&mut self) -> Result<(), RemoteError>;

    /// Test connectivity without full connection
    async fn test_connection(&self) -> Result<(), RemoteError>;

    /// List directory contents
    async fn list_directory(&mut self, path: &str) -> Result<Vec<RemoteFileInfo>, RemoteError>;

    /// Get file/directory information
    async fn get_file_info(&mut self, path: &str) -> Result<RemoteFileInfo, RemoteError>;

    /// Check if file/directory exists
    async fn exists(&mut self, path: &str) -> Result<bool, RemoteError>;

    /// Create directory
    async fn create_directory(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError>;

    /// Remove file or directory
    async fn remove(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError>;

    /// Rename/move file or directory
    async fn rename(&mut self, from: &str, to: &str) -> Result<(), RemoteError>;

    /// Download file from remote to local
    async fn download(
        &mut self,
        remote_path: &str,
        local_path: &Path,
        options: TransferOptions,
        progress: Option<ProgressCallback>,
    ) -> Result<(), RemoteError>;

    /// Upload file from local to remote
    async fn upload(
        &mut self,
        local_path: &Path,
        remote_path: &str,
        options: TransferOptions,
        progress: Option<ProgressCallback>,
    ) -> Result<(), RemoteError>;

    /// Read file content as bytes
    async fn read_file(&mut self, path: &str) -> Result<Vec<u8>, RemoteError>;

    /// Write file content from bytes
    async fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), RemoteError>;

    /// Get available disk space (if supported)
    async fn get_disk_space(&mut self, path: &str) -> Result<Option<DiskSpace>, RemoteError>;

    /// Set file permissions (if supported)
    async fn set_permissions(&mut self, path: &str, permissions: u32) -> Result<(), RemoteError>;
}

/// Disk space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpace {
    pub total: u64,
    pub free: u64,
    pub available: u64, // Available to current user
}

/// Connection pool for managing multiple remote connections
pub struct ConnectionPool {
    connections: HashMap<String, Box<dyn RemoteFileSystem>>,
    max_connections: usize,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            max_connections,
        }
    }

    /// Add a connection to the pool
    pub fn add_connection(&mut self, id: String, connection: Box<dyn RemoteFileSystem>) -> Result<(), RemoteError> {
        if self.connections.len() >= self.max_connections {
            return Err(RemoteError::Other {
                message: "Connection pool is full".to_string(),
            });
        }

        self.connections.insert(id, connection);
        Ok(())
    }

    /// Get a connection by ID
    pub fn get_connection(&mut self, id: &str) -> Option<&mut Box<dyn RemoteFileSystem>> {
        self.connections.get_mut(id)
    }

    /// Remove a connection from the pool
    pub async fn remove_connection(&mut self, id: &str) -> Result<(), RemoteError> {
        if let Some(mut connection) = self.connections.remove(id) {
            connection.disconnect().await?;
        }
        Ok(())
    }

    /// Get all connection IDs
    pub fn connection_ids(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }

    /// Get connection statuses
    pub async fn connection_statuses(&mut self) -> HashMap<String, ConnectionStatus> {
        let mut statuses = HashMap::new();
        for (id, connection) in &mut self.connections {
            statuses.insert(id.clone(), connection.status().await);
        }
        statuses
    }

    /// Close all connections
    pub async fn close_all(&mut self) -> Result<(), RemoteError> {
        let ids: Vec<String> = self.connections.keys().cloned().collect();
        for id in ids {
            self.remove_connection(&id).await?;
        }
        Ok(())
    }
}

/// Factory for creating remote file system instances
pub struct RemoteFileSystemFactory;

impl RemoteFileSystemFactory {
    /// Create a remote file system instance based on protocol
    pub fn create(config: RemoteConfig) -> Result<Box<dyn RemoteFileSystem>, RemoteError> {
        match config.protocol {
            RemoteProtocol::Sftp => {
                let sftp = crate::sftp::SftpClient::new(config)?;
                Ok(Box::new(sftp))
            }
            RemoteProtocol::Ftp | RemoteProtocol::Ftps => {
                let ftp = crate::ftp::FtpClient::new(config)?;
                Ok(Box::new(ftp))
            }
            RemoteProtocol::WebDav | RemoteProtocol::WebDavs => {
                let webdav = crate::webdav::WebDavClient::new(config)?;
                Ok(Box::new(webdav))
            }
        }
    }

    /// Get supported protocols
    pub fn supported_protocols() -> Vec<RemoteProtocol> {
        vec![
            RemoteProtocol::Sftp,
            RemoteProtocol::Ftp,
            RemoteProtocol::Ftps,
            RemoteProtocol::WebDav,
            RemoteProtocol::WebDavs,
        ]
    }

    /// Validate configuration
    pub fn validate_config(config: &RemoteConfig) -> Result<(), RemoteError> {
        if config.host.is_empty() {
            return Err(RemoteError::Other {
                message: "Host is required".to_string(),
            });
        }

        if config.username.is_empty() {
            return Err(RemoteError::Other {
                message: "Username is required".to_string(),
            });
        }

        match config.protocol {
            RemoteProtocol::Sftp => {
                if config.password.is_none() && config.private_key_path.is_none() {
                    return Err(RemoteError::Other {
                        message: "SFTP requires either password or private key".to_string(),
                    });
                }
            }
            RemoteProtocol::Ftp | RemoteProtocol::Ftps => {
                if config.password.is_none() {
                    return Err(RemoteError::Other {
                        message: "FTP requires password".to_string(),
                    });
                }
            }
            RemoteProtocol::WebDav | RemoteProtocol::WebDavs => {
                if config.password.is_none() {
                    return Err(RemoteError::Other {
                        message: "WebDAV requires password".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Utility functions
pub mod utils {
    use super::*;

    /// Parse remote URL into config
    pub fn parse_url(url: &str) -> Result<RemoteConfig, RemoteError> {
        let parsed = url::Url::parse(url)?;

        let protocol = match parsed.scheme() {
            "sftp" => RemoteProtocol::Sftp,
            "ftp" => RemoteProtocol::Ftp,
            "ftps" => RemoteProtocol::Ftps,
            "webdav" => RemoteProtocol::WebDav,
            "webdavs" | "https" => RemoteProtocol::WebDavs,
            scheme => return Err(RemoteError::Other {
                message: format!("Unsupported protocol: {}", scheme),
            }),
        };

        let host = parsed.host_str()
            .ok_or_else(|| RemoteError::Other {
                message: "Host is required".to_string(),
            })?
            .to_string();

        let port = parsed.port();
        let username = parsed.username().to_string();
        let password = parsed.password().map(|p| p.to_string());

        Ok(RemoteConfig {
            protocol,
            host,
            port,
            username,
            password,
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: format!("{}@{}", username, host),
            use_passive_ftp: true,
            verify_ssl: true,
            base_path: if parsed.path().is_empty() {
                None
            } else {
                Some(parsed.path().to_string())
            },
        })
    }

    /// Format file size for display
    pub fn format_size(size: u64) -> String {
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

    /// Normalize remote path
    pub fn normalize_path(path: &str) -> String {
        let mut normalized = path.replace('\\', "/");
        if !normalized.starts_with('/') {
            normalized = format!("/{}", normalized);
        }
        normalized
    }

    /// Get file extension from remote path
    pub fn get_extension(path: &str) -> Option<String> {
        std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Get parent directory from remote path
    pub fn get_parent_dir(path: &str) -> String {
        let normalized = normalize_path(path);
        match normalized.rfind('/') {
            Some(0) => "/".to_string(),
            Some(pos) => normalized[..pos].to_string(),
            None => "/".to_string(),
        }
    }

    /// Join remote paths
    pub fn join_path(base: &str, path: &str) -> String {
        let normalized_base = normalize_path(base);
        let normalized_path = path.replace('\\', "/");
        
        if normalized_path.starts_with('/') {
            normalized_path
        } else if normalized_base.ends_with('/') {
            format!("{}{}", normalized_base, normalized_path)
        } else {
            format!("{}/{}", normalized_base, normalized_path)
        }
    }
}

// Re-export implementations
pub mod sftp;
pub mod ftp;
pub mod webdav;

pub use sftp::SftpClient;
pub use ftp::FtpClient;
pub use webdav::WebDavClient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let url = "sftp://user:pass@example.com:22/path/to/dir";
        let config = utils::parse_url(url).unwrap();
        
        assert_eq!(config.protocol, RemoteProtocol::Sftp);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, Some(22));
        assert_eq!(config.username, "user");
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.base_path, Some("/path/to/dir".to_string()));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(utils::normalize_path("path/to/file"), "/path/to/file");
        assert_eq!(utils::normalize_path("/path/to/file"), "/path/to/file");
        assert_eq!(utils::normalize_path("path\\to\\file"), "/path/to/file");
    }

    #[test]
    fn test_join_path() {
        assert_eq!(utils::join_path("/base", "file.txt"), "/base/file.txt");
        assert_eq!(utils::join_path("/base/", "file.txt"), "/base/file.txt");
        assert_eq!(utils::join_path("/base", "/absolute/file.txt"), "/absolute/file.txt");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(utils::format_size(1024), "1.0 KB");
        assert_eq!(utils::format_size(1048576), "1.0 MB");
        assert_eq!(utils::format_size(500), "500 B");
    }

    #[test]
    fn test_validate_config() {
        let mut config = RemoteConfig {
            protocol: RemoteProtocol::Sftp,
            host: "example.com".to_string(),
            port: Some(22),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: true,
            verify_ssl: true,
            base_path: None,
        };

        assert!(RemoteFileSystemFactory::validate_config(&config).is_ok());

        config.host = "".to_string();
        assert!(RemoteFileSystemFactory::validate_config(&config).is_err());
    }
}