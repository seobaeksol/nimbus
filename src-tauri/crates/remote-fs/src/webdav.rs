//! WebDAV (Web Distributed Authoring and Versioning) implementation
//!
//! Provides WebDAV and WebDAVS support using HTTP/HTTPS requests.
//! Supports basic file operations, directory listing, and authentication.

use crate::{
    RemoteConfig, RemoteError, RemoteFileSystem, RemoteFileInfo, RemoteFileType, 
    RemotePermissions, TransferOptions, ProgressCallback, ConnectionStatus, DiskSpace
};
use async_trait::async_trait;
use reqwest::{Client, Method, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH}};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use xml::reader::{EventReader, XmlEvent};

/// WebDAV client implementation
pub struct WebDavClient {
    config: RemoteConfig,
    client: Client,
    base_url: String,
    status: ConnectionStatus,
}

impl WebDavClient {
    /// Create a new WebDAV client
    pub fn new(config: RemoteConfig) -> Result<Self, RemoteError> {
        if config.protocol != crate::RemoteProtocol::WebDav && config.protocol != crate::RemoteProtocol::WebDavs {
            return Err(RemoteError::Other {
                message: "Invalid protocol for WebDAV client".to_string(),
            });
        }

        let scheme = match config.protocol {
            crate::RemoteProtocol::WebDav => "http",
            crate::RemoteProtocol::WebDavs => "https",
            _ => "https",
        };

        let port = config.port.unwrap_or_else(|| {
            match config.protocol {
                crate::RemoteProtocol::WebDav => 80,
                crate::RemoteProtocol::WebDavs => 443,
                _ => 443,
            }
        });

        let base_url = format!("{}://{}:{}", scheme, config.host, port);
        
        // Create HTTP client with basic auth
        let mut headers = HeaderMap::new();
        if let Some(password) = &config.password {
            let auth = base64::encode(format!("{}:{}", config.username, password));
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", auth))
                .map_err(|e| RemoteError::Other {
                    message: format!("Failed to create auth header: {}", e),
                })?);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.timeout.unwrap_or(30)))
            .build()
            .map_err(|e| RemoteError::Other {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            config,
            client,
            base_url,
            status: ConnectionStatus::Disconnected,
        })
    }

    /// Build full URL for a path
    fn build_url(&self, path: &str) -> String {
        let base_path = self.config.base_path.as_deref().unwrap_or("/");
        let full_path = if path.starts_with('/') {
            format!("{}{}", base_path, path)
        } else {
            format!("{}/{}", base_path, path)
        };
        
        format!("{}{}", self.base_url, full_path)
    }

    /// Parse WebDAV PROPFIND response
    fn parse_propfind_response(&self, xml_content: &str) -> Result<Vec<RemoteFileInfo>, RemoteError> {
        let mut files = Vec::new();
        let parser = EventReader::from_str(xml_content);
        let mut depth = 0;
        let mut current_file: Option<RemoteFileInfo> = None;
        let mut current_path = String::new();
        let mut current_name = String::new();
        let mut current_size = 0u64;
        let mut current_type = RemoteFileType::File;

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, attributes: _, .. }) => {
                    depth += 1;
                    match name.local_name.as_str() {
                        "response" => {
                            current_file = Some(RemoteFileInfo {
                                name: String::new(),
                                path: String::new(),
                                size: 0,
                                modified: None,
                                created: None,
                                file_type: RemoteFileType::File,
                                permissions: None,
                                mime_type: None,
                                is_hidden: false,
                                owner: None,
                                group: None,
                            });
                        }
                        "href" => {
                            // Will be filled in text content
                        }
                        "getcontentlength" => {
                            // Will be filled in text content
                        }
                        "resourcetype" => {
                            // Check if it's a collection (directory)
                            current_type = RemoteFileType::Directory;
                        }
                        _ => {}
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    match name.local_name.as_str() {
                        "response" => {
                            if let Some(file) = current_file.take() {
                                if !file.path.is_empty() && file.path != "/" {
                                    // Skip root directory
                                    if !file.name.is_empty() {
                                        files.push(file);
                                    }
                                }
                            }
                        }
                        "href" => {
                            if let Some(ref mut file) = current_file {
                                file.path = current_path.clone();
                                file.name = current_name.clone();
                            }
                        }
                        "getcontentlength" => {
                            if let Some(ref mut file) = current_file {
                                file.size = current_size;
                            }
                        }
                        "resourcetype" => {
                            if let Some(ref mut file) = current_file {
                                file.file_type = current_type.clone();
                            }
                        }
                        _ => {}
                    }
                    depth -= 1;
                }
                Ok(XmlEvent::Characters(text)) => {
                    match depth {
                        2 => {
                            if text.contains("href") {
                                current_path = text.trim().to_string();
                                current_name = std::path::Path::new(&current_path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("")
                                    .to_string();
                            }
                        }
                        3 => {
                            if text.trim().parse::<u64>().is_ok() {
                                current_size = text.trim().parse().unwrap_or(0);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(RemoteError::ProtocolError {
                        message: format!("XML parsing error: {}", e),
                    });
                }
                _ => {}
            }
        }

        Ok(files)
    }

    /// Ensure connection is active
    async fn ensure_connected(&mut self) -> Result<(), RemoteError> {
        if self.status == ConnectionStatus::Disconnected {
            self.connect().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl RemoteFileSystem for WebDavClient {
    fn config(&self) -> &RemoteConfig {
        &self.config
    }

    async fn status(&self) -> ConnectionStatus {
        self.status.clone()
    }

    async fn connect(&mut self) -> Result<(), RemoteError> {
        self.status = ConnectionStatus::Connecting;

        // Test connection with a PROPFIND request
        let test_url = self.build_url("/");
        let response = self.client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &test_url)
            .header("Depth", "0")
            .send()
            .await
            .map_err(|e| RemoteError::ConnectionFailed {
                message: format!("Failed to connect to WebDAV server: {}", e),
            })?;

        if response.status().is_success() {
            self.status = ConnectionStatus::Connected;
            Ok(())
        } else {
            self.status = ConnectionStatus::Error(format!("HTTP {}", response.status()));
            Err(RemoteError::ConnectionFailed {
                message: format!("WebDAV server returned status: {}", response.status()),
            })
        }
    }

    async fn disconnect(&mut self) -> Result<(), RemoteError> {
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn test_connection(&self) -> Result<(), RemoteError> {
        let test_url = self.build_url("/");
        let response = self.client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &test_url)
            .header("Depth", "0")
            .send()
            .await
            .map_err(|e| RemoteError::ConnectionFailed {
                message: format!("Failed to test WebDAV connection: {}", e),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(RemoteError::ConnectionFailed {
                message: format!("WebDAV test failed with status: {}", response.status()),
            })
        }
    }

    async fn list_directory(&mut self, path: &str) -> Result<Vec<RemoteFileInfo>, RemoteError> {
        self.ensure_connected().await?;

        let url = self.build_url(path);
        let response = self.client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:getcontentlength/>
    <D:resourcetype/>
    <D:getlastmodified/>
  </D:prop>
</D:propfind>"#)
            .send()
            .await
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to list directory {}: {}", path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::ProtocolError {
                message: format!("WebDAV server returned status: {}", response.status()),
            });
        }

        let xml_content = response.text().await.map_err(|e| RemoteError::ProtocolError {
            message: format!("Failed to read response: {}", e),
        })?;

        self.parse_propfind_response(&xml_content)
    }

    async fn get_file_info(&mut self, path: &str) -> Result<RemoteFileInfo, RemoteError> {
        self.ensure_connected().await?;

        let url = self.build_url(path);
        let response = self.client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:getcontentlength/>
    <D:resourcetype/>
    <D:getlastmodified/>
  </D:prop>
</D:propfind>"#)
            .send()
            .await
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to get file info for {}: {}", path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::FileNotFound {
                path: path.to_string(),
            });
        }

        let xml_content = response.text().await.map_err(|e| RemoteError::ProtocolError {
            message: format!("Failed to read response: {}", e),
        })?;

        let files = self.parse_propfind_response(&xml_content)?;
        files.into_iter().next().ok_or_else(|| RemoteError::FileNotFound {
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

        if recursive {
            // Create parent directories first
            let parent = crate::utils::get_parent_dir(path);
            if parent != "/" && parent != path {
                self.create_directory(&parent, true).await?;
            }
        }

        let url = self.build_url(path);
        let response = self.client
            .request(Method::from_bytes(b"MKCOL").unwrap(), &url)
            .send()
            .await
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to create directory {}: {}", path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::ProtocolError {
                message: format!("Failed to create directory {}: HTTP {}", path, response.status()),
            });
        }

        Ok(())
    }

    async fn remove(&mut self, path: &str, recursive: bool) -> Result<(), RemoteError> {
        self.ensure_connected().await?;

        if recursive {
            // List directory contents and remove them first
            if let Ok(files) = self.list_directory(path).await {
                for file in files {
                    let file_path = crate::utils::join_path(path, &file.name);
                    self.remove(&file_path, true).await?;
                }
            }
        }

        let url = self.build_url(path);
        let response = self.client
            .request(Method::DELETE, &url)
            .send()
            .await
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to remove {}: {}", path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::ProtocolError {
                message: format!("Failed to remove {}: HTTP {}", path, response.status()),
            });
        }

        Ok(())
    }

    async fn rename(&mut self, from: &str, to: &str) -> Result<(), RemoteError> {
        self.ensure_connected().await?;

        let from_url = self.build_url(from);
        let to_url = self.build_url(to);
        
        let response = self.client
            .request(Method::from_bytes(b"MOVE").unwrap(), &from_url)
            .header("Destination", &to_url)
            .send()
            .await
            .map_err(|e| RemoteError::ProtocolError {
                message: format!("Failed to rename {} to {}: {}", from, to, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::ProtocolError {
                message: format!("Failed to rename {} to {}: HTTP {}", from, to, response.status()),
            });
        }

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

        let url = self.build_url(remote_path);
        let mut response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to download {}: {}", remote_path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::TransferFailed {
                message: format!("Download failed with status: {}", response.status()),
            });
        }

        let mut local_file = File::create(local_path).await.map_err(|e| RemoteError::Io(e))?;
        let mut downloaded = 0u64;
        let total_size = response.content_length().unwrap_or(0);

        while let Some(chunk) = response.chunk().await.map_err(|e| RemoteError::TransferFailed {
            message: format!("Failed to read chunk: {}", e),
        })? {
            local_file.write_all(&chunk).await.map_err(|e| RemoteError::Io(e))?;
            downloaded += chunk.len() as u64;

            // Report progress
            if let Some(ref callback) = progress {
                callback(downloaded, total_size);
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

        // Check if remote file exists and handle overwrite
        if !options.overwrite && self.exists(remote_path).await? {
            return Err(RemoteError::Other {
                message: "Remote file already exists and overwrite is disabled".to_string(),
            });
        }

        let file_size = tokio::fs::metadata(local_path).await.map_err(|e| RemoteError::Io(e))?.len();
        let mut file = File::open(local_path).await.map_err(|e| RemoteError::Io(e))?;

        let url = self.build_url(remote_path);
        let mut uploaded = 0u64;
        let mut buffer = vec![0u8; options.buffer_size.unwrap_or(8192)];

        loop {
            let bytes_read = file.read(&mut buffer).await.map_err(|e| RemoteError::Io(e))?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            let response = self.client
                .put(&url)
                .header(CONTENT_LENGTH, chunk.len())
                .body(chunk.to_vec())
                .send()
                .await
                .map_err(|e| RemoteError::TransferFailed {
                    message: format!("Failed to upload {}: {}", remote_path, e),
                })?;

            if !response.status().is_success() {
                return Err(RemoteError::TransferFailed {
                    message: format!("Upload failed with status: {}", response.status()),
                });
            }

            uploaded += bytes_read as u64;

            // Report progress
            if let Some(ref callback) = progress {
                callback(uploaded, file_size);
            }
        }

        Ok(())
    }

    async fn read_file(&mut self, path: &str) -> Result<Vec<u8>, RemoteError> {
        self.ensure_connected().await?;

        let url = self.build_url(path);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to read file {}: {}", path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::FileNotFound {
                path: path.to_string(),
            });
        }

        let data = response.bytes().await.map_err(|e| RemoteError::TransferFailed {
            message: format!("Failed to read file content: {}", e),
        })?;

        Ok(data.to_vec())
    }

    async fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), RemoteError> {
        self.ensure_connected().await?;

        let url = self.build_url(path);
        let response = self.client
            .put(&url)
            .header(CONTENT_LENGTH, content.len())
            .body(content.to_vec())
            .send()
            .await
            .map_err(|e| RemoteError::TransferFailed {
                message: format!("Failed to write file {}: {}", path, e),
            })?;

        if !response.status().is_success() {
            return Err(RemoteError::TransferFailed {
                message: format!("Write failed with status: {}", response.status()),
            });
        }

        Ok(())
    }

    async fn get_disk_space(&mut self, _path: &str) -> Result<Option<DiskSpace>, RemoteError> {
        // WebDAV doesn't typically provide disk space information
        Ok(None)
    }

    async fn set_permissions(&mut self, _path: &str, _permissions: u32) -> Result<(), RemoteError> {
        // WebDAV doesn't support setting Unix-style permissions
        Err(RemoteError::Other {
            message: "WebDAV does not support setting file permissions".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RemoteProtocol;

    #[test]
    fn test_webdav_client_creation() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::WebDav,
            host: "example.com".to_string(),
            port: Some(80),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: false,
            verify_ssl: false,
            base_path: None,
        };

        let client = WebDavClient::new(config).unwrap();
        assert_eq!(client.config().host, "example.com");
        assert_eq!(client.config().port, Some(80));
    }

    #[test]
    fn test_webdavs_client_creation() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::WebDavs,
            host: "example.com".to_string(),
            port: Some(443),
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

        let client = WebDavClient::new(config).unwrap();
        assert_eq!(client.config().protocol, RemoteProtocol::WebDavs);
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

        assert!(WebDavClient::new(config).is_err());
    }

    #[test]
    fn test_build_url() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::WebDav,
            host: "example.com".to_string(),
            port: Some(80),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "test".to_string(),
            use_passive_ftp: false,
            verify_ssl: false,
            base_path: Some("/webdav".to_string()),
        };

        let client = WebDavClient::new(config).unwrap();
        assert_eq!(client.build_url("/test"), "http://example.com:80/webdav/test");
        assert_eq!(client.build_url("test"), "http://example.com:80/webdav/test");
    }
}
