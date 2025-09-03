//! SFTP (SSH File Transfer Protocol) implementation
//!
//! Provides secure file transfer capabilities over SSH using the ssh2 crate.
//! Supports key-based and password authentication, file operations, and directory traversal.

use crate::{
    RemoteConfig, RemoteError, RemoteFileSystem, RemoteFileInfo, RemoteFileType, 
    RemotePermissions, TransferOptions, ProgressCallback, ConnectionStatus, DiskSpace
};
use async_trait::async_trait;
use ssh2::{Session, Sftp};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use chrono::{DateTime, Utc};
use tokio::task;

/// SFTP client implementation
pub struct SftpClient {
    config: RemoteConfig,
    session: Option<Session>,
    sftp: Option<Sftp>,
    status: ConnectionStatus,
    known_hosts_file: Option<PathBuf>,
}

impl SftpClient {
    /// Create a new SFTP client
    pub fn new(config: RemoteConfig) -> Result<Self, RemoteError> {
        if config.protocol != crate::RemoteProtocol::Sftp {
            return Err(RemoteError::Other {
                message: "Invalid protocol for SFTP client".to_string(),
            });
        }

        Ok(Self {
            config,
            session: None,
            sftp: None,
            status: ConnectionStatus::Disconnected,
            known_hosts_file: None,
        })
    }

    /// Get the default port for SFTP
    fn default_port(&self) -> u16 {
        22
    }

    /// Convert SSH2 file attributes to RemoteFileInfo (static version)
    fn convert_file_info_static(
        name: String,
        path: String,
        attrs: &ssh2::FileStat,
    ) -> RemoteFileInfo {
        let size = attrs.size.unwrap_or(0);
        let modified = attrs.mtime.map(|mtime| {
            DateTime::<Utc>::from_timestamp(mtime as i64, 0)
                .unwrap_or_else(|| Utc::now())
        });
        
        let file_type = if attrs.is_dir() {
            RemoteFileType::Directory
        } else if attrs.is_file() {
            RemoteFileType::File
        } else {
            RemoteFileType::Other
        };
        
        let permissions = attrs.perm.map(|perm| {
            RemotePermissions {
                read: (perm & 0o400) != 0,
                write: (perm & 0o200) != 0,
                execute: (perm & 0o100) != 0,
                mode: Some(perm),
            }
        });
        
        RemoteFileInfo {
            name: name.clone(),
            path,
            size,
            modified,
            created: None, // SSH2 doesn't provide creation time
            file_type,
            permissions,
            mime_type: crate::utils::get_extension(&name)
                .and_then(|ext| match ext.as_str() {
                    "txt" => Some("text/plain".to_string()),
                    "html" => Some("text/html".to_string()),
                    "json" => Some("application/json".to_string()),
                    "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                    "png" => Some("image/png".to_string()),
                    "pdf" => Some("application/pdf".to_string()),
                    "mp3" => Some("audio/mpeg".to_string()),
                    "mp4" => Some("video/mp4".to_string()),
                    _ => None,
                }),
            is_hidden: name.starts_with('.'),
            owner: attrs.uid.map(|uid| uid.to_string()),
            group: attrs.gid.map(|gid| gid.to_string()),
        }
    }
    
    /// Convert SSH2 file attributes to RemoteFileInfo (instance method)
    fn convert_file_info(
        &self,
        name: String,
        path: String,
        attrs: &ssh2::FileStat,
    ) -> RemoteFileInfo {
        Self::convert_file_info_static(name, path, attrs)
    }
    
    /// Helper method to remove directory recursively
    fn remove_directory_recursive(sftp: &Sftp, path: &str) -> Result<(), RemoteError> {
        let entries = sftp.readdir(std::path::Path::new(path)).map_err(|e| RemoteError::Other {
            message: format!("Failed to read directory {}: {}", path, e),
        })?;
        
        for (entry_path, stat) in entries {
            let entry_path_str = entry_path.to_string_lossy();
            
            if stat.is_dir() {
                Self::remove_directory_recursive(sftp, &entry_path_str)?;
            } else {
                sftp.unlink(&entry_path).map_err(|e| RemoteError::Other {
                    message: format!("Failed to remove file {}: {}", entry_path_str, e),
                })?;
            }
        }
        
        sftp.rmdir(std::path::Path::new(path)).map_err(|e| RemoteError::Other {
            message: format!("Failed to remove directory {}: {}", path, e),
        })
    }
    
    /// Helper method to create remote directories recursively
    fn create_remote_directories_recursive(sftp: &Sftp, path: &str) -> Result<(), RemoteError> {
        let mut current_path = String::new();
        for component in path.split('/').filter(|c| !c.is_empty()) {
            current_path.push('/');
            current_path.push_str(component);
            
            if sftp.stat(std::path::Path::new(&current_path)).is_err() {
                sftp.mkdir(std::path::Path::new(&current_path), 0o755).map_err(|e| {
                    RemoteError::Other {
                        message: format!("Failed to create remote directory {}: {}", current_path, e),
                    }
                })?;
            }
        }
        Ok(())
    }

    /// Create TCP connection to SSH server
    async fn create_tcp_connection(&self) -> Result<TcpStream, RemoteError> {
        let port = self.config.port.unwrap_or(self.default_port());
        let address = format!("{}:{}", self.config.host, port);
        
        let result = task::spawn_blocking(move || {
            let mut addrs = address.to_socket_addrs().map_err(|e| RemoteError::Other {
                message: format!("Failed to resolve address {}: {}", address, e),
            })?;
            
            let addr = addrs.next().ok_or_else(|| RemoteError::Other {
                message: format!("No address found for {}", address),
            })?;
            
            TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(30))
                .map_err(|e| RemoteError::ConnectionFailed {
                    message: format!("Failed to connect to {}: {}", addr, e),
                })
        }).await.map_err(|e| RemoteError::Other {
            message: format!("Task join error: {}", e),
        })??;
        
        Ok(result)
    }


    /// Ensure connection is active
    async fn ensure_connected(&mut self) -> Result<(), RemoteError> {
        if self.session.is_none() || self.sftp.is_none() {
            self.connect().await?;
        }
        Ok(())
    }

    /// Get SFTP connection reference
    fn get_sftp(&mut self) -> Result<&mut Sftp, RemoteError> {
        self.sftp.as_mut().ok_or_else(|| RemoteError::ConnectionFailed {
            message: "Not connected to SFTP server".to_string(),
        })
    }
    
    /// Get SSH session reference
    fn get_session(&mut self) -> Result<&mut Session, RemoteError> {
        self.session.as_mut().ok_or_else(|| RemoteError::ConnectionFailed {
            message: "SSH session not established".to_string(),
        })
    }
}

#[async_trait]
impl RemoteFileSystem for SftpClient {
    fn config(&self) -> &RemoteConfig {
        &self.config
    }

    async fn status(&self) -> ConnectionStatus {
        self.status.clone()
    }

    async fn connect(&mut self) -> Result<(), RemoteError> {
        self.status = ConnectionStatus::Connecting;

        // Create TCP connection
        let tcp = self.create_tcp_connection().await?;
        
        // Create and configure SSH session, authenticate, and create SFTP - all in one blocking task
        let (session, sftp) = {
            let username = self.config.username.clone();
            let password = self.config.password.clone();
            let private_key_path = self.config.private_key_path.clone();
            let passphrase = self.config.private_key_passphrase.clone();
            
            task::spawn_blocking(move || {
                // Create SSH session
                let mut session = Session::new().map_err(|e| RemoteError::ConnectionFailed {
                    message: format!("Failed to create SSH session: {}", e),
                })?;
                
                session.set_tcp_stream(tcp);
                session.handshake().map_err(|e| RemoteError::ConnectionFailed {
                    message: format!("SSH handshake failed: {}", e),
                })?;
                
                // Authenticate - try public key first if available
                if let Some(private_key_path) = private_key_path {
                    let public_key_path = private_key_path.with_extension("pub");
                    let result = session.userauth_pubkey_file(
                        &username,
                        Some(&public_key_path),
                        &private_key_path,
                        passphrase.as_deref(),
                    );
                    
                    if result.is_err() {
                        // Fall back to password if key auth fails
                        if let Some(password) = password {
                            session.userauth_password(&username, &password)
                                .map_err(|e| RemoteError::AuthenticationFailed {
                                    message: format!("Authentication failed: {}", e),
                                })?;
                        } else {
                            return Err(RemoteError::AuthenticationFailed {
                                message: "Key authentication failed and no password provided".to_string(),
                            });
                        }
                    }
                } else if let Some(password) = password {
                    session.userauth_password(&username, &password)
                        .map_err(|e| RemoteError::AuthenticationFailed {
                            message: format!("Password authentication failed: {}", e),
                        })?;
                } else {
                    return Err(RemoteError::AuthenticationFailed {
                        message: "No authentication method available".to_string(),
                    });
                }
                
                // Verify authentication
                if !session.authenticated() {
                    return Err(RemoteError::AuthenticationFailed {
                        message: "Authentication verification failed".to_string(),
                    });
                }
                
                // Create SFTP channel
                let sftp = session.sftp().map_err(|e| RemoteError::ConnectionFailed {
                    message: format!("Failed to create SFTP channel: {}", e),
                })?;
                
                Ok::<(Session, Sftp), RemoteError>((session, sftp))
            }).await.map_err(|e| RemoteError::Other {
                message: format!("Task join error during connection: {}", e),
            })??
        };
        
        // Store connections
        self.session = Some(session);
        self.sftp = Some(sftp);
        self.status = ConnectionStatus::Connected;
        
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), RemoteError> {
        // Drop SFTP channel first
        if let Some(sftp) = self.sftp.take() {
            drop(sftp);
        }
        
        // Close SSH session
        if let Some(session) = self.session.take() {
            let _ = task::spawn_blocking(move || {
                session.disconnect(None, "Client disconnect", None)
            }).await;
        }
        
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn test_connection(&self) -> Result<(), RemoteError> {
        // This would test connectivity without establishing a full connection
        Err(RemoteError::Other {
            message: "Test connection not yet implemented".to_string(),
        })
    }

    async fn list_directory(&mut self, path: &str) -> Result<Vec<RemoteFileInfo>, RemoteError> {
        self.ensure_connected().await?;
        
        // Create a scope to borrow self.sftp temporarily
        let entries = {
            let sftp = self.get_sftp()?;
            let path_buf = std::path::Path::new(path).to_path_buf();
            
            sftp.readdir(&path_buf).map_err(|e| RemoteError::Other {
                message: format!("Failed to list directory {}: {}", path, e),
            })?
        };
        
        let path = path.to_string();
        let result = task::spawn_blocking(move || {
            let mut file_infos = Vec::new();
            for (entry_path, stat) in entries {
                if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    let full_path = crate::utils::join_path(&path, name);
                    let file_info = Self::convert_file_info_static(name.to_string(), full_path, &stat);
                    file_infos.push(file_info);
                }
            }
            Ok::<Vec<RemoteFileInfo>, RemoteError>(file_infos)
        }).await.map_err(|e| RemoteError::Other {
            message: format!("Task join error during directory listing: {}", e),
        })??;
        
        Ok(result)
    }

    async fn get_file_info(&mut self, path: &str) -> Result<RemoteFileInfo, RemoteError> {
        self.ensure_connected().await?;
        
        let stat = {
            let sftp = self.get_sftp()?;
            sftp.stat(std::path::Path::new(path)).map_err(|_| RemoteError::FileNotFound {
                path: path.to_string(),
            })?
        };
        
        let path = path.to_string();
        let result = task::spawn_blocking(move || {
            let name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&path)
                .to_string();
            
            Self::convert_file_info_static(name, path, &stat)
        }).await.map_err(|e| RemoteError::Other {
            message: format!("Task join error getting file info: {}", e),
        })?;
        
        Ok(result)
    }

    async fn exists(&mut self, path: &str) -> Result<bool, RemoteError> {
        self.ensure_connected().await?;
        
        let result = {
            let sftp = self.get_sftp()?;
            sftp.stat(std::path::Path::new(path)).is_ok()
        };
        
        Ok(result)
    }

    async fn create_directory(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        
        let sftp = self.get_sftp()?;
        if recursive {
            // Create parent directories if they don't exist
            let mut current_path = String::new();
            for component in path.split('/').filter(|c| !c.is_empty()) {
                current_path.push('/');
                current_path.push_str(component);
                
                if sftp.stat(std::path::Path::new(&current_path)).is_err() {
                    sftp.mkdir(std::path::Path::new(&current_path), 0o755).map_err(|e| {
                        RemoteError::Other {
                            message: format!("Failed to create directory {}: {}", current_path, e),
                        }
                    })?;
                }
            }
        } else {
            sftp.mkdir(std::path::Path::new(path), 0o755).map_err(|e| {
                RemoteError::Other {
                    message: format!("Failed to create directory {}: {}", path, e),
                }
            })?;
        }
        
        Ok(())
    }

    async fn remove(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        
        let sftp = self.get_sftp()?;
        let stat = sftp.stat(std::path::Path::new(path)).map_err(|_| RemoteError::FileNotFound {
            path: path.to_string(),
        })?;
        
        if stat.is_dir() {
            if recursive {
                Self::remove_directory_recursive(sftp, path)?;
            } else {
                sftp.rmdir(std::path::Path::new(path)).map_err(|e| RemoteError::Other {
                    message: format!("Failed to remove directory {}: {}", path, e),
                })?;
            }
        } else {
            sftp.unlink(std::path::Path::new(path)).map_err(|e| RemoteError::Other {
                message: format!("Failed to remove file {}: {}", path, e),
            })?;
        }
        
        Ok(())
    }

    async fn rename(&mut self, from: &str, to: &str) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        
        let sftp = self.get_sftp()?;
        sftp.rename(std::path::Path::new(from), std::path::Path::new(to), None)
            .map_err(|e| RemoteError::Other {
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
        
        // Get remote file info first
        let (remote_stat, mut remote_file) = {
            let sftp = self.get_sftp()?;
            let remote_stat = sftp.stat(std::path::Path::new(remote_path)).map_err(|_| {
                RemoteError::FileNotFound { path: remote_path.to_string() }
            })?;
            
            let remote_file = sftp.open(std::path::Path::new(remote_path)).map_err(|e| {
                RemoteError::Other {
                    message: format!("Failed to open remote file {}: {}", remote_path, e),
                }
            })?;
            
            (remote_stat, remote_file)
        };
        
        let total_size = remote_stat.size.unwrap_or(0);
        
        // Check if local file exists and handle overwrite
        if local_path.exists() && !options.overwrite {
            return Err(RemoteError::Other {
                message: format!("Local file already exists: {}", local_path.display()),
            });
        }
        
        // Create parent directories if needed
        if options.create_directories {
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| RemoteError::Other {
                    message: format!("Failed to create local directory: {}", e),
                })?;
            }
        }
        
        // Create local file and copy data
        let mut local_file = std::fs::File::create(local_path).map_err(|e| RemoteError::Other {
            message: format!("Failed to create local file {}: {}", local_path.display(), e),
        })?;
        
        let mut buffer = vec![0u8; options.buffer_size.unwrap_or(8192)];
        let mut bytes_transferred = 0u64;
        
        loop {
            let bytes_read = remote_file.read(&mut buffer).map_err(|e| RemoteError::Other {
                message: format!("Failed to read from remote file: {}", e),
            })?;
            
            if bytes_read == 0 {
                break;
            }
            
            local_file.write_all(&buffer[..bytes_read]).map_err(|e| RemoteError::Other {
                message: format!("Failed to write to local file: {}", e),
            })?;
            
            bytes_transferred += bytes_read as u64;
            
            if let Some(ref callback) = progress {
                callback(bytes_transferred, total_size);
            }
        }
        
        // Set timestamps if requested
        if options.preserve_timestamps {
            if let Some(mtime) = remote_stat.mtime {
                let _ = filetime::set_file_mtime(
                    local_path,
                    filetime::FileTime::from_unix_time(mtime as i64, 0)
                );
            }
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
        
        // Get local file metadata
        let local_metadata = std::fs::metadata(local_path).map_err(|e| RemoteError::Other {
            message: format!("Failed to read local file metadata: {}", e),
        })?;
        let total_size = local_metadata.len();
        
        // Check if remote file exists and handle overwrite
        {
            let sftp = self.get_sftp()?;
            if !options.overwrite && sftp.stat(std::path::Path::new(remote_path)).is_ok() {
                return Err(RemoteError::Other {
                    message: format!("Remote file already exists: {}", remote_path),
                });
            }
            
            // Create remote directories if needed
            if options.create_directories {
                let remote_dir = crate::utils::get_parent_dir(remote_path);
                if !remote_dir.is_empty() && remote_dir != "/" {
                    let _ = Self::create_remote_directories_recursive(sftp, &remote_dir);
                }
            }
        }
        
        // Open files
        let mut local_file = std::fs::File::open(local_path).map_err(|e| RemoteError::Other {
            message: format!("Failed to open local file {}: {}", local_path.display(), e),
        })?;
        
        let mut remote_file = {
            let sftp = self.get_sftp()?;
            sftp.create(std::path::Path::new(remote_path)).map_err(|e| {
                RemoteError::Other {
                    message: format!("Failed to create remote file {}: {}", remote_path, e),
                }
            })?
        };
        
        // Copy data with progress reporting
        let mut buffer = vec![0u8; options.buffer_size.unwrap_or(8192)];
        let mut bytes_transferred = 0u64;
        
        loop {
            let bytes_read = local_file.read(&mut buffer).map_err(|e| RemoteError::Other {
                message: format!("Failed to read from local file: {}", e),
            })?;
            
            if bytes_read == 0 {
                break;
            }
            
            remote_file.write_all(&buffer[..bytes_read]).map_err(|e| RemoteError::Other {
                message: format!("Failed to write to remote file: {}", e),
            })?;
            
            bytes_transferred += bytes_read as u64;
            
            if let Some(ref callback) = progress {
                callback(bytes_transferred, total_size);
            }
        }
        
        // Set timestamps if requested
        if options.preserve_timestamps {
            if let Ok(mtime) = local_metadata.modified() {
                if let Ok(duration) = mtime.duration_since(UNIX_EPOCH) {
                    let sftp = self.get_sftp()?;
                    let _ = sftp.setstat(
                        std::path::Path::new(remote_path),
                        ssh2::FileStat {
                            size: None,
                            uid: None,
                            gid: None,
                            perm: None,
                            atime: None,
                            mtime: Some(duration.as_secs()),
                        }
                    );
                }
            }
        }
        
        Ok(())
    }

    async fn read_file(&mut self, path: &str) -> Result<Vec<u8>, RemoteError> {
        self.ensure_connected().await?;
        
        let sftp = self.get_sftp()?;
        let mut file = sftp.open(std::path::Path::new(path)).map_err(|_| {
            RemoteError::FileNotFound { path: path.to_string() }
        })?;
        
        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(|e| RemoteError::Other {
            message: format!("Failed to read file {}: {}", path, e),
        })?;
        
        Ok(content)
    }

    async fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        
        let sftp = self.get_sftp()?;
        let mut file = sftp.create(std::path::Path::new(path)).map_err(|e| RemoteError::Other {
            message: format!("Failed to create file {}: {}", path, e),
        })?;
        
        file.write_all(content).map_err(|e| RemoteError::Other {
            message: format!("Failed to write to file {}: {}", path, e),
        })?;
        
        Ok(())
    }

    async fn get_disk_space(&mut self, _path: &str) -> Result<Option<DiskSpace>, RemoteError> {
        // SFTP doesn't typically provide disk space information
        Ok(None)
    }

    async fn set_permissions(&mut self, path: &str, permissions: u32) -> Result<(), RemoteError> {
        self.ensure_connected().await?;
        
        let sftp = self.get_sftp()?;
        sftp.setstat(
            std::path::Path::new(path),
            ssh2::FileStat {
                size: None,
                uid: None,
                gid: None,
                perm: Some(permissions),
                atime: None,
                mtime: None,
            }
        ).map_err(|e| RemoteError::Other {
            message: format!("Failed to set permissions for {}: {}", path, e),
        })?;
        
        Ok(())
    }
}

/// SFTP-specific utility functions
impl SftpClient {
    /// Parse SSH private key from file
    pub async fn load_private_key(&self, key_path: &Path, _passphrase: Option<&str>) -> Result<String, RemoteError> {
        let key_content = tokio::fs::read_to_string(key_path).await.map_err(|e| {
            RemoteError::Other {
                message: format!("Failed to read private key: {}", e),
            }
        })?;

        // In a real implementation, you would parse and decrypt the key if needed
        Ok(key_content)
    }

    /// Get host key fingerprint
    pub async fn get_host_key_fingerprint(&self) -> Result<String, RemoteError> {
        // This would connect and get the host key fingerprint for verification
        Err(RemoteError::Other {
            message: "Host key fingerprint not yet implemented".to_string(),
        })
    }

    /// Set known hosts file path
    pub fn set_known_hosts_file(&mut self, _path: std::path::PathBuf) {
        // This would configure the known hosts file for host verification
        // Implementation would store this for use during connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RemoteProtocol;

    #[test]
    fn test_sftp_client_creation() {
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

        let client = SftpClient::new(config).unwrap();
        assert_eq!(client.config().host, "example.com");
        assert_eq!(client.config().port, Some(22));
    }

    #[test]
    fn test_invalid_protocol() {
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

        assert!(SftpClient::new(config).is_err());
    }

    #[tokio::test]
    async fn test_initial_status() {
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

        let client = SftpClient::new(config).unwrap();
        assert_eq!(client.status().await, ConnectionStatus::Disconnected);
    }
}