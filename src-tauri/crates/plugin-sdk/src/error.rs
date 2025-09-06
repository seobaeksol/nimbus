//! Error types for the plugin system

use thiserror::Error;
use std::path::PathBuf;

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>;

/// Comprehensive error types for plugin operations
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin loading errors
    #[error("Failed to load plugin from {path}: {source}")]
    LoadError {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Plugin manifest parsing errors
    #[error("Invalid plugin manifest in {path}: {message}")]
    ManifestError {
        path: PathBuf,
        message: String,
    },

    /// Version compatibility errors
    #[error("Plugin {plugin_name} version {plugin_version} is not compatible with Nimbus {nimbus_version}")]
    VersionIncompatible {
        plugin_name: String,
        plugin_version: String,
        nimbus_version: String,
    },

    /// Platform compatibility errors
    #[error("Plugin {plugin_name} is not compatible with current platform")]
    PlatformIncompatible {
        plugin_name: String,
    },

    /// Missing dependency errors
    #[error("Plugin {plugin_name} requires dependency {dependency_name} {version_req}")]
    MissingDependency {
        plugin_name: String,
        dependency_name: String,
        version_req: String,
    },

    /// Permission denied errors
    #[error("Plugin {plugin_name} requires permission {permission} which was denied")]
    PermissionDenied {
        plugin_name: String,
        permission: String,
    },

    /// Plugin not found errors
    #[error("Plugin {plugin_name} not found")]
    PluginNotFound {
        plugin_name: String,
    },

    /// Plugin already loaded errors
    #[error("Plugin {plugin_name} is already loaded")]
    PluginAlreadyLoaded {
        plugin_name: String,
    },

    /// Plugin configuration errors
    #[error("Invalid configuration for plugin {plugin_name}: {message}")]
    ConfigurationError {
        plugin_name: String,
        message: String,
    },

    /// Plugin execution errors
    #[error("Plugin {plugin_name} execution failed: {message}")]
    ExecutionError {
        plugin_name: String,
        message: String,
    },

    /// IO errors (file operations)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Dynamic library loading errors
    #[error("Library loading error: {0}")]
    LibraryLoading(#[from] libloading::Error),

    /// Plugin API version mismatch
    #[error("Plugin API version mismatch: expected {expected}, found {found}")]
    ApiVersionMismatch {
        expected: String,
        found: String,
    },

    /// Plugin initialization errors
    #[error("Plugin initialization failed: {message}")]
    InitializationError {
        message: String,
    },

    /// Timeout errors for plugin operations
    #[error("Plugin operation timed out after {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
    },

    /// Generic plugin errors
    #[error("Plugin error: {message}")]
    Generic {
        message: String,
    },
}

impl PluginError {
    /// Create a new loading error
    pub fn loading_error(
        path: PathBuf,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        PluginError::LoadError { path, source }
    }

    /// Create a new manifest error
    pub fn manifest_error(path: PathBuf, message: String) -> Self {
        PluginError::ManifestError { path, message }
    }

    /// Create a new version incompatibility error
    pub fn version_incompatible(
        plugin_name: String,
        plugin_version: String,
        nimbus_version: String,
    ) -> Self {
        PluginError::VersionIncompatible {
            plugin_name,
            plugin_version,
            nimbus_version,
        }
    }

    /// Create a new platform incompatibility error
    pub fn platform_incompatible(plugin_name: String) -> Self {
        PluginError::PlatformIncompatible { plugin_name }
    }

    /// Create a new missing dependency error
    pub fn missing_dependency(
        plugin_name: String,
        dependency_name: String,
        version_req: String,
    ) -> Self {
        PluginError::MissingDependency {
            plugin_name,
            dependency_name,
            version_req,
        }
    }

    /// Create a new permission denied error
    pub fn permission_denied(plugin_name: String, permission: String) -> Self {
        PluginError::PermissionDenied {
            plugin_name,
            permission,
        }
    }

    /// Create a new plugin not found error
    pub fn plugin_not_found(plugin_name: String) -> Self {
        PluginError::PluginNotFound { plugin_name }
    }

    /// Create a new plugin already loaded error
    pub fn plugin_already_loaded(plugin_name: String) -> Self {
        PluginError::PluginAlreadyLoaded { plugin_name }
    }

    /// Create a new configuration error
    pub fn configuration_error(plugin_name: String, message: String) -> Self {
        PluginError::ConfigurationError {
            plugin_name,
            message,
        }
    }

    /// Create a new execution error
    pub fn execution_error(plugin_name: String, message: String) -> Self {
        PluginError::ExecutionError {
            plugin_name,
            message,
        }
    }

    /// Create a new API version mismatch error
    pub fn api_version_mismatch(expected: String, found: String) -> Self {
        PluginError::ApiVersionMismatch { expected, found }
    }

    /// Create a new initialization error
    pub fn initialization_error(message: String) -> Self {
        PluginError::InitializationError { message }
    }

    /// Create a new timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        PluginError::Timeout { timeout_ms }
    }

    /// Create a new generic error
    pub fn generic(message: String) -> Self {
        PluginError::Generic { message }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // These errors might be fixed by user action
            PluginError::PermissionDenied { .. } => true,
            PluginError::ConfigurationError { .. } => true,
            PluginError::MissingDependency { .. } => true,
            PluginError::Timeout { .. } => true,
            
            // These errors are generally not recoverable
            PluginError::LoadError { .. } => false,
            PluginError::ManifestError { .. } => false,
            PluginError::VersionIncompatible { .. } => false,
            PluginError::PlatformIncompatible { .. } => false,
            PluginError::ApiVersionMismatch { .. } => false,
            
            // IO and library errors might be temporary
            PluginError::Io(_) => true,
            PluginError::LibraryLoading(_) => false,
            
            // JSON errors usually indicate programming errors
            PluginError::Json(_) => false,
            
            // Execution and initialization errors depend on context
            PluginError::ExecutionError { .. } => true,
            PluginError::InitializationError { .. } => true,
            
            // Plugin management errors
            PluginError::PluginNotFound { .. } => false,
            PluginError::PluginAlreadyLoaded { .. } => false,
            
            // Generic errors are case-by-case
            PluginError::Generic { .. } => true,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            PluginError::LoadError { .. } => {
                "Failed to load plugin. The plugin file may be corrupted or incompatible.".to_string()
            }
            PluginError::ManifestError { .. } => {
                "Plugin configuration is invalid. Please check the plugin manifest file.".to_string()
            }
            PluginError::VersionIncompatible { .. } => {
                "This plugin version is not compatible with your version of Nimbus.".to_string()
            }
            PluginError::PlatformIncompatible { .. } => {
                "This plugin is not compatible with your operating system.".to_string()
            }
            PluginError::MissingDependency { dependency_name, .. } => {
                format!("This plugin requires '{}' which is not installed.", dependency_name)
            }
            PluginError::PermissionDenied { permission, .. } => {
                format!("Plugin requires permission '{}' which was denied.", permission)
            }
            PluginError::PluginNotFound { plugin_name } => {
                format!("Plugin '{}' could not be found.", plugin_name)
            }
            PluginError::PluginAlreadyLoaded { plugin_name } => {
                format!("Plugin '{}' is already loaded.", plugin_name)
            }
            PluginError::ConfigurationError { .. } => {
                "Plugin configuration is invalid. Please check your settings.".to_string()
            }
            PluginError::ExecutionError { .. } => {
                "Plugin encountered an error during execution.".to_string()
            }
            PluginError::ApiVersionMismatch { .. } => {
                "Plugin API version is incompatible with this version of Nimbus.".to_string()
            }
            PluginError::InitializationError { .. } => {
                "Plugin failed to initialize properly.".to_string()
            }
            PluginError::Timeout { .. } => {
                "Plugin operation timed out.".to_string()
            }
            _ => self.to_string(),
        }
    }
}

/// Helper macro for creating plugin errors with context
#[macro_export]
macro_rules! plugin_error {
    ($kind:ident, $($arg:tt)*) => {
        PluginError::$kind { $($arg)* }
    };
}

/// Helper macro for propagating plugin errors with additional context
#[macro_export]
macro_rules! plugin_context {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| {
            log::error!("Plugin error: {} - Context: {}", e, $context);
            e
        })
    };
}