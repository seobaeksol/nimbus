//! Content plugin interface for extending file metadata and column display

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{PluginInfo, Result};

/// File information that can be extended by content plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedFileInfo {
    /// Basic file metadata
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub is_directory: bool,
    pub extension: Option<String>,
    
    /// Extended metadata provided by plugins
    pub metadata: HashMap<String, String>,
    
    /// Custom columns provided by plugins
    pub columns: HashMap<String, ColumnValue>,
}

/// Values that can be displayed in custom columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnValue {
    /// Text value
    Text(String),
    /// Numeric value
    Number(f64),
    /// Boolean value (displayed as Yes/No or icons)
    Boolean(bool),
    /// Date/time value (ISO 8601 format)
    DateTime(String),
    /// File size value (automatically formatted)
    FileSize(u64),
    /// Progress value (0.0 to 1.0)
    Progress(f32),
    /// Custom formatted value with tooltip
    Custom {
        display: String,
        tooltip: Option<String>,
        sort_value: Option<String>,
    },
}

impl ColumnValue {
    /// Get the display string for this value
    pub fn display_text(&self) -> String {
        match self {
            ColumnValue::Text(s) => s.clone(),
            ColumnValue::Number(n) => n.to_string(),
            ColumnValue::Boolean(b) => if *b { "Yes".to_string() } else { "No".to_string() },
            ColumnValue::DateTime(dt) => dt.clone(),
            ColumnValue::FileSize(size) => format_file_size(*size),
            ColumnValue::Progress(p) => format!("{:.1}%", p * 100.0),
            ColumnValue::Custom { display, .. } => display.clone(),
        }
    }

    /// Get the value for sorting purposes
    pub fn sort_value(&self) -> String {
        match self {
            ColumnValue::Text(s) => s.clone(),
            ColumnValue::Number(n) => format!("{:020.6}", n), // Zero-padded for lexical sorting
            ColumnValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
            ColumnValue::DateTime(dt) => dt.clone(),
            ColumnValue::FileSize(size) => format!("{:020}", size),
            ColumnValue::Progress(p) => format!("{:010.6}", p),
            ColumnValue::Custom { sort_value: Some(sv), .. } => sv.clone(),
            ColumnValue::Custom { display, .. } => display.clone(),
        }
    }

    /// Get tooltip text for this value
    pub fn tooltip(&self) -> Option<String> {
        match self {
            ColumnValue::Custom { tooltip, .. } => tooltip.clone(),
            ColumnValue::FileSize(size) => Some(format!("{} bytes", size)),
            ColumnValue::Progress(p) => Some(format!("Progress: {:.2}%", p * 100.0)),
            _ => None,
        }
    }
}

/// Column definition for custom columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Unique column identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Optional description/tooltip
    pub description: Option<String>,
    /// Column width in pixels (0 for auto)
    pub width: u32,
    /// Whether this column is sortable
    pub sortable: bool,
    /// Whether this column is visible by default
    pub visible_by_default: bool,
    /// Column alignment
    pub alignment: ColumnAlignment,
}

/// Text alignment for columns
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
}

/// Content plugin trait that must be implemented by all content plugins
#[async_trait]
pub trait ContentPlugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;
    
    /// Get list of file extensions this plugin supports
    /// Return empty vector to support all files
    fn supported_extensions(&self) -> Vec<String>;
    
    /// Get custom column definitions provided by this plugin
    fn column_definitions(&self) -> Vec<ColumnDefinition> {
        Vec::new()
    }
    
    /// Get extended metadata for a file
    /// This is called for files matching supported extensions
    async fn get_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>>;
    
    /// Get custom column values for a file
    /// This is called for files matching supported extensions
    async fn get_columns(&self, file_path: &Path) -> Result<HashMap<String, ColumnValue>> {
        // Default implementation returns empty columns
        Ok(HashMap::new())
    }
    
    /// Get file thumbnail/icon (optional)
    /// Return None to use default icons
    async fn get_thumbnail(&self, file_path: &Path, size: u32) -> Result<Option<Vec<u8>>> {
        // Default implementation returns None
        Ok(None)
    }
    
    /// Check if this plugin can handle the given file
    /// Override this for more complex file type detection
    async fn can_handle_file(&self, file_path: &Path) -> bool {
        if self.supported_extensions().is_empty() {
            return true; // Support all files if no specific extensions
        }
        
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.supported_extensions()
                    .iter()
                    .any(|supported| supported.eq_ignore_ascii_case(ext_str));
            }
        }
        
        false
    }
    
    /// Initialize the plugin (called once on load)
    async fn initialize(&mut self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Cleanup the plugin (called on unload)
    async fn cleanup(&mut self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Helper function to format file sizes
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Registry for content plugins
pub struct ContentPluginRegistry {
    plugins: HashMap<String, Box<dyn ContentPlugin>>,
    column_definitions: HashMap<String, ColumnDefinition>,
}

impl ContentPluginRegistry {
    /// Create a new content plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            column_definitions: HashMap::new(),
        }
    }
    
    /// Register a content plugin
    pub async fn register_plugin(&mut self, mut plugin: Box<dyn ContentPlugin>) -> Result<()> {
        let info = plugin.info();
        
        // Initialize the plugin
        plugin.initialize().await?;
        
        // Register column definitions
        for column_def in plugin.column_definitions() {
            self.column_definitions.insert(column_def.id.clone(), column_def);
        }
        
        // Store the plugin
        self.plugins.insert(info.name.clone(), plugin);
        
        log::info!("Registered content plugin: {}", info.name);
        Ok(())
    }
    
    /// Unregister a content plugin
    pub async fn unregister_plugin(&mut self, plugin_name: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(plugin_name) {
            // Cleanup the plugin
            plugin.cleanup().await?;
            
            // Remove column definitions
            let plugin_columns: Vec<String> = self
                .column_definitions
                .iter()
                .filter(|(_, def)| def.id.starts_with(&format!("{}.", plugin_name)))
                .map(|(id, _)| id.clone())
                .collect();
            
            for column_id in plugin_columns {
                self.column_definitions.remove(&column_id);
            }
            
            log::info!("Unregistered content plugin: {}", plugin_name);
        }
        
        Ok(())
    }
    
    /// Get extended file information from all applicable plugins
    pub async fn get_extended_file_info(&self, file_path: &Path) -> Result<ExtendedFileInfo> {
        // Get basic file info (this would come from the file system)
        let metadata = std::fs::metadata(file_path)?;
        let name = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let extension = file_path.extension()
            .map(|ext| ext.to_string_lossy().to_string());
        
        let mut extended_info = ExtendedFileInfo {
            name,
            path: file_path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified: format!("{:?}", metadata.modified().unwrap_or(std::time::UNIX_EPOCH)),
            is_directory: metadata.is_dir(),
            extension,
            metadata: HashMap::new(),
            columns: HashMap::new(),
        };
        
        // Collect metadata and columns from all applicable plugins
        for (plugin_name, plugin) in &self.plugins {
            if plugin.can_handle_file(file_path).await {
                // Get metadata
                if let Ok(plugin_metadata) = plugin.get_metadata(file_path).await {
                    for (key, value) in plugin_metadata {
                        extended_info.metadata.insert(
                            format!("{}.{}", plugin_name, key),
                            value
                        );
                    }
                }
                
                // Get column values
                if let Ok(plugin_columns) = plugin.get_columns(file_path).await {
                    for (key, value) in plugin_columns {
                        extended_info.columns.insert(
                            format!("{}.{}", plugin_name, key),
                            value
                        );
                    }
                }
            }
        }
        
        Ok(extended_info)
    }
    
    /// Get all column definitions
    pub fn get_column_definitions(&self) -> &HashMap<String, ColumnDefinition> {
        &self.column_definitions
    }
    
    /// Get list of registered plugins
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

impl Default for ContentPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}