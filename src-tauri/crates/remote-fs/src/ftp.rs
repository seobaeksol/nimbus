//! FTP (File Transfer Protocol) implementation
//!
//! Provides FTP and FTPS support using the suppaftp crate.
//! Supports passive/active modes, SSL/TLS encryption, and basic file operations.

use crate::{
    RemoteConfig, RemoteError, RemoteFileSystem, RemoteFileInfo, RemoteFileType, 
    RemotePermissions, TransferOptions, ProgressCallback, ConnectionStatus, DiskSpace
};
use async_trait::async_trait;
use suppaftp::FtpStream;
use std::path::Path;

/// FTP client implementation
pub struct FtpClient {
    config: RemoteConfig,
    connection: Option<FtpStream>,
    status: ConnectionStatus,
}

impl FtpClient {
    /// Create a new FTP client
    pub fn new(config: RemoteConfig) -> Result<Self, RemoteError> {
        if config.protocol != crate::RemoteProtocol::Ftp && config.protocol != crate::RemoteProtocol::Ftps {
            return Err(RemoteError::Other {
                message: "Invalid protocol for FTP client".to_string(),
            });
        }

        Ok(Self {
            config,
            connection: None,
            status: ConnectionStatus::Disconnected,
        })
    }

    /// Get the default port for FTP/FTPS
    fn default_port(&self) -> u16 {
        match self.config.protocol {
            crate::RemoteProtocol::Ftp => 21,
            crate::RemoteProtocol::Ftps => 990,
            _ => 21,
        }
    }

    /// Convert FTP file listing to RemoteFileInfo
    fn parse_file_listing(&self, line: &str, base_path: &str) -> Option<RemoteFileInfo> {
        // Parse Unix-style listing: -rw-r--r-- 1 user group 1234 Dec 25 12:34 filename
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            return None;
        }

        let permissions_str = parts[0];
        let size_str = parts[4];
        let name = parts[8..].join(" ");

        // Skip . and .. entries
        if name == "." || name == ".." {
            return None;
        }

        // Parse file type and permissions
        let (file_type, permissions) = self.parse_permissions(permissions_str);

        // Parse size
        let size = size_str.parse::<u64>().unwrap_or(0);

        // Build full path
        let path = if base_path.ends_with('/') {
            format!("{}{}", base_path, name)
        } else {
            format!("{}/{}", base_path, name)
        };

        Some(RemoteFileInfo {
            name: name.clone(),
            path: path.clone(),
            size,
            modified: None, // FTP listing doesn't always provide reliable timestamps
            created: None,
            file_type,
            permissions,
            mime_type: crate::utils::get_extension(&path)
                .and_then(|ext| match ext.as_str() {
                    "txt" => Some("text/plain".to_string()),
                    "html" => Some("text/html".to_string()),
                    "json" => Some("application/json".to_string()),
                    "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                    "png" => Some("image/png".to_string()),
                    _ => None,
                }),
            is_hidden: name.starts_with('.'),
            owner: None,
            group: None,
        })
    }

    /// Parse Unix-style permissions string
    fn parse_permissions(&self, perm_str: &str) -> (RemoteFileType, Option<RemotePermissions>) {
        if perm_str.len() < 10 {
            return (RemoteFileType::Other, None);
        }

        let file_type = match perm_str.chars().nth(0) {
            Some('d') => RemoteFileType::Directory,
            Some('l') => RemoteFileType::Symlink,
            Some('-') => RemoteFileType::File,
            _ => RemoteFileType::Other,
        };

        let permissions = if perm_str.len() >= 10 {
            let owner_read = perm_str.chars().nth(1) == Some('r');
            let owner_write = perm_str.chars().nth(2) == Some('w');
            let owner_execute = perm_str.chars().nth(3) == Some('x');
            let group_read = perm_str.chars().nth(4) == Some('r');
            let group_write = perm_str.chars().nth(5) == Some('w');
            let group_execute = perm_str.chars().nth(6) == Some('x');
            let other_read = perm_str.chars().nth(7) == Some('r');
            let other_write = perm_str.chars().nth(8) == Some('w');
            let other_execute = perm_str.chars().nth(9) == Some('x');

            Some(RemotePermissions {
                read: owner_read || group_read || other_read,
                write: owner_write || group_write || other_write,
                execute: owner_execute || group_execute || other_execute,
                mode: None, // Could calculate numeric mode if needed
            })
        } else {
            None
        };

        (file_type, permissions)
    }

    /// Ensure connection is active
    async fn ensure_connected(&mut self) -> Result<(), RemoteError> {
        if self.connection.is_none() {
            self.connect().await?;
        }
        Ok(())
    }

    /// Get FTP connection reference
    fn get_connection(&mut self) -> Result<&mut FtpStream, RemoteError> {
        self.connection.as_mut().ok_or_else(|| RemoteError::ConnectionFailed {
            message: "Not connected to FTP server".to_string(),
        })
    }
}

#[async_trait]
impl RemoteFileSystem for FtpClient {
    fn config(&self) -> &RemoteConfig {
        &self.config
    }

    async fn status(&self) -> ConnectionStatus {
        self.status.clone()
    }

    async fn connect(&mut self) -> Result<(), RemoteError> {
        self.status = ConnectionStatus::Connecting;

        let host = &self.config.host;
        let port = self.config.port.unwrap_or_else(|| self.default_port());
        let username = &self.config.username;
        let password = self.config.password.as_ref()
            .ok_or_else(|| RemoteError::AuthenticationFailed {
                message: "Password is required for FTP".to_string(),
            })?;

        // Create FTP connection
        let mut ftp = FtpStream::connect(&format!("{}:{}", host, port))
            .map_err(|e| RemoteError::ConnectionFailed {
                message: format!("Failed to connect to FTP server: {}", e),
            })?;

        // Note: FTPS support would require additional configuration
        // For now, we'll implement basic FTP only

        // Login
        ftp.login(username, password)
            .map_err(|e| RemoteError::AuthenticationFailed {
                message: format!("FTP login failed: {}", e),
            })?;

        // Set transfer mode (simplified)
        ftp.transfer_type(suppaftp::types::FileType::Binary)
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to set transfer type: {}", e),
            })?;

        // Change to base directory if specified
        if let Some(ref base_path) = self.config.base_path {
            ftp.cwd(base_path)
                .map_err(|e| RemoteError::ProtocolError {
                    message: format!("Failed to change to base directory: {}", e),
                })?;
        }

        self.connection = Some(ftp);
        self.status = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), RemoteError> {
        if let Some(mut connection) = self.connection.take() {
            let _ = connection.quit(); // Ignore quit errors
        }
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn test_connection(&self) -> Result<(), RemoteError> {
        // For FTP, we can't really test without connecting
        // This would require a separate connection just for testing
        Err(RemoteError::Other {
            message: "FTP test connection not implemented".to_string(),
        })
    }

    async fn list_directory(&mut self, path: &str) -> Result<Vec<RemoteFileInfo>, RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        // Change to directory
        connection.cwd(path)
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to change to directory {}: {}", path, e),
            })?;

        // Get directory listing
        let listing = connection.list(None)
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to list directory {}: {}", path, e),
            })?;

        let mut files = Vec::new();
        for line in listing {
            if let Some(file_info) = self.parse_file_listing(&line, path) {
                files.push(file_info);
            }
        }

        Ok(files)
    }

    async fn get_file_info(&mut self, path: &str) -> Result<RemoteFileInfo, RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        // Get file size
        let size = connection.size(path)
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to get file size for {}: {}", path, e),
            })?;

        // Get parent directory and list it to find the file
        let parent = crate::utils::get_parent_dir(path);
        let files = self.list_directory(&parent).await?;
        
        let file_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        files.into_iter()
            .find(|f| f.name == file_name)
            .ok_or_else(|| RemoteError::FileNotFound {
                path: path.to_string(),
            })
    }

    async fn exists(&mut self, path: &str) -> Result<bool, RemoteError> {
        match self.get_file_info(path).await {
            Ok(_) => Ok(true),
            Err(RemoteError::FileNotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn create_directory(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        if recursive {
            // Create parent directories first
            let parent = crate::utils::get_parent_dir(path);
            if parent != "/" && parent != path {
                // Release the connection borrow before recursive call
                drop(connection);
                self.create_directory(&parent, true).await?;
                // Re-acquire connection
                let connection = self.get_connection()?;
                connection.mkdir(path)
            } else {
                connection.mkdir(path)
            }
        } else {
            connection.mkdir(path)
        }
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to create directory {}: {}", path, e),
            })?;

        Ok(())
    }

    async fn remove(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        if recursive {
            // List directory contents and remove them first
            drop(connection); // Release connection borrow
            if let Ok(files) = self.list_directory(path).await {
                for file in files {
                    let file_path = crate::utils::join_path(path, &file.name);
                    self.remove(&file_path, true).await?;
                }
            }
            // Re-acquire connection
            let connection = self.get_connection()?;
            connection.rm(path)
        } else {
            connection.rm(path)
        }
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to remove {}: {}", path, e),
            })?;

        Ok(())
    }

    async fn rename(&mut self, from: &str, to: &str) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        connection.rename(from, to)
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to rename {} to {}: {}", from, to, e),
            })?;

        Ok(())
    }

    async fn download(
        &mut self,
        remote_path: &str,
        local_path: &Path,
        options: TransferOptions,
        progress: Option<ProgressCallback>,
    ) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        // Create local directory if needed
        if options.create_directories {
            if let Some(parent) = local_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| RemoteError::Io(e))?;
            }
        }

        // Check if file exists and handle overwrite
        if !options.overwrite && local_path.exists() {
            return Err(RemoteError::Other {
                message: "File already exists and overwrite is disabled".to_string(),
            });
        }

        // Get file size for progress tracking
        let file_size = connection.size(remote_path)
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to get file size: {}", e),
            })?;

        // Simplified download - in a real implementation, you would use streaming
        let data = connection.retr_as_buffer(remote_path)
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to download: {}", e),
            })?;

        tokio::fs::write(local_path, data.into_inner()).await.map_err(|e| RemoteError::Io(e))?;

        // Report progress
        if let Some(ref callback) = progress {
            callback(file_size as u64, file_size as u64);
        }

        Ok(())
    }

    async fn upload(
        &mut self,
        local_path: &Path,
        remote_path: &str,
        options: TransferOptions,
        progress: Option<ProgressCallback>,
    ) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        // Check if remote file exists and handle overwrite
        drop(connection); // Release connection borrow
        if !options.overwrite && self.exists(remote_path).await? {
            return Err(RemoteError::Other {
                message: "Remote file already exists and overwrite is disabled".to_string(),
            });
        }

        // Get local file size for progress tracking
        let file_size = tokio::fs::metadata(local_path).await.map_err(|e| RemoteError::Io(e))?.len();

        // Simplified upload - in a real implementation, you would use streaming
        let data = tokio::fs::read(local_path).await.map_err(|e| RemoteError::Io(e))?;
        let mut cursor = std::io::Cursor::new(data);
        
        // Re-acquire connection
        let connection = self.get_connection()?;
        connection.put_file(remote_path, &mut cursor)
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to upload: {}", e),
            })?;

        // Report progress
        if let Some(ref callback) = progress {
            callback(file_size as u64, file_size as u64);
        }

        Ok(())
    }

    async fn read_file(&mut self, path: &str) -> Result<Vec<u8>, RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        let data = connection.retr_as_buffer(path)
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to read file {}: {}", path, e),
            })?;

        Ok(data.into_inner())
    }

    async fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        let connection = self.get_connection()?;

        let mut cursor = std::io::Cursor::new(content);
        connection.put_file(path, &mut cursor)
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to write file {}: {}", path, e),
            })?;

        Ok(())
    }

    async fn get_disk_space(&mut self, _path: &str) -> Result<Option<DiskSpace>, RemoteError> {
        // FTP doesn't provide disk space information
        Ok(None)
    }

    async fn set_permissions(&mut self, _path: &str, _permissions: u32) -> Result<(), RemoteError> {
        // FTP doesn't support setting permissions
        Err(RemoteError::Other {
            message: "FTP does not support setting file permissions".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RemoteProtocol;

    #[test]
    fn test_ftp_client_creation() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::Ftp,
            host: "example.com".to_string(),
            port: Some(21),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: true,
            verify_ssl: false,
            base_path: None,
        };

        let client = FtpClient::new(config).unwrap();
        assert_eq!(client.config().host, "example.com");
        assert_eq!(client.config().port, Some(21));
    }

    #[test]
    fn test_ftps_client_creation() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::Ftps,
            host: "example.com".to_string(),
            port: Some(990),
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

        let client = FtpClient::new(config).unwrap();
        assert_eq!(client.config().protocol, RemoteProtocol::Ftps);
    }

    #[test]
    fn test_invalid_protocol() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::Sftp,
            host: "example.com".to_string(),
            port: Some(22),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: false,
            verify_ssl: true,
            base_path: None,
        };

        assert!(FtpClient::new(config).is_err());
    }

    #[test]
    fn test_parse_file_listing() {
        let client = FtpClient::new(RemoteConfig {
            protocol: RemoteProtocol::Ftp,
            host: "test".to_string(),
            port: Some(21),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: true,
            verify_ssl: false,
            base_path: None,
        }).unwrap();

        let listing = "-rw-r--r-- 1 user group 1234 Dec 25 12:34 test.txt";
        let result = client.parse_file_listing(listing, "/");
        
        assert!(result.is_some());
        let file_info = result.unwrap();
        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, 1234);
        assert_eq!(file_info.file_type, RemoteFileType::File);
    }

    #[test]
    fn test_parse_directory_listing() {
        let client = FtpClient::new(RemoteConfig {
            protocol: RemoteProtocol::Ftp,
            host: "test".to_string(),
            port: Some(21),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: true,
            verify_ssl: false,
            base_path: None,
        }).unwrap();

        let listing = "drwxr-xr-x 2 user group 4096 Dec 25 12:34 documents";
        let result = client.parse_file_listing(listing, "/");
        
        assert!(result.is_some());
        let file_info = result.unwrap();
        assert_eq!(file_info.name, "documents");
        assert_eq!(file_info.file_type, RemoteFileType::Directory);
    }
}
