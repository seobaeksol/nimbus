//! Core types and data structures for the plugin system

use serde::{Deserialize, Serialize};
use semver::Version;
use std::collections::HashMap;
use uuid::Uuid;

/// Plugin information and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginInfo {
    /// Plugin name (human-readable)
    pub name: String,
    /// Plugin version using semantic versioning
    pub version: Version,
    /// Short description of plugin functionality
    pub description: String,
    /// Plugin author/organization
    pub author: String,
    /// Optional website or repository URL
    pub website: Option<String>,
    /// Optional license information
    pub license: Option<String>,
    /// Minimum Nimbus version required
    pub min_nimbus_version: Option<Version>,
    /// Maximum Nimbus version supported
    pub max_nimbus_version: Option<Version>,
}

impl PluginInfo {
    /// Create a new PluginInfo with minimal required fields
    pub fn new(name: String, version: Version, description: String, author: String) -> Self {
        Self {
            name,
            version,
            description,
            author,
            website: None,
            license: None,
            min_nimbus_version: None,
            max_nimbus_version: None,
        }
    }

    /// Check if this plugin is compatible with the given Nimbus version
    pub fn is_compatible_with(&self, nimbus_version: &Version) -> bool {
        if let Some(min) = &self.min_nimbus_version {
            if nimbus_version < min {
                return false;
            }
        }
        
        if let Some(max) = &self.max_nimbus_version {
            if nimbus_version > max {
                return false;
            }
        }
        
        true
    }
}

/// Types of plugins supported by Nimbus
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginType {
    /// Plugins that extend file metadata and column display
    Content,
    /// Plugins that add support for remote file systems
    Protocol,
    /// Plugins that implement file viewers and editors
    Viewer,
}

impl PluginType {
    /// Get all supported plugin types
    pub fn all() -> Vec<PluginType> {
        vec![PluginType::Content, PluginType::Protocol, PluginType::Viewer]
    }

    /// Get a human-readable name for the plugin type
    pub fn display_name(&self) -> &'static str {
        match self {
            PluginType::Content => "Content Plugin",
            PluginType::Protocol => "Protocol Plugin", 
            PluginType::Viewer => "Viewer Plugin",
        }
    }

    /// Get a description of what this plugin type does
    pub fn description(&self) -> &'static str {
        match self {
            PluginType::Content => "Extends file metadata and column display functionality",
            PluginType::Protocol => "Adds support for custom remote file systems and protocols",
            PluginType::Viewer => "Implements custom file viewers, editors, and preview handlers",
        }
    }
}

/// Current status of a plugin
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is loaded and active
    Active,
    /// Plugin is loaded but disabled
    Inactive,
    /// Plugin failed to load due to errors
    Error,
    /// Plugin is being loaded
    Loading,
    /// Plugin is not loaded
    Unloaded,
}

impl PluginStatus {
    /// Check if the plugin is currently usable
    pub fn is_usable(&self) -> bool {
        matches!(self, PluginStatus::Active)
    }

    /// Check if the plugin has encountered an error
    pub fn is_error(&self) -> bool {
        matches!(self, PluginStatus::Error)
    }

    /// Get a human-readable description of the status
    pub fn description(&self) -> &'static str {
        match self {
            PluginStatus::Active => "Plugin is loaded and functioning",
            PluginStatus::Inactive => "Plugin is disabled by user",
            PluginStatus::Error => "Plugin failed to load or encountered errors",
            PluginStatus::Loading => "Plugin is being loaded",
            PluginStatus::Unloaded => "Plugin is not loaded",
        }
    }
}

/// Plugin manifest file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub info: PluginInfo,
    /// Plugin type
    pub plugin_type: PluginType,
    /// Entry point for the plugin (library file name)
    pub entry_point: String,
    /// Plugin configuration schema (JSON Schema)
    pub config_schema: Option<serde_json::Value>,
    /// Default configuration values
    pub default_config: Option<HashMap<String, serde_json::Value>>,
    /// List of permissions required by this plugin
    pub permissions: Vec<PluginPermission>,
    /// Dependencies on other plugins
    pub dependencies: Vec<PluginDependency>,
    /// Platform-specific information
    pub platforms: Vec<Platform>,
}

/// Permissions that a plugin can request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginPermission {
    /// Read access to file system
    FileSystemRead,
    /// Write access to file system
    FileSystemWrite,
    /// Network access for remote operations
    Network,
    /// Access to system information
    SystemInfo,
    /// Execute external programs
    Execute,
    /// Access to user configuration
    Config,
    /// Custom permission with description
    Custom(String),
}

impl PluginPermission {
    /// Get a human-readable description of the permission
    pub fn description(&self) -> String {
        match self {
            PluginPermission::FileSystemRead => "Read files and directories".to_string(),
            PluginPermission::FileSystemWrite => "Create, modify, and delete files".to_string(),
            PluginPermission::Network => "Access network resources".to_string(),
            PluginPermission::SystemInfo => "Access system information".to_string(),
            PluginPermission::Execute => "Execute external programs".to_string(),
            PluginPermission::Config => "Access and modify plugin configuration".to_string(),
            PluginPermission::Custom(desc) => desc.clone(),
        }
    }

    /// Check if this permission is considered high-risk
    pub fn is_high_risk(&self) -> bool {
        matches!(
            self,
            PluginPermission::FileSystemWrite | PluginPermission::Execute | PluginPermission::Network
        )
    }
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Name of the required plugin
    pub name: String,
    /// Version requirement (semver range)
    pub version_req: String,
    /// Whether this dependency is optional
    pub optional: bool,
}

/// Supported platforms for plugins
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    All,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Platform {
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Platform::All;
    }

    /// Check if this platform matches the current system
    pub fn is_compatible(&self) -> bool {
        matches!(self, Platform::All) || *self == Platform::current()
    }
}

/// Plugin instance tracking
#[derive(Debug, Clone)]
pub struct PluginInstance {
    /// Unique instance ID
    pub id: Uuid,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Current status
    pub status: PluginStatus,
    /// Plugin file path
    pub path: std::path::PathBuf,
    /// Last error message if status is Error
    pub last_error: Option<String>,
    /// Plugin configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Load timestamp
    pub loaded_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PluginInstance {
    /// Create a new plugin instance
    pub fn new(manifest: PluginManifest, path: std::path::PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            config: manifest.default_config.clone().unwrap_or_default(),
            manifest,
            status: PluginStatus::Unloaded,
            path,
            last_error: None,
            loaded_at: None,
        }
    }

    /// Get the plugin's unique identifier (name + version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.manifest.info.name, self.manifest.info.version)
    }

    /// Check if the plugin is compatible with current platform
    pub fn is_platform_compatible(&self) -> bool {
        self.manifest
            .platforms
            .iter()
            .any(|p| p.is_compatible())
    }
}