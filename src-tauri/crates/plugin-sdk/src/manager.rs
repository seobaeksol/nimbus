//! Plugin manager for loading, managing, and coordinating plugins

use async_trait::async_trait;
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    ContentPlugin, ProtocolPlugin, ViewerPlugin, PluginError, PluginManifest, PluginStatus, PluginType, Result,
    content::ContentPluginRegistry,
    protocol::ProtocolPluginRegistry,
    viewer::ViewerPluginRegistry,
    types::PluginInstance,
};

/// Plugin loading context and metadata
#[derive(Debug)]
pub struct LoadedPlugin {
    /// Plugin instance information
    pub instance: PluginInstance,
    /// Loaded dynamic library
    pub library: Arc<Library>,
    /// Plugin type-specific data
    pub plugin_data: PluginData,
}

/// Type-specific plugin data
pub enum PluginData {
    Content(Box<dyn ContentPlugin>),
    Protocol(Box<dyn ProtocolPlugin>),
    Viewer(Box<dyn ViewerPlugin>),
}

impl std::fmt::Debug for PluginData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginData::Content(_) => write!(f, "PluginData::Content(<ContentPlugin>)"),
            PluginData::Protocol(_) => write!(f, "PluginData::Protocol(<ProtocolPlugin>)"),
            PluginData::Viewer(_) => write!(f, "PluginData::Viewer(<ViewerPlugin>)"),
        }
    }
}

/// Plugin discovery settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDiscoverySettings {
    /// Directories to scan for plugins
    pub plugin_directories: Vec<PathBuf>,
    /// Whether to recursively scan subdirectories
    pub recursive_scan: bool,
    /// File extensions to consider as plugins
    pub plugin_extensions: Vec<String>,
    /// Whether to auto-load plugins on discovery
    pub auto_load: bool,
    /// Maximum plugin load time in seconds
    pub load_timeout: u64,
    /// Whether to verify plugin signatures
    pub verify_signatures: bool,
}

impl Default for PluginDiscoverySettings {
    fn default() -> Self {
        Self {
            plugin_directories: vec![
                PathBuf::from("plugins"),
                PathBuf::from("/usr/local/lib/nimbus/plugins"),
                PathBuf::from("~/.local/lib/nimbus/plugins"),
            ],
            recursive_scan: true,
            plugin_extensions: vec!["dll".to_string(), "so".to_string(), "dylib".to_string()],
            auto_load: false,
            load_timeout: 30,
            verify_signatures: true,
        }
    }
}

/// Plugin manager for coordinating all plugin types
pub struct PluginManager {
    /// Loaded plugins
    loaded_plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    /// Content plugin registry
    content_registry: Arc<RwLock<ContentPluginRegistry>>,
    /// Protocol plugin registry
    protocol_registry: Arc<RwLock<ProtocolPluginRegistry>>,
    /// Viewer plugin registry
    viewer_registry: Arc<RwLock<ViewerPluginRegistry>>,
    /// Discovery settings
    settings: PluginDiscoverySettings,
    /// Nimbus version for compatibility checking
    nimbus_version: semver::Version,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(nimbus_version: semver::Version) -> Self {
        Self {
            loaded_plugins: Arc::new(RwLock::new(HashMap::new())),
            content_registry: Arc::new(RwLock::new(ContentPluginRegistry::new())),
            protocol_registry: Arc::new(RwLock::new(ProtocolPluginRegistry::new())),
            viewer_registry: Arc::new(RwLock::new(ViewerPluginRegistry::new())),
            settings: PluginDiscoverySettings::default(),
            nimbus_version,
        }
    }

    /// Update discovery settings
    pub fn update_settings(&mut self, settings: PluginDiscoverySettings) {
        self.settings = settings;
    }

    /// Discover plugins in configured directories
    pub async fn discover_plugins(&self) -> Result<Vec<PluginManifest>> {
        let mut manifests = Vec::new();

        for plugin_dir in &self.settings.plugin_directories {
            if plugin_dir.exists() {
                let discovered = self.scan_directory(plugin_dir).await?;
                manifests.extend(discovered);
            }
        }

        log::info!("Discovered {} plugins", manifests.len());
        Ok(manifests)
    }

    /// Scan a directory for plugins
    fn scan_directory<'a>(&'a self, dir: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PluginManifest>>> + Send + 'a>> {
        Box::pin(async move {
            let mut manifests = Vec::new();

            let mut entries = tokio::fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() && self.settings.recursive_scan {
                    let sub_manifests = self.scan_directory(&path).await?;
                    manifests.extend(sub_manifests);
                } else if self.is_plugin_file(&path) {
                    if let Ok(manifest) = self.load_manifest(&path).await {
                        manifests.push(manifest);
                    }
                }
            }

            Ok(manifests)
        })
    }

    /// Check if a file is a potential plugin file
    fn is_plugin_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.settings.plugin_extensions
                    .iter()
                    .any(|supported| supported.eq_ignore_ascii_case(ext_str));
            }
        }
        false
    }

    /// Load plugin manifest from file
    async fn load_manifest(&self, plugin_path: &Path) -> Result<PluginManifest> {
        // Look for manifest file next to plugin library
        let manifest_path = plugin_path.with_extension("json");
        
        if manifest_path.exists() {
            let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
            let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;
            Ok(manifest)
        } else {
            // Try to load manifest from plugin library itself (embedded)
            self.load_embedded_manifest(plugin_path).await
        }
    }

    /// Load embedded manifest from plugin library
    async fn load_embedded_manifest(&self, plugin_path: &Path) -> Result<PluginManifest> {
        // This would require loading the library and calling a manifest function
        // For now, we'll return an error
        Err(PluginError::manifest_error(
            plugin_path.to_path_buf(),
            "No manifest file found".to_string(),
        ))
    }

    /// Load a plugin from file
    pub async fn load_plugin(&self, plugin_path: &Path) -> Result<String> {
        let manifest = self.load_manifest(plugin_path).await?;
        
        // Validate compatibility
        self.validate_plugin_compatibility(&manifest)?;
        
        // Load the plugin library
        let library = unsafe {
            Library::new(plugin_path)
                .map_err(|e| PluginError::loading_error(
                    plugin_path.to_path_buf(),
                    Box::new(e),
                ))?
        };

        let library = Arc::new(library);
        
        // Create plugin instance
        let instance = PluginInstance::new(manifest.clone(), plugin_path.to_path_buf());
        let plugin_id = instance.id.to_string();

        // Load the plugin based on its type
        let plugin_data = match manifest.plugin_type {
            PluginType::Content => {
                let plugin = self.load_content_plugin(&library, &manifest).await?;
                PluginData::Content(plugin)
            }
            PluginType::Protocol => {
                let plugin = self.load_protocol_plugin(&library, &manifest).await?;
                PluginData::Protocol(plugin)
            }
            PluginType::Viewer => {
                let plugin = self.load_viewer_plugin(&library, &manifest).await?;
                PluginData::Viewer(plugin)
            }
        };

        // Create loaded plugin
        let loaded_plugin = LoadedPlugin {
            instance,
            library,
            plugin_data,
        };

        // Register with appropriate registry
        self.register_plugin(&loaded_plugin).await?;

        // Store loaded plugin
        let mut loaded_plugins = self.loaded_plugins.write().await;
        loaded_plugins.insert(plugin_id.clone(), loaded_plugin);

        log::info!("Successfully loaded plugin: {}", manifest.info.name);
        Ok(plugin_id)
    }

    /// Validate plugin compatibility
    fn validate_plugin_compatibility(&self, manifest: &PluginManifest) -> Result<()> {
        // Check version compatibility
        if !manifest.info.is_compatible_with(&self.nimbus_version) {
            return Err(PluginError::version_incompatible(
                manifest.info.name.clone(),
                manifest.info.version.to_string(),
                self.nimbus_version.to_string(),
            ));
        }

        // Check platform compatibility
        if !manifest.platforms.iter().any(|p| p.is_compatible()) {
            return Err(PluginError::platform_incompatible(
                manifest.info.name.clone(),
            ));
        }

        Ok(())
    }

    /// Load content plugin from library
    async fn load_content_plugin(
        &self,
        _library: &Arc<Library>,
        _manifest: &PluginManifest,
    ) -> Result<Box<dyn ContentPlugin>> {
        // In a real implementation, this would use the plugin's entry point
        // to create the plugin instance. For now, we'll return an error.
        Err(PluginError::initialization_error(
            "Dynamic content plugin loading not yet implemented".to_string(),
        ))
    }

    /// Load protocol plugin from library
    async fn load_protocol_plugin(
        &self,
        _library: &Arc<Library>,
        _manifest: &PluginManifest,
    ) -> Result<Box<dyn ProtocolPlugin>> {
        Err(PluginError::initialization_error(
            "Dynamic protocol plugin loading not yet implemented".to_string(),
        ))
    }

    /// Load viewer plugin from library
    async fn load_viewer_plugin(
        &self,
        _library: &Arc<Library>,
        _manifest: &PluginManifest,
    ) -> Result<Box<dyn ViewerPlugin>> {
        Err(PluginError::initialization_error(
            "Dynamic viewer plugin loading not yet implemented".to_string(),
        ))
    }

    /// Register plugin with appropriate registry
    async fn register_plugin(&self, loaded_plugin: &LoadedPlugin) -> Result<()> {
        match &loaded_plugin.plugin_data {
            PluginData::Content(_plugin) => {
                // We can't move the plugin out of the Box, so for now we'll skip registration
                log::info!("Content plugin registration skipped for dynamic loading");
            }
            PluginData::Protocol(_plugin) => {
                log::info!("Protocol plugin registration skipped for dynamic loading");
            }
            PluginData::Viewer(_plugin) => {
                log::info!("Viewer plugin registration skipped for dynamic loading");
            }
        }
        Ok(())
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut loaded_plugins = self.loaded_plugins.write().await;
        
        if let Some(loaded_plugin) = loaded_plugins.remove(plugin_id) {
            // Unregister from appropriate registry
            self.unregister_plugin(&loaded_plugin).await?;
            
            log::info!("Unloaded plugin: {}", loaded_plugin.instance.manifest.info.name);
        }

        Ok(())
    }

    /// Unregister plugin from registries
    async fn unregister_plugin(&self, loaded_plugin: &LoadedPlugin) -> Result<()> {
        let plugin_name = &loaded_plugin.instance.manifest.info.name;
        
        match &loaded_plugin.plugin_data {
            PluginData::Content(_) => {
                let mut registry = self.content_registry.write().await;
                registry.unregister_plugin(plugin_name).await?;
            }
            PluginData::Protocol(_plugin) => {
                let _registry = self.protocol_registry.write().await;
                // We need the scheme, but we can't call methods on the plugin here
                // This would need to be stored in the LoadedPlugin structure
                log::info!("Protocol plugin unregistration needs scheme information");
            }
            PluginData::Viewer(_) => {
                let mut registry = self.viewer_registry.write().await;
                registry.unregister_plugin(plugin_name).await?;
            }
        }

        Ok(())
    }

    /// Get list of loaded plugins
    pub async fn get_loaded_plugins(&self) -> Vec<PluginInstance> {
        let loaded_plugins = self.loaded_plugins.read().await;
        loaded_plugins.values().map(|p| p.instance.clone()).collect()
    }

    /// Get plugin information by ID
    pub async fn get_plugin_info(&self, plugin_id: &str) -> Option<PluginInstance> {
        let loaded_plugins = self.loaded_plugins.read().await;
        loaded_plugins.get(plugin_id).map(|p| p.instance.clone())
    }

    /// Enable/disable a plugin
    pub async fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> Result<()> {
        let mut loaded_plugins = self.loaded_plugins.write().await;
        
        if let Some(loaded_plugin) = loaded_plugins.get_mut(plugin_id) {
            let new_status = if enabled {
                PluginStatus::Active
            } else {
                PluginStatus::Inactive
            };
            
            loaded_plugin.instance.status = new_status;
            
            log::info!(
                "Plugin {} {}", 
                loaded_plugin.instance.manifest.info.name,
                if enabled { "enabled" } else { "disabled" }
            );
        }

        Ok(())
    }

    /// Get plugin statistics
    pub async fn get_plugin_stats(&self) -> PluginStats {
        let loaded_plugins = self.loaded_plugins.read().await;
        let mut stats = PluginStats::default();

        for plugin in loaded_plugins.values() {
            stats.total_plugins += 1;
            
            match plugin.instance.status {
                PluginStatus::Active => stats.active_plugins += 1,
                PluginStatus::Inactive => stats.inactive_plugins += 1,
                PluginStatus::Error => stats.error_plugins += 1,
                PluginStatus::Loading => stats.loading_plugins += 1,
                PluginStatus::Unloaded => {}
            }

            match plugin.instance.manifest.plugin_type {
                PluginType::Content => stats.content_plugins += 1,
                PluginType::Protocol => stats.protocol_plugins += 1,
                PluginType::Viewer => stats.viewer_plugins += 1,
            }
        }

        stats
    }

    /// Get content plugin registry
    pub fn content_registry(&self) -> Arc<RwLock<ContentPluginRegistry>> {
        Arc::clone(&self.content_registry)
    }

    /// Get protocol plugin registry
    pub fn protocol_registry(&self) -> Arc<RwLock<ProtocolPluginRegistry>> {
        Arc::clone(&self.protocol_registry)
    }

    /// Get viewer plugin registry
    pub fn viewer_registry(&self) -> Arc<RwLock<ViewerPluginRegistry>> {
        Arc::clone(&self.viewer_registry)
    }
}

/// Plugin statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub inactive_plugins: usize,
    pub error_plugins: usize,
    pub loading_plugins: usize,
    pub content_plugins: usize,
    pub protocol_plugins: usize,
    pub viewer_plugins: usize,
}

/// Plugin registry trait for unified plugin management
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Get registry type name
    fn registry_type(&self) -> &'static str;
    
    /// Get number of registered plugins
    async fn plugin_count(&self) -> usize;
    
    /// Get list of plugin names
    async fn get_plugin_names(&self) -> Vec<String>;
    
    /// Check if a plugin is registered
    async fn has_plugin(&self, plugin_name: &str) -> bool;
}

/// Implement PluginRegistry for ContentPluginRegistry
#[async_trait]
impl PluginRegistry for ContentPluginRegistry {
    fn registry_type(&self) -> &'static str {
        "Content"
    }
    
    async fn plugin_count(&self) -> usize {
        self.get_plugin_names().len()
    }
    
    async fn get_plugin_names(&self) -> Vec<String> {
        self.get_plugin_names()
    }
    
    async fn has_plugin(&self, plugin_name: &str) -> bool {
        self.get_plugin_names().contains(&plugin_name.to_string())
    }
}

/// Implement PluginRegistry for ProtocolPluginRegistry
#[async_trait]
impl PluginRegistry for ProtocolPluginRegistry {
    fn registry_type(&self) -> &'static str {
        "Protocol"
    }
    
    async fn plugin_count(&self) -> usize {
        self.get_supported_schemes().len()
    }
    
    async fn get_plugin_names(&self) -> Vec<String> {
        self.get_plugin_info()
            .into_iter()
            .map(|(_, info, _)| info.name)
            .collect()
    }
    
    async fn has_plugin(&self, plugin_name: &str) -> bool {
        self.get_plugin_info()
            .into_iter()
            .any(|(_, info, _)| info.name == plugin_name)
    }
}

/// Implement PluginRegistry for ViewerPluginRegistry
#[async_trait]
impl PluginRegistry for ViewerPluginRegistry {
    fn registry_type(&self) -> &'static str {
        "Viewer"
    }
    
    async fn plugin_count(&self) -> usize {
        self.get_plugin_names().len()
    }
    
    async fn get_plugin_names(&self) -> Vec<String> {
        self.get_plugin_names()
    }
    
    async fn has_plugin(&self, plugin_name: &str) -> bool {
        self.get_plugin_names().contains(&plugin_name.to_string())
    }
}