//! Viewer plugin interface for custom file viewers and editors

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{PluginInfo, Result};

/// Content that can be displayed by a viewer plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewerContent {
    /// Plain text content
    Text {
        content: String,
        encoding: Option<String>,
        language: Option<String>, // For syntax highlighting
    },
    /// HTML content
    Html {
        content: String,
        base_url: Option<String>,
        scripts: Vec<String>,     // JavaScript dependencies
        styles: Vec<String>,      // CSS dependencies
    },
    /// Image content
    Image {
        data: Vec<u8>,
        format: String,           // "png", "jpg", "gif", etc.
        width: Option<u32>,
        height: Option<u32>,
        metadata: HashMap<String, String>, // EXIF data, etc.
    },
    /// Binary/hex content
    Binary {
        data: Vec<u8>,
        offset: usize,
        total_size: u64,
    },
    /// Structured data (JSON, XML, etc.)
    Structured {
        data: serde_json::Value,
        format: String,           // "json", "xml", "yaml", etc.
        schema: Option<serde_json::Value>, // JSON Schema for validation
    },
    /// Media content (audio, video)
    Media {
        file_path: String,        // Path to media file
        media_type: MediaType,
        duration: Option<f64>,    // Duration in seconds
        metadata: HashMap<String, String>,
    },
    /// Custom content with plugin-specific data
    Custom {
        content_type: String,
        data: serde_json::Value,
        renderer: String,         // Renderer component name
    },
    /// Error content when file cannot be viewed
    Error {
        message: String,
        error_code: Option<String>,
        suggestions: Vec<String>, // Suggested actions for user
    },
}

impl ViewerContent {
    /// Create text content
    pub fn text(content: String) -> Self {
        ViewerContent::Text {
            content,
            encoding: None,
            language: None,
        }
    }
    
    /// Create text content with syntax highlighting
    pub fn text_with_language(content: String, language: String) -> Self {
        ViewerContent::Text {
            content,
            encoding: None,
            language: Some(language),
        }
    }
    
    /// Create HTML content
    pub fn html(content: String) -> Self {
        ViewerContent::Html {
            content,
            base_url: None,
            scripts: Vec::new(),
            styles: Vec::new(),
        }
    }
    
    /// Create image content
    pub fn image(data: Vec<u8>, format: String) -> Self {
        ViewerContent::Image {
            data,
            format,
            width: None,
            height: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create structured data content
    pub fn structured(data: serde_json::Value, format: String) -> Self {
        ViewerContent::Structured {
            data,
            format,
            schema: None,
        }
    }
    
    /// Create error content
    pub fn error(message: String) -> Self {
        ViewerContent::Error {
            message,
            error_code: None,
            suggestions: Vec::new(),
        }
    }
    
    /// Get the content type for this viewer content
    pub fn content_type(&self) -> &'static str {
        match self {
            ViewerContent::Text { .. } => "text",
            ViewerContent::Html { .. } => "html",
            ViewerContent::Image { .. } => "image",
            ViewerContent::Binary { .. } => "binary",
            ViewerContent::Structured { .. } => "structured",
            ViewerContent::Media { .. } => "media",
            ViewerContent::Custom { .. } => "custom",
            ViewerContent::Error { .. } => "error",
        }
    }
    
    /// Check if this content is editable
    pub fn is_editable(&self) -> bool {
        matches!(self, 
            ViewerContent::Text { .. } | 
            ViewerContent::Html { .. } | 
            ViewerContent::Structured { .. }
        )
    }
    
    /// Get estimated content size in bytes
    pub fn estimated_size(&self) -> usize {
        match self {
            ViewerContent::Text { content, .. } => content.len(),
            ViewerContent::Html { content, .. } => content.len(),
            ViewerContent::Image { data, .. } => data.len(),
            ViewerContent::Binary { data, .. } => data.len(),
            ViewerContent::Structured { data, .. } => {
                serde_json::to_string(data).map(|s| s.len()).unwrap_or(0)
            }
            ViewerContent::Media { .. } => 0, // File size would be separate
            ViewerContent::Custom { data, .. } => {
                serde_json::to_string(data).map(|s| s.len()).unwrap_or(0)
            }
            ViewerContent::Error { message, .. } => message.len(),
        }
    }
}

/// Media types for media content
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
    Animation,
}

/// Viewer capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerCapabilities {
    /// Can display file content
    pub can_view: bool,
    /// Can edit file content
    pub can_edit: bool,
    /// Can save changes back to file
    pub can_save: bool,
    /// Supports search within content
    pub can_search: bool,
    /// Supports printing content
    pub can_print: bool,
    /// Supports copying content to clipboard
    pub can_copy: bool,
    /// Supports zooming/scaling
    pub can_zoom: bool,
    /// Supports full-screen viewing
    pub can_fullscreen: bool,
    /// Maximum file size that can be handled (in bytes)
    pub max_file_size: Option<u64>,
    /// Preferred viewport size (width, height)
    pub preferred_size: Option<(u32, u32)>,
}

impl Default for ViewerCapabilities {
    fn default() -> Self {
        Self {
            can_view: true,
            can_edit: false,
            can_save: false,
            can_search: false,
            can_print: false,
            can_copy: true,
            can_zoom: false,
            can_fullscreen: false,
            max_file_size: Some(100 * 1024 * 1024), // 100MB default limit
            preferred_size: None,
        }
    }
}

/// Viewer action that can be performed by the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerAction {
    /// Action identifier
    pub id: String,
    /// Human-readable action name
    pub name: String,
    /// Optional description/tooltip
    pub description: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Icon name or emoji
    pub icon: Option<String>,
    /// Whether this action is enabled
    pub enabled: bool,
}

/// Options for viewing files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerOptions {
    /// Read-only mode
    pub readonly: bool,
    /// Line numbers for text content
    pub show_line_numbers: bool,
    /// Syntax highlighting
    pub syntax_highlighting: bool,
    /// Word wrapping for text
    pub word_wrap: bool,
    /// Theme for syntax highlighting
    pub theme: Option<String>,
    /// Font family for text display
    pub font_family: Option<String>,
    /// Font size
    pub font_size: Option<f32>,
    /// Maximum content length to load
    pub max_content_length: Option<usize>,
    /// Custom options for specific viewers
    pub custom_options: HashMap<String, serde_json::Value>,
}

impl Default for ViewerOptions {
    fn default() -> Self {
        Self {
            readonly: false,
            show_line_numbers: true,
            syntax_highlighting: true,
            word_wrap: false,
            theme: Some("default".to_string()),
            font_family: Some("monospace".to_string()),
            font_size: Some(14.0),
            max_content_length: Some(10 * 1024 * 1024), // 10MB limit
            custom_options: HashMap::new(),
        }
    }
}

/// Viewer plugin trait
#[async_trait]
pub trait ViewerPlugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;
    
    /// Get list of file extensions this viewer supports
    fn supported_extensions(&self) -> Vec<String>;
    
    /// Get list of MIME types this viewer supports
    fn supported_mime_types(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Get viewer capabilities
    fn capabilities(&self) -> ViewerCapabilities;
    
    /// Get list of actions available for this viewer
    fn get_actions(&self) -> Vec<ViewerAction> {
        Vec::new()
    }
    
    /// Check if this plugin can handle the given file
    async fn can_handle_file(&self, file_path: &Path) -> Result<bool> {
        // Check by extension
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if self.supported_extensions()
                    .iter()
                    .any(|supported| supported.eq_ignore_ascii_case(ext_str))
                {
                    return Ok(true);
                }
            }
        }
        
        // Check by MIME type if available
        if !self.supported_mime_types().is_empty() {
            // This would require MIME type detection, which we'll skip for now
            // In a real implementation, you'd use a library like `tree_magic` or similar
        }
        
        Ok(false)
    }
    
    /// Load and render file content for viewing
    async fn view_file(
        &self,
        file_path: &Path,
        options: &ViewerOptions,
    ) -> Result<ViewerContent>;
    
    /// Save edited content back to file (if supported)
    async fn save_file(
        &self,
        file_path: &Path,
        content: &ViewerContent,
        options: &ViewerOptions,
    ) -> Result<()> {
        // Default implementation returns error for unsupported operation
        Err(crate::PluginError::execution_error(
            self.info().name,
            "Save operation not supported by this viewer".to_string(),
        ))
    }
    
    /// Perform a viewer action
    async fn perform_action(
        &self,
        action_id: &str,
        file_path: &Path,
        content: &ViewerContent,
    ) -> Result<ViewerContent> {
        // Default implementation returns error for unsupported action
        Err(crate::PluginError::execution_error(
            self.info().name,
            format!("Action '{}' not supported by this viewer", action_id),
        ))
    }
    
    /// Search within content (if supported)
    async fn search_content(
        &self,
        content: &ViewerContent,
        query: &str,
        case_sensitive: bool,
    ) -> Result<Vec<SearchMatch>> {
        // Default implementation returns empty results
        Ok(Vec::new())
    }
    
    /// Get preview/thumbnail for file (optional)
    async fn get_preview(
        &self,
        file_path: &Path,
        size: (u32, u32),
    ) -> Result<Option<Vec<u8>>> {
        // Default implementation returns no preview
        Ok(None)
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

/// Search match result within content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    /// Line number (0-based)
    pub line: usize,
    /// Column number (0-based)
    pub column: usize,
    /// Length of the match
    pub length: usize,
    /// Context around the match
    pub context: String,
    /// Highlighted text
    pub highlight: String,
}

/// Registry for viewer plugins
pub struct ViewerPluginRegistry {
    plugins: HashMap<String, Box<dyn ViewerPlugin>>,
    extension_map: HashMap<String, Vec<String>>, // extension -> plugin names
    mime_type_map: HashMap<String, Vec<String>>, // mime type -> plugin names
}

impl ViewerPluginRegistry {
    /// Create a new viewer plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            extension_map: HashMap::new(),
            mime_type_map: HashMap::new(),
        }
    }
    
    /// Register a viewer plugin
    pub async fn register_plugin(&mut self, mut plugin: Box<dyn ViewerPlugin>) -> Result<()> {
        let info = plugin.info();
        let plugin_name = info.name.clone();
        
        // Initialize the plugin
        plugin.initialize().await?;
        
        // Build extension mapping
        for ext in plugin.supported_extensions() {
            self.extension_map
                .entry(ext.to_lowercase())
                .or_insert_with(Vec::new)
                .push(plugin_name.clone());
        }
        
        // Build MIME type mapping
        for mime_type in plugin.supported_mime_types() {
            self.mime_type_map
                .entry(mime_type)
                .or_insert_with(Vec::new)
                .push(plugin_name.clone());
        }
        
        // Store the plugin
        self.plugins.insert(plugin_name.clone(), plugin);
        
        log::info!("Registered viewer plugin: {}", plugin_name);
        Ok(())
    }
    
    /// Unregister a viewer plugin
    pub async fn unregister_plugin(&mut self, plugin_name: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(plugin_name) {
            // Cleanup the plugin
            plugin.cleanup().await?;
            
            // Remove from extension mapping
            self.extension_map.retain(|_, names| {
                names.retain(|name| name != plugin_name);
                !names.is_empty()
            });
            
            // Remove from MIME type mapping
            self.mime_type_map.retain(|_, names| {
                names.retain(|name| name != plugin_name);
                !names.is_empty()
            });
            
            log::info!("Unregistered viewer plugin: {}", plugin_name);
        }
        
        Ok(())
    }
    
    /// Get viewer plugins that can handle a file
    pub async fn get_viewers_for_file(&self, file_path: &Path) -> Vec<&dyn ViewerPlugin> {
        let mut viewers = Vec::new();
        
        // Check by file extension
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if let Some(plugin_names) = self.extension_map.get(&ext_str.to_lowercase()) {
                    for plugin_name in plugin_names {
                        if let Some(plugin) = self.plugins.get(plugin_name) {
                            if plugin.can_handle_file(file_path).await.unwrap_or(false) {
                                viewers.push(plugin.as_ref());
                            }
                        }
                    }
                }
            }
        }
        
        viewers
    }
    
    /// Get the best viewer for a file (first match)
    pub async fn get_default_viewer(&self, file_path: &Path) -> Option<&dyn ViewerPlugin> {
        let viewers = self.get_viewers_for_file(file_path).await;
        viewers.into_iter().next()
    }
    
    /// Get a specific plugin by name
    pub fn get_plugin(&self, plugin_name: &str) -> Option<&dyn ViewerPlugin> {
        self.plugins.get(plugin_name).map(|p| p.as_ref())
    }
    
    /// Get list of all registered viewer plugins
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
    
    /// Get plugin information for all registered plugins
    pub fn get_plugin_info(&self) -> Vec<(String, PluginInfo, ViewerCapabilities)> {
        self.plugins
            .iter()
            .map(|(name, plugin)| (name.clone(), plugin.info(), plugin.capabilities()))
            .collect()
    }
}

impl Default for ViewerPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}