//! WebDAV Protocol Plugin for Nimbus
//! 
//! This plugin provides WebDAV protocol support, enabling access to:
//! - WebDAV servers and file shares
//! - Cloud storage services (Nextcloud, ownCloud, etc.)
//! - Network-attached storage with WebDAV support
//! - Any HTTP/HTTPS server with WebDAV extensions

use async_trait::async_trait;
use nimbus_plugin_sdk::{
    ProtocolPlugin, RemoteClient, PluginInfo, Result, PluginError,
    protocol::{
        ConnectionConfig, RemoteFileInfo, ProtocolCapabilities, 
        TransferProgress, TransferStatus, TransferOptions
    }
};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use log::{debug, info, warn, error};
use reqwest::{Client, Method, StatusCode};
use url::Url;
use chrono::{DateTime, Utc};

pub struct WebDAVPlugin;

impl WebDAVPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProtocolPlugin for WebDAVPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "WebDAV Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "WebDAV protocol plugin for cloud storage and remote servers".to_string(),
            author: "Nimbus Team".to_string(),
            homepage: Some("https://github.com/nimbus-file-manager/plugins/webdav".to_string()),
            repository: Some("https://github.com/nimbus-file-manager/plugins".to_string()),
            license: Some("MIT".to_string()),
            tags: vec![
                "webdav".to_string(),
                "protocol".to_string(),
                "cloud".to_string(),
                "remote".to_string(),
                "nextcloud".to_string(),
                "owncloud".to_string(),
                "http".to_string(),
            ],
            min_version: "0.1.0".to_string(),
            max_version: None,
        }
    }
    
    fn scheme(&self) -> String {
        "webdav".to_string()
    }
    
    fn default_port(&self) -> u16 {
        443 // HTTPS by default
    }
    
    fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            can_create_directories: true,
            can_delete: true,
            can_rename: true,
            can_set_permissions: false, // WebDAV doesn't typically support Unix permissions
            can_preserve_timestamps: true,
            can_resume_transfers: false, // Could be implemented with Range headers
            can_concurrent_transfers: true,
            max_path_length: Some(2048),
            max_file_size: None, // Depends on server configuration
        }
    }
    
    async fn create_client(&self, config: ConnectionConfig) -> Result<Box<dyn RemoteClient>> {
        debug!("Creating WebDAV client for: {}", config.host);
        
        let client = WebDAVClient::new(config).await?;
        Ok(Box::new(client))
    }
    
    async fn validate_config(&self, config: &ConnectionConfig) -> Result<()> {
        debug!("Validating WebDAV configuration");
        
        // Check scheme
        if config.scheme != self.scheme() {
            return Err(PluginError::configuration_error(
                self.info().name,
                format!("Invalid scheme: expected 'webdav', got '{}'", config.scheme)
            ));
        }
        
        // Check host
        if config.host.is_empty() {
            return Err(PluginError::configuration_error(
                self.info().name,
                "Host cannot be empty".to_string()
            ));
        }
        
        // Validate URL format
        let base_url = if config.use_ssl.unwrap_or(true) {
            format!("https://{}:{}", config.host, config.port.unwrap_or(443))
        } else {
            format!("http://{}:{}", config.host, config.port.unwrap_or(80))
        };
        
        if Url::parse(&base_url).is_err() {
            return Err(PluginError::configuration_error(
                self.info().name,
                format!("Invalid host format: {}", config.host)
            ));
        }
        
        // Check credentials
        if config.username.is_none() {
            warn!("No username provided for WebDAV connection");
        }
        
        info!("WebDAV configuration validated successfully");
        Ok(())
    }
    
    fn get_config_template(&self) -> ConnectionConfig {
        let mut config = ConnectionConfig::new(self.scheme(), String::new());
        config.port = Some(443);
        config.use_ssl = Some(true);
        config.timeout = Some(30);
        
        // Add WebDAV-specific options
        config = config.with_option("dav_path".to_string(), "/remote.php/dav/files/".to_string()); // Nextcloud default
        config = config.with_option("user_agent".to_string(), "Nimbus-WebDAV/1.0".to_string());
        config = config.with_option("chunk_size".to_string(), "1048576".to_string()); // 1MB chunks
        
        config
    }
}

pub struct WebDAVClient {
    config: ConnectionConfig,
    client: Client,
    base_url: Url,
    connected: bool,
}

impl WebDAVClient {
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        debug!("Initializing WebDAV client");
        
        // Build base URL
        let scheme = if config.use_ssl.unwrap_or(true) { "https" } else { "http" };
        let port = config.port.unwrap_or(if scheme == "https" { 443 } else { 80 });
        let base_url_str = format!("{}://{}:{}", scheme, config.host, port);
        
        let base_url = Url::parse(&base_url_str)
            .map_err(|e| PluginError::configuration_error(
                "WebDAV Client".to_string(),
                format!("Invalid URL: {}", e)
            ))?;
        
        // Build HTTP client with timeout and authentication
        let mut client_builder = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout.unwrap_or(30)))
            .user_agent(config.options.get("user_agent").unwrap_or(&"Nimbus-WebDAV/1.0".to_string()));
        
        // Add basic authentication if credentials provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            debug!("Configuring basic authentication");
            client_builder = client_builder.basic_auth(username, Some(password));
        }
        
        let client = client_builder.build()
            .map_err(|e| PluginError::initialization_error(
                format!("Failed to create HTTP client: {}", e)
            ))?;
        
        Ok(Self {
            config,
            client,
            base_url,
            connected: false,
        })
    }
    
    /// Build URL for WebDAV path
    fn build_url(&self, path: &str) -> Result<Url> {
        let dav_path = self.config.options.get("dav_path").unwrap_or(&"/".to_string());
        let full_path = if path.starts_with('/') {
            format!("{}{}", dav_path.trim_end_matches('/'), path)
        } else {
            format!("{}/{}", dav_path.trim_end_matches('/'), path)
        };
        
        self.base_url.join(&full_path)
            .map_err(|e| PluginError::execution_error(
                "WebDAV Client".to_string(),
                format!("Failed to build URL for path '{}': {}", path, e)
            ))
    }
    
    /// Parse WebDAV PROPFIND response
    fn parse_propfind_response(&self, xml_body: &str) -> Result<Vec<RemoteFileInfo>> {
        debug!("Parsing PROPFIND response: {} bytes", xml_body.len());
        
        // This is a simplified XML parser for WebDAV responses
        // In a production implementation, you'd use a proper XML parser like quick-xml
        let mut files = Vec::new();
        
        // Mock parsing - in real implementation, parse XML properly
        if xml_body.contains("collection") {
            files.push(RemoteFileInfo {
                name: "sample-directory".to_string(),
                path: "/sample-directory/".to_string(),
                size: 0,
                modified: Some(Utc::now().to_rfc3339()),
                created: None,
                is_directory: true,
                permissions: None,
                owner: None,
                group: None,
                mime_type: None,
                metadata: HashMap::new(),
            });
        }
        
        files.push(RemoteFileInfo {
            name: "sample-file.txt".to_string(),
            path: "/sample-file.txt".to_string(),
            size: 1024,
            modified: Some(Utc::now().to_rfc3339()),
            created: None,
            is_directory: false,
            permissions: None,
            owner: None,
            group: None,
            mime_type: Some("text/plain".to_string()),
            metadata: HashMap::new(),
        });
        
        debug!("Parsed {} file entries", files.len());
        Ok(files)
    }
    
    /// Execute WebDAV PROPFIND request
    async fn propfind(&self, path: &str, depth: u32) -> Result<Vec<RemoteFileInfo>> {
        debug!("PROPFIND request for path: {} (depth: {})", path, depth);
        
        let url = self.build_url(path)?;
        let depth_header = if depth == 0 { "0" } else if depth == 1 { "1" } else { "infinity" };
        
        // WebDAV PROPFIND request body
        let propfind_body = r#"<?xml version="1.0" encoding="utf-8" ?>
        <D:propfind xmlns:D="DAV:">
            <D:allprop/>
        </D:propfind>"#;
        
        let response = self.client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), url)
            .header("Depth", depth_header)
            .header("Content-Type", "application/xml")
            .body(propfind_body)
            .send()
            .await
            .map_err(|e| PluginError::network_error(
                format!("PROPFIND request failed: {}", e)
            ))?;
        
        if !response.status().is_success() {
            return Err(PluginError::server_error(
                format!("PROPFIND failed with status: {}", response.status())
            ));
        }
        
        let xml_body = response.text().await
            .map_err(|e| PluginError::network_error(
                format!("Failed to read PROPFIND response: {}", e)
            ))?;
        
        self.parse_propfind_response(&xml_body)
    }
}

#[async_trait]
impl RemoteClient for WebDAVClient {
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebDAV server: {}", self.config.host);
        
        // Test connection with OPTIONS request
        let url = self.build_url("/")?;
        let response = self.client
            .request(Method::OPTIONS, url)
            .send()
            .await
            .map_err(|e| PluginError::connection_failed(
                format!("Connection test failed: {}", e)
            ))?;
        
        if !response.status().is_success() {
            return Err(PluginError::connection_failed(
                format!("Server returned status: {}", response.status())
            ));
        }
        
        // Check for WebDAV support
        if let Some(dav_header) = response.headers().get("dav") {
            debug!("WebDAV capabilities: {:?}", dav_header);
        } else {
            warn!("Server may not support WebDAV (no DAV header found)");
        }
        
        self.connected = true;
        info!("Successfully connected to WebDAV server");
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from WebDAV server");
        self.connected = false;
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        self.connected
    }
    
    async fn list_directory(&self, path: &str) -> Result<Vec<RemoteFileInfo>> {
        debug!("Listing directory: {}", path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        self.propfind(path, 1).await
    }
    
    async fn get_file_info(&self, path: &str) -> Result<RemoteFileInfo> {
        debug!("Getting file info for: {}", path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        let mut files = self.propfind(path, 0).await?;
        
        files.into_iter().next()
            .ok_or_else(|| PluginError::not_found(
                format!("File not found: {}", path)
            ))
    }
    
    async fn create_directory(&self, path: &str) -> Result<()> {
        info!("Creating directory: {}", path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        let url = self.build_url(path)?;
        let response = self.client
            .request(Method::from_bytes(b"MKCOL").unwrap(), url)
            .send()
            .await
            .map_err(|e| PluginError::network_error(
                format!("MKCOL request failed: {}", e)
            ))?;
        
        match response.status() {
            StatusCode::CREATED => {
                info!("Directory created successfully: {}", path);
                Ok(())
            }
            StatusCode::METHOD_NOT_ALLOWED => Err(PluginError::already_exists(
                format!("Directory already exists: {}", path)
            )),
            status => Err(PluginError::server_error(
                format!("Failed to create directory: {}", status)
            ))
        }
    }
    
    async fn delete(&self, path: &str, _recursive: bool) -> Result<()> {
        info!("Deleting: {}", path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        let url = self.build_url(path)?;
        let response = self.client
            .delete(url)
            .send()
            .await
            .map_err(|e| PluginError::network_error(
                format!("DELETE request failed: {}", e)
            ))?;
        
        if response.status().is_success() {
            info!("Successfully deleted: {}", path);
            Ok(())
        } else {
            Err(PluginError::server_error(
                format!("Failed to delete '{}': {}", path, response.status())
            ))
        }
    }
    
    async fn rename(&self, from_path: &str, to_path: &str) -> Result<()> {
        info!("Renaming: {} -> {}", from_path, to_path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        let from_url = self.build_url(from_path)?;
        let to_url = self.build_url(to_path)?;
        
        let response = self.client
            .request(Method::from_bytes(b"MOVE").unwrap(), from_url)
            .header("Destination", to_url.as_str())
            .header("Overwrite", "F") // Don't overwrite existing files
            .send()
            .await
            .map_err(|e| PluginError::network_error(
                format!("MOVE request failed: {}", e)
            ))?;
        
        match response.status() {
            StatusCode::CREATED | StatusCode::NO_CONTENT => {
                info!("Successfully renamed: {} -> {}", from_path, to_path);
                Ok(())
            }
            StatusCode::PRECONDITION_FAILED => Err(PluginError::already_exists(
                format!("Destination already exists: {}", to_path)
            )),
            status => Err(PluginError::server_error(
                format!("Failed to rename: {}", status)
            ))
        }
    }
    
    async fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
        _options: &TransferOptions,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()> {
        info!("Downloading: {} -> {:?}", remote_path, local_path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        let url = self.build_url(remote_path)?;
        
        // Send progress update
        if let Some(ref callback) = progress_callback {
            callback(TransferProgress {
                transferred: 0,
                total: 0, // We don't know the size yet
                speed: 0,
                eta: None,
                current_file: Some(remote_path.to_string()),
                status: TransferStatus::InProgress,
            });
        }
        
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| PluginError::network_error(
                format!("Download request failed: {}", e)
            ))?;
        
        if !response.status().is_success() {
            return Err(PluginError::server_error(
                format!("Download failed with status: {}", response.status())
            ));
        }
        
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        
        // Stream the response to file
        let mut file = tokio::fs::File::create(local_path).await
            .map_err(|e| PluginError::io_error(
                format!("Failed to create local file: {}", e)
            ))?;
        
        let mut stream = response.bytes_stream();
        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| PluginError::network_error(
                format!("Failed to read response chunk: {}", e)
            ))?;
            
            file.write_all(&chunk).await.map_err(|e| PluginError::io_error(
                format!("Failed to write to local file: {}", e)
            ))?;
            
            downloaded += chunk.len() as u64;
            
            // Send progress update
            if let Some(ref callback) = progress_callback {
                callback(TransferProgress {
                    transferred: downloaded,
                    total: total_size,
                    speed: 0, // TODO: Calculate actual speed
                    eta: None, // TODO: Calculate ETA
                    current_file: Some(remote_path.to_string()),
                    status: TransferStatus::InProgress,
                });
            }
        }
        
        file.flush().await.map_err(|e| PluginError::io_error(
            format!("Failed to flush file: {}", e)
        ))?;
        
        // Final progress update
        if let Some(ref callback) = progress_callback {
            callback(TransferProgress {
                transferred: downloaded,
                total: downloaded,
                speed: 0,
                eta: Some(0),
                current_file: Some(remote_path.to_string()),
                status: TransferStatus::Completed,
            });
        }
        
        info!("Download completed: {} bytes", downloaded);
        Ok(())
    }
    
    async fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        _options: &TransferOptions,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()> {
        info!("Uploading: {:?} -> {}", local_path, remote_path);
        
        if !self.connected {
            return Err(PluginError::not_connected());
        }
        
        let file_size = tokio::fs::metadata(local_path).await
            .map_err(|e| PluginError::io_error(
                format!("Failed to read local file metadata: {}", e)
            ))?
            .len();
        
        let file_data = tokio::fs::read(local_path).await
            .map_err(|e| PluginError::io_error(
                format!("Failed to read local file: {}", e)
            ))?;
        
        let url = self.build_url(remote_path)?;
        
        // Send progress update
        if let Some(ref callback) = progress_callback {
            callback(TransferProgress {
                transferred: 0,
                total: file_size,
                speed: 0,
                eta: None,
                current_file: Some(remote_path.to_string()),
                status: TransferStatus::InProgress,
            });
        }
        
        let response = self.client
            .put(url)
            .body(file_data)
            .send()
            .await
            .map_err(|e| PluginError::network_error(
                format!("Upload request failed: {}", e)
            ))?;
        
        match response.status() {
            StatusCode::CREATED | StatusCode::NO_CONTENT => {
                // Send completion update
                if let Some(ref callback) = progress_callback {
                    callback(TransferProgress {
                        transferred: file_size,
                        total: file_size,
                        speed: 0,
                        eta: Some(0),
                        current_file: Some(remote_path.to_string()),
                        status: TransferStatus::Completed,
                    });
                }
                
                info!("Upload completed: {} bytes", file_size);
                Ok(())
            }
            status => Err(PluginError::server_error(
                format!("Upload failed with status: {}", status)
            ))
        }
    }
    
    async fn test_connection(&self) -> Result<bool> {
        debug!("Testing WebDAV connection");
        
        let url = self.build_url("/")?;
        match self.client.request(Method::OPTIONS, url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    fn get_capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            can_create_directories: true,
            can_delete: true,
            can_rename: true,
            can_set_permissions: false,
            can_preserve_timestamps: true,
            can_resume_transfers: false,
            can_concurrent_transfers: true,
            max_path_length: Some(2048),
            max_file_size: None,
        }
    }
}

// Additional error types for WebDAV-specific errors
impl PluginError {
    pub fn connection_failed(message: String) -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: format!("Connection failed: {}", message),
        }
    }
    
    pub fn network_error(message: String) -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: format!("Network error: {}", message),
        }
    }
    
    pub fn server_error(message: String) -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: format!("Server error: {}", message),
        }
    }
    
    pub fn not_connected() -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: "Not connected to server".to_string(),
        }
    }
    
    pub fn not_found(message: String) -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: format!("Not found: {}", message),
        }
    }
    
    pub fn already_exists(message: String) -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: format!("Already exists: {}", message),
        }
    }
    
    pub fn io_error(message: String) -> Self {
        PluginError::ExecutionError {
            plugin: "WebDAV Plugin".to_string(),
            message: format!("I/O error: {}", message),
        }
    }
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn plugin_main() -> *mut dyn ProtocolPlugin {
    env_logger::init();
    info!("Creating WebDAV Plugin instance");
    let plugin = WebDAVPlugin::new();
    Box::into_raw(Box::new(plugin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio_test::tokio::test]
    async fn test_plugin_info() {
        let plugin = WebDAVPlugin::new();
        let info = plugin.info();
        
        assert_eq!(info.name, "WebDAV Plugin");
        assert_eq!(info.scheme(), "webdav");
        assert!(info.tags.contains(&"webdav".to_string()));
    }

    #[tokio_test::tokio::test]
    async fn test_config_validation() {
        let plugin = WebDAVPlugin::new();
        
        // Valid config
        let mut config = ConnectionConfig::new("webdav".to_string(), "example.com".to_string());
        assert!(plugin.validate_config(&config).await.is_ok());
        
        // Invalid scheme
        config.scheme = "ftp".to_string();
        assert!(plugin.validate_config(&config).await.is_err());
        
        // Empty host
        config.scheme = "webdav".to_string();
        config.host = String::new();
        assert!(plugin.validate_config(&config).await.is_err());
    }

    #[tokio_test::tokio::test]
    async fn test_url_building() {
        let config = ConnectionConfig::new("webdav".to_string(), "example.com".to_string())
            .with_option("dav_path".to_string(), "/webdav/".to_string());
        
        let client = WebDAVClient::new(config).await.unwrap();
        
        let url = client.build_url("/test/path").unwrap();
        assert!(url.as_str().contains("/webdav/test/path"));
        
        let url = client.build_url("relative/path").unwrap();
        assert!(url.as_str().contains("/webdav/relative/path"));
    }

    #[tokio_test::tokio::test]
    async fn test_capabilities() {
        let plugin = WebDAVPlugin::new();
        let caps = plugin.capabilities();
        
        assert!(caps.can_create_directories);
        assert!(caps.can_delete);
        assert!(caps.can_rename);
        assert!(!caps.can_set_permissions); // WebDAV limitation
        assert!(caps.can_concurrent_transfers);
    }
}