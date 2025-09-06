//! # Nimbus Plugin SDK
//!
//! A comprehensive plugin system for the Nimbus file manager, enabling extensibility
//! through three main plugin types: Content, Protocol, and Viewer plugins.
//!
//! ## Plugin Types
//!
//! - **Content Plugins**: Extend file metadata and column display
//! - **Protocol Plugins**: Add support for custom remote file systems
//! - **Viewer Plugins**: Implement custom file viewers and editors
//!
//! ## Example Usage
//!
//! ```rust
//! use nimbus_plugin_sdk::{ContentPlugin, PluginInfo, Result};
//! use std::collections::HashMap;
//!
//! pub struct MyContentPlugin;
//!
//! impl ContentPlugin for MyContentPlugin {
//!     fn info(&self) -> PluginInfo {
//!         PluginInfo {
//!             name: "My Content Plugin".to_string(),
//!             version: semver::Version::new(1, 0, 0),
//!             description: "Adds custom metadata".to_string(),
//!             author: "Plugin Developer".to_string(),
//!         }
//!     }
//!
//!     fn supported_extensions(&self) -> Vec<String> {
//!         vec!["txt".to_string(), "md".to_string()]
//!     }
//!
//!     async fn get_metadata(&self, file_path: &std::path::Path) -> Result<HashMap<String, String>> {
//!         // Implementation here
//!         Ok(HashMap::new())
//!     }
//! }
//! ```

pub mod content;
pub mod protocol;
pub mod viewer;
pub mod manager;
pub mod error;
pub mod types;

// Re-export main types
pub use content::ContentPlugin;
pub use protocol::{ProtocolPlugin, RemoteClient, ConnectionConfig};
pub use viewer::{ViewerPlugin, ViewerContent};
pub use manager::{PluginManager, PluginRegistry};
pub use error::{PluginError, Result};
pub use types::{PluginInfo, PluginType, PluginStatus, PluginManifest};