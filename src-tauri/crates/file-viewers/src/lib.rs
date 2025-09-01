//! File Viewers Library for Nimbus
//!
//! Provides built-in viewers for common file types including text, images,
//! and binary files, eliminating the need for external applications.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// File viewer error types
#[derive(Error, Debug)]
pub enum ViewerError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Unsupported file type: {extension}")]
    UnsupportedFileType { extension: String },

    #[error("File too large: {size} bytes (max: {max_size})")]
    FileTooLarge { size: u64, max_size: u64 },

    #[error("Encoding error: {message}")]
    EncodingError { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {message}")]
    ImageError { message: String },

    #[error("Other error: {0}")]
    Other(String),
}

/// File viewer capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerCapabilities {
    pub name: String,
    pub description: String,
    pub supported_extensions: Vec<String>,
    pub max_file_size: u64,
    pub supports_search: bool,
    pub supports_editing: bool,
}

/// Viewer content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewerContent {
    Text {
        content: String,
        encoding: String,
        language: Option<String>,
        line_count: usize,
    },
    Image {
        width: u32,
        height: u32,
        format: String,
        color_depth: u8,
        has_alpha: bool,
        metadata: Option<ImageMetadata>,
    },
    Binary {
        data: Vec<u8>,
        offset: u64,
        total_size: u64,
        display_format: BinaryDisplayFormat,
    },
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub exif: std::collections::HashMap<String, String>,
    pub creation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

/// Binary display format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryDisplayFormat {
    Hex { bytes_per_row: usize },
    Ascii,
    Mixed { bytes_per_row: usize },
}

/// Unified file viewer trait
#[async_trait]
pub trait FileViewer: Send + Sync {
    /// Get viewer capabilities
    fn capabilities(&self) -> ViewerCapabilities;

    /// Check if viewer can handle the given file
    fn can_handle(&self, path: &Path) -> bool;

    /// Open and process file for viewing
    async fn view_file(
        &self,
        path: &Path,
        options: ViewerOptions,
    ) -> Result<ViewerContent, ViewerError>;

    /// Search within file content (if supported)
    async fn search(
        &self,
        path: &Path,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, ViewerError>;
}

/// Viewer options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerOptions {
    pub encoding: Option<String>,
    pub max_size: Option<u64>,
    pub offset: Option<u64>,
    pub length: Option<u64>,
    pub syntax_highlighting: bool,
}

impl Default for ViewerOptions {
    fn default() -> Self {
        Self {
            encoding: None,
            max_size: Some(100 * 1024 * 1024), // 100MB default limit
            offset: None,
            length: None,
            syntax_highlighting: true,
        }
    }
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub regex: bool,
    pub max_results: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            regex: false,
            max_results: 1000,
        }
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub line_number: Option<usize>,
    pub offset: u64,
    pub length: usize,
    pub context_before: String,
    pub matched_text: String,
    pub context_after: String,
}

// Re-export viewer implementations
pub mod text;
pub mod image;
pub mod binary;

pub use text::TextViewer;
pub use image::ImageViewer; 
pub use binary::BinaryViewer;

/// Viewer factory for creating appropriate viewers
pub struct ViewerFactory {
    viewers: Vec<Box<dyn FileViewer>>,
}

impl ViewerFactory {
    /// Create a new viewer factory with default viewers
    pub fn new() -> Self {
        let mut factory = Self {
            viewers: Vec::new(),
        };
        
        // Register built-in viewers
        factory.register(Box::new(TextViewer::new()));
        factory.register(Box::new(ImageViewer::new()));
        factory.register(Box::new(BinaryViewer::new()));
        
        factory
    }

    /// Register a new viewer
    pub fn register(&mut self, viewer: Box<dyn FileViewer>) {
        self.viewers.push(viewer);
    }

    /// Find appropriate viewer for a file
    pub fn find_viewer(&self, path: &Path) -> Option<&dyn FileViewer> {
        self.viewers
            .iter()
            .find(|v| v.can_handle(path))
            .map(|v| v.as_ref())
    }

    /// Get all registered viewers
    pub fn get_all_viewers(&self) -> Vec<&dyn FileViewer> {
        self.viewers.iter().map(|v| v.as_ref()).collect()
    }
}

impl Default for ViewerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions
pub mod utils {
    use super::*;
    use std::ffi::OsStr;

    /// Get file extension from path
    pub fn get_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(OsStr::to_str)
            .map(|s| s.to_lowercase())
    }

    /// Detect MIME type from file extension
    pub fn detect_mime_type(path: &Path) -> Option<String> {
        get_extension(path).and_then(|ext| {
            match ext.as_str() {
                "txt" | "text" => Some("text/plain".to_string()),
                "md" | "markdown" => Some("text/markdown".to_string()),
                "json" => Some("application/json".to_string()),
                "xml" => Some("application/xml".to_string()),
                "html" | "htm" => Some("text/html".to_string()),
                "css" => Some("text/css".to_string()),
                "js" => Some("application/javascript".to_string()),
                "ts" => Some("application/typescript".to_string()),
                "rs" => Some("text/x-rust".to_string()),
                "py" => Some("text/x-python".to_string()),
                "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                "png" => Some("image/png".to_string()),
                "gif" => Some("image/gif".to_string()),
                "webp" => Some("image/webp".to_string()),
                "bmp" => Some("image/bmp".to_string()),
                "svg" => Some("image/svg+xml".to_string()),
                _ => None,
            }
        })
    }

    /// Format file size for display
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_factory_creation() {
        let factory = ViewerFactory::new();
        let viewers = factory.get_all_viewers();
        assert_eq!(viewers.len(), 3); // Text, Image, Binary viewers
    }

    #[test]
    fn test_mime_type_detection() {
        assert_eq!(
            utils::detect_mime_type(Path::new("test.txt")),
            Some("text/plain".to_string())
        );
        assert_eq!(
            utils::detect_mime_type(Path::new("image.png")),
            Some("image/png".to_string())
        );
        assert_eq!(
            utils::detect_mime_type(Path::new("unknown.xyz")),
            None
        );
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(utils::format_file_size(1024), "1.0 KB");
        assert_eq!(utils::format_file_size(1048576), "1.0 MB");
        assert_eq!(utils::format_file_size(500), "500 B");
    }
}