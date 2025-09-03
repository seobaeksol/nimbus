//! Secure Credential Storage
//!
//! Provides secure storage and retrieval of remote connection credentials using the OS keychain.
//! Supports Windows Credential Manager, macOS Keychain, and Linux Secret Service.

use crate::RemoteError;
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Secure credential storage manager
pub struct CredentialManager {
    service_name: String,
}

/// Stored credential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub connection_name: String,
    pub host: String,
    pub port: Option<u16>,
    pub protocol: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Credential storage result
pub type CredentialResult<T> = Result<T, RemoteError>;

impl CredentialManager {
    /// Create a new credential manager
    pub fn new(service_name: Option<String>) -> Self {
        Self {
            service_name: service_name.unwrap_or_else(|| "Nimbus-RemoteFS".to_string()),
        }
    }

    /// Store credentials securely in the OS keychain
    pub async fn store_credentials(
        &self,
        connection_id: &str,
        credentials: StoredCredentials,
    ) -> CredentialResult<()> {
        let entry = Entry::new(&self.service_name, connection_id)
            .map_err(|e| self.keyring_error_to_remote_error(e))?;

        let serialized = serde_json::to_string(&credentials)
            .map_err(|e| RemoteError::Other {
                message: format!("Failed to serialize credentials: {}", e),
            })?;

        entry.set_password(&serialized)
            .map_err(|e| self.keyring_error_to_remote_error(e))?;

        Ok(())
    }

    /// Retrieve credentials from the OS keychain
    pub async fn get_credentials(&self, connection_id: &str) -> CredentialResult<StoredCredentials> {
        let entry = Entry::new(&self.service_name, connection_id)
            .map_err(|e| self.keyring_error_to_remote_error(e))?;

        let serialized = entry.get_password()
            .map_err(|e| self.keyring_error_to_remote_error(e))?;

        let mut credentials: StoredCredentials = serde_json::from_str(&serialized)
            .map_err(|e| RemoteError::Other {
                message: format!("Failed to deserialize credentials: {}", e),
            })?;

        // Update last used timestamp
        credentials.last_used = Some(chrono::Utc::now());
        self.store_credentials(connection_id, credentials.clone()).await?;

        Ok(credentials)
    }

    /// Update existing credentials
    pub async fn update_credentials(
        &self,
        connection_id: &str,
        mut credentials: StoredCredentials,
    ) -> CredentialResult<()> {
        credentials.last_used = Some(chrono::Utc::now());
        self.store_credentials(connection_id, credentials).await
    }

    /// Remove credentials from the OS keychain
    pub async fn delete_credentials(&self, connection_id: &str) -> CredentialResult<()> {
        let entry = Entry::new(&self.service_name, connection_id)
            .map_err(|e| self.keyring_error_to_remote_error(e))?;

        entry.delete_password()
            .map_err(|e| self.keyring_error_to_remote_error(e))?;

        Ok(())
    }

    /// List all stored connection IDs (metadata only, no credentials)
    pub async fn list_connections(&self) -> CredentialResult<Vec<ConnectionMetadata>> {
        // Note: The keyring crate doesn't provide a way to list all entries
        // This would typically be implemented by maintaining a separate index
        // or using platform-specific APIs
        
        // For now, we return an empty list and suggest using a separate index
        // In a production implementation, you might:
        // 1. Keep a separate index file (encrypted)
        // 2. Use platform-specific APIs to enumerate keychain entries
        // 3. Maintain an in-memory cache of connection IDs
        
        Ok(Vec::new())
    }

    /// Check if credentials exist for a connection
    pub async fn has_credentials(&self, connection_id: &str) -> bool {
        let entry = match Entry::new(&self.service_name, connection_id) {
            Ok(entry) => entry,
            Err(_) => return false,
        };

        entry.get_password().is_ok()
    }

    /// Get connection metadata without retrieving sensitive credentials
    pub async fn get_connection_metadata(&self, connection_id: &str) -> CredentialResult<ConnectionMetadata> {
        let credentials = self.get_credentials(connection_id).await?;
        
        Ok(ConnectionMetadata {
            connection_id: connection_id.to_string(),
            connection_name: credentials.connection_name,
            host: credentials.host,
            port: credentials.port,
            protocol: credentials.protocol,
            username: credentials.username,
            has_password: credentials.password.is_some(),
            has_private_key: credentials.private_key_path.is_some(),
            created_at: credentials.created_at,
            last_used: credentials.last_used,
        })
    }

    /// Migrate credentials from old format or service name
    pub async fn migrate_credentials(
        &self,
        old_service_name: &str,
        connection_mappings: HashMap<String, String>,
    ) -> CredentialResult<usize> {
        let mut migrated_count = 0;

        for (old_id, new_id) in connection_mappings {
            // Try to retrieve from old service
            let old_entry = Entry::new(old_service_name, &old_id)
                .map_err(|e| self.keyring_error_to_remote_error(e))?;

            if let Ok(password) = old_entry.get_password() {
                if let Ok(credentials) = serde_json::from_str::<StoredCredentials>(&password) {
                    // Store in new service
                    self.store_credentials(&new_id, credentials).await?;
                    
                    // Delete from old service
                    let _ = old_entry.delete_password();
                    
                    migrated_count += 1;
                }
            }
        }

        Ok(migrated_count)
    }

    /// Convert keyring error to RemoteError
    fn keyring_error_to_remote_error(&self, error: KeyringError) -> RemoteError {
        match error {
            KeyringError::NoEntry => RemoteError::Other {
                message: "Credentials not found in keychain".to_string(),
            },
            KeyringError::PlatformFailure(e) => RemoteError::Other {
                message: format!("Keychain operation failed: {}", e),
            },
            KeyringError::Ambiguous(e) => RemoteError::Other {
                message: format!("Ambiguous keychain entry: {}", e),
            },
            KeyringError::Invalid(e) => RemoteError::Other {
                message: format!("Invalid keychain operation: {}", e),
            },
        }
    }
}

/// Connection metadata (non-sensitive information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub connection_id: String,
    pub connection_name: String,
    pub host: String,
    pub port: Option<u16>,
    pub protocol: String,
    pub username: String,
    pub has_password: bool,
    pub has_private_key: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Utility functions for credential management
pub mod utils {
    use super::*;
    use crate::RemoteConfig;

    /// Create credentials from RemoteConfig
    pub fn config_to_credentials(config: &RemoteConfig) -> StoredCredentials {
        StoredCredentials {
            username: config.username.clone(),
            password: config.password.clone(),
            private_key_path: config.private_key_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            private_key_passphrase: config.private_key_passphrase.clone(),
            connection_name: config.connection_name.clone(),
            host: config.host.clone(),
            port: config.port,
            protocol: format!("{:?}", config.protocol),
            created_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    /// Update RemoteConfig with stored credentials
    pub fn apply_credentials_to_config(
        config: &mut RemoteConfig,
        credentials: &StoredCredentials,
    ) {
        config.username = credentials.username.clone();
        config.password = credentials.password.clone();
        config.private_key_passphrase = credentials.private_key_passphrase.clone();
        
        if let Some(ref key_path) = credentials.private_key_path {
            config.private_key_path = Some(std::path::PathBuf::from(key_path));
        }
    }

    /// Generate a connection ID from config
    pub fn generate_connection_id(config: &RemoteConfig) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}@{}:{}", config.username, config.host, 
                config.port.unwrap_or(22)).hash(&mut hasher);
        config.protocol.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RemoteConfig, RemoteProtocol};
    use std::path::PathBuf;

    fn create_test_credentials() -> StoredCredentials {
        StoredCredentials {
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            private_key_path: Some("/path/to/key".to_string()),
            private_key_passphrase: Some("keypass".to_string()),
            connection_name: "Test Connection".to_string(),
            host: "test.example.com".to_string(),
            port: Some(22),
            protocol: "Sftp".to_string(),
            created_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    #[tokio::test]
    async fn test_credential_roundtrip() {
        let manager = CredentialManager::new(Some("nimbus-test".to_string()));
        let test_creds = create_test_credentials();
        let connection_id = "test-connection-1";

        // Store credentials
        assert!(manager.store_credentials(connection_id, test_creds.clone()).await.is_ok());

        // Retrieve credentials
        let retrieved = manager.get_credentials(connection_id).await.unwrap();
        assert_eq!(retrieved.username, test_creds.username);
        assert_eq!(retrieved.password, test_creds.password);
        assert_eq!(retrieved.host, test_creds.host);

        // Cleanup
        let _ = manager.delete_credentials(connection_id).await;
    }

    #[tokio::test]
    async fn test_credential_deletion() {
        let manager = CredentialManager::new(Some("nimbus-test".to_string()));
        let test_creds = create_test_credentials();
        let connection_id = "test-connection-2";

        // Store and verify
        assert!(manager.store_credentials(connection_id, test_creds).await.is_ok());
        assert!(manager.has_credentials(connection_id).await);

        // Delete and verify
        assert!(manager.delete_credentials(connection_id).await.is_ok());
        assert!(!manager.has_credentials(connection_id).await);
    }

    #[test]
    fn test_config_conversion() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::Sftp,
            host: "example.com".to_string(),
            port: Some(22),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: Some(PathBuf::from("/path/to/key")),
            private_key_passphrase: Some("keypass".to_string()),
            timeout: Some(30),
            connection_name: "Test".to_string(),
            use_passive_ftp: false,
            verify_ssl: true,
            base_path: None,
        };

        let credentials = utils::config_to_credentials(&config);
        assert_eq!(credentials.username, config.username);
        assert_eq!(credentials.password, config.password);
        assert_eq!(credentials.host, config.host);
    }

    #[test]
    fn test_connection_id_generation() {
        let config = RemoteConfig {
            protocol: RemoteProtocol::Sftp,
            host: "example.com".to_string(),
            port: Some(22),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            private_key_path: None,
            private_key_passphrase: None,
            timeout: Some(30),
            connection_name: "Test".to_string(),
            use_passive_ftp: false,
            verify_ssl: true,
            base_path: None,
        };

        let id1 = utils::generate_connection_id(&config);
        let id2 = utils::generate_connection_id(&config);
        assert_eq!(id1, id2); // Should be deterministic

        let mut config2 = config.clone();
        config2.host = "different.com".to_string();
        let id3 = utils::generate_connection_id(&config2);
        assert_ne!(id1, id3); // Different configs should have different IDs
    }
}