//! Protocol plugin interface for adding custom remote file system support

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{PluginInfo, Result};

/// Connection configuration for remote protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Protocol scheme (e.g., "ftp", "sftp", "webdav")
    pub scheme: String,
    /// Hostname or IP address
    pub host: String,
    /// Port number (optional, uses protocol default if not specified)
    pub port: Option<u16>,
    /// Username for authentication
    pub username: Option<String>,
    /// Password for authentication (should be stored securely)
    pub password: Option<String>,
    /// Path to private key file (for key-based authentication)
    pub private_key_path: Option<PathBuf>,
    /// Additional protocol-specific options
    pub options: HashMap<String, String>,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// Enable passive mode (FTP)
    pub passive_mode: Option<bool>,
    /// Use SSL/TLS encryption
    pub use_ssl: Option<bool>,
}

impl ConnectionConfig {
    /// Create a new connection configuration
    pub fn new(scheme: String, host: String) -> Self {
        Self {
            scheme,
            host,
            port: None,
            username: None,
            password: None,
            private_key_path: None,
            options: HashMap::new(),
            timeout: Some(30), // Default 30 second timeout
            passive_mode: None,
            use_ssl: None,
        }
    }
    
    /// Set authentication credentials
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }
    
    /// Set private key path for key-based authentication
    pub fn with_private_key(mut self, key_path: PathBuf) -> Self {
        self.private_key_path = Some(key_path);
        self
    }
    
    /// Add a protocol-specific option
    pub fn with_option(mut self, key: String, value: String) -> Self {
        self.options.insert(key, value);
        self
    }
    
    /// Get the full URL for this connection
    pub fn to_url(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);
        
        if let Some(port) = self.port {
            url.push_str(&format!(":{}", port));
        }
        
        url
    }
}

/// Remote file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFileInfo {
    /// File/directory name
    pub name: String,
    /// Full remote path
    pub path: String,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Last modified timestamp (ISO 8601 format)
    pub modified: Option<String>,
    /// File creation timestamp (ISO 8601 format)
    pub created: Option<String>,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File permissions (Unix-style)
    pub permissions: Option<String>,
    /// File owner
    pub owner: Option<String>,
    /// File group
    pub group: Option<String>,
    /// MIME type (if determinable)
    pub mime_type: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Transfer progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    /// Bytes transferred so far
    pub transferred: u64,
    /// Total bytes to transfer
    pub total: u64,
    /// Transfer speed in bytes per second
    pub speed: u64,
    /// Estimated time remaining in seconds
    pub eta: Option<u64>,
    /// Current file being transferred
    pub current_file: Option<String>,
    /// Transfer status
    pub status: TransferStatus,
}

impl TransferProgress {
    /// Calculate completion percentage (0.0 to 1.0)
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.transferred as f32) / (self.total as f32)
        }
    }
    
    /// Check if transfer is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, TransferStatus::Completed)
    }
    
    /// Check if transfer has failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, TransferStatus::Failed(_))
    }
}

/// Status of a transfer operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    /// Transfer is preparing
    Preparing,
    /// Transfer is in progress
    InProgress,
    /// Transfer is paused
    Paused,
    /// Transfer completed successfully
    Completed,
    /// Transfer was cancelled
    Cancelled,
    /// Transfer failed with error message
    Failed(String),
}

/// Options for transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOptions {
    /// Overwrite existing files
    pub overwrite: bool,
    /// Resume incomplete transfers
    pub resume: bool,
    /// Preserve file timestamps
    pub preserve_timestamps: bool,
    /// Preserve file permissions
    pub preserve_permissions: bool,
    /// Maximum number of concurrent transfers
    pub max_concurrent: Option<u32>,
    /// Buffer size for transfers in bytes
    pub buffer_size: Option<usize>,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            resume: true,
            preserve_timestamps: true,
            preserve_permissions: false,
            max_concurrent: Some(3),
            buffer_size: Some(64 * 1024), // 64KB default buffer
        }
    }
}

/// Remote client interface for protocol implementations
#[async_trait]
pub trait RemoteClient: Send + Sync {
    /// Connect to the remote server
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the remote server
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check if the connection is still active
    async fn is_connected(&self) -> bool;
    
    /// List files and directories in a remote path
    async fn list_directory(&self, path: &str) -> Result<Vec<RemoteFileInfo>>;
    
    /// Get information about a specific remote file/directory
    async fn get_file_info(&self, path: &str) -> Result<RemoteFileInfo>;
    
    /// Create a directory on the remote server
    async fn create_directory(&self, path: &str) -> Result<()>;
    
    /// Delete a file or directory on the remote server
    async fn delete(&self, path: &str, recursive: bool) -> Result<()>;
    
    /// Rename/move a file or directory on the remote server
    async fn rename(&self, from_path: &str, to_path: &str) -> Result<()>;
    
    /// Download a file from the remote server
    async fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: &TransferOptions,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()>;
    
    /// Upload a file to the remote server
    async fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: &TransferOptions,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()>;
    
    /// Download multiple files/directories
    async fn download_multiple(
        &self,
        items: Vec<(String, PathBuf)>, // (remote_path, local_path) pairs
        options: &TransferOptions,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()> {
        // Default implementation downloads files sequentially
        for (remote_path, local_path) in items {
            self.download_file(&remote_path, &local_path, options, progress_callback.as_ref().map(|cb| {
                let cb_clone: Box<dyn Fn(TransferProgress) + Send + Sync> = unsafe { std::mem::transmute_copy(cb) };
                cb_clone
            })).await?;
        }
        Ok(())
    }
    
    /// Upload multiple files/directories
    async fn upload_multiple(
        &self,
        items: Vec<(PathBuf, String)>, // (local_path, remote_path) pairs
        options: &TransferOptions,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()> {
        // Default implementation uploads files sequentially
        for (local_path, remote_path) in items {
            self.upload_file(&local_path, &remote_path, options, progress_callback.as_ref().map(|cb| {
                let cb_clone: Box<dyn Fn(TransferProgress) + Send + Sync> = unsafe { std::mem::transmute_copy(cb) };
                cb_clone
            })).await?;
        }
        Ok(())
    }
    
    /// Test the connection
    async fn test_connection(&self) -> Result<bool> {
        // Default implementation tries to list root directory
        match self.list_directory("/").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Get supported features for this protocol
    fn get_capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities::default()
    }
}

/// Capabilities supported by a protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    /// Supports directory creation
    pub can_create_directories: bool,
    /// Supports file/directory deletion
    pub can_delete: bool,
    /// Supports renaming/moving files
    pub can_rename: bool,
    /// Supports setting file permissions
    pub can_set_permissions: bool,
    /// Supports preserving timestamps
    pub can_preserve_timestamps: bool,
    /// Supports resuming interrupted transfers
    pub can_resume_transfers: bool,
    /// Supports concurrent transfers
    pub can_concurrent_transfers: bool,
    /// Maximum path length supported
    pub max_path_length: Option<usize>,
    /// Maximum file size supported
    pub max_file_size: Option<u64>,
}

impl Default for ProtocolCapabilities {
    fn default() -> Self {
        Self {
            can_create_directories: true,
            can_delete: true,
            can_rename: true,
            can_set_permissions: false,
            can_preserve_timestamps: true,
            can_resume_transfers: false,
            can_concurrent_transfers: false,
            max_path_length: None,
            max_file_size: None,
        }
    }
}

/// Protocol plugin trait
#[async_trait]
pub trait ProtocolPlugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;
    
    /// Get the protocol scheme this plugin handles (e.g., "ftp", "sftp")
    fn scheme(&self) -> String;
    
    /// Get default port for this protocol
    fn default_port(&self) -> u16;
    
    /// Get protocol capabilities
    fn capabilities(&self) -> ProtocolCapabilities;
    
    /// Create a new client instance for this protocol
    async fn create_client(&self, config: ConnectionConfig) -> Result<Box<dyn RemoteClient>>;
    
    /// Validate connection configuration
    async fn validate_config(&self, config: &ConnectionConfig) -> Result<()> {
        if config.scheme != self.scheme() {
            return Err(crate::PluginError::configuration_error(
                self.info().name,
                format!("Invalid scheme: expected '{}', got '{}'", self.scheme(), config.scheme)
            ));
        }
        
        if config.host.is_empty() {
            return Err(crate::PluginError::configuration_error(
                self.info().name,
                "Host cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get connection configuration template/defaults
    fn get_config_template(&self) -> ConnectionConfig {
        ConnectionConfig::new(self.scheme(), String::new())
            .with_option("port".to_string(), self.default_port().to_string())
    }
    
    /// Initialize the plugin
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Cleanup the plugin
    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Registry for protocol plugins
pub struct ProtocolPluginRegistry {
    plugins: HashMap<String, Box<dyn ProtocolPlugin>>,
}

impl ProtocolPluginRegistry {
    /// Create a new protocol plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a protocol plugin
    pub async fn register_plugin(&mut self, mut plugin: Box<dyn ProtocolPlugin>) -> Result<()> {
        let info = plugin.info();
        let scheme = plugin.scheme();
        
        // Check for scheme conflicts
        if self.plugins.contains_key(&scheme) {
            return Err(crate::PluginError::plugin_already_loaded(format!(
                "Protocol plugin for scheme '{}' is already registered",
                scheme
            )));
        }
        
        // Initialize the plugin
        plugin.initialize().await?;
        
        // Register the plugin
        self.plugins.insert(scheme.clone(), plugin);
        
        log::info!("Registered protocol plugin: {} for scheme '{}'", info.name, scheme);
        Ok(())
    }
    
    /// Unregister a protocol plugin
    pub async fn unregister_plugin(&mut self, scheme: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(scheme) {
            plugin.cleanup().await?;
            log::info!("Unregistered protocol plugin for scheme: {}", scheme);
        }
        Ok(())
    }
    
    /// Get a protocol plugin by scheme
    pub fn get_plugin(&self, scheme: &str) -> Option<&dyn ProtocolPlugin> {
        self.plugins.get(scheme).map(|p| p.as_ref())
    }
    
    /// Create a client for the given configuration
    pub async fn create_client(&self, config: ConnectionConfig) -> Result<Box<dyn RemoteClient>> {
        let plugin = self.get_plugin(&config.scheme)
            .ok_or_else(|| crate::PluginError::plugin_not_found(
                format!("No plugin found for scheme '{}'", config.scheme)
            ))?;
        
        plugin.validate_config(&config).await?;
        plugin.create_client(config).await
    }
    
    /// Get list of supported schemes
    pub fn get_supported_schemes(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
    
    /// Get plugin information for all registered plugins
    pub fn get_plugin_info(&self) -> Vec<(String, PluginInfo, ProtocolCapabilities)> {
        self.plugins
            .iter()
            .map(|(scheme, plugin)| (scheme.clone(), plugin.info(), plugin.capabilities()))
            .collect()
    }
}

impl Default for ProtocolPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}