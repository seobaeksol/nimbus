use crate::{FileViewer, ViewerCapabilities, ViewerContent, ViewerOptions, ViewerError, SearchOptions, SearchResult, ImageMetadata};
use async_trait::async_trait;
use std::path::Path;
use image::{ImageFormat, GenericImageView, ColorType, io::Reader as ImageReader};
use std::collections::HashMap;

/// Image file viewer with metadata extraction and format support
pub struct ImageViewer {
    max_file_size: u64,
}

impl ImageViewer {
    /// Create a new image viewer with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB default limit
        }
    }

    /// Create a new image viewer with custom max file size
    pub fn with_max_size(max_size: u64) -> Self {
        Self {
            max_file_size: max_size,
        }
    }

    /// Get supported image file extensions
    fn get_supported_extensions(&self) -> Vec<String> {
        vec![
            // Common formats
            "jpg", "jpeg", "png", "gif", "bmp", "ico",
            // Advanced formats
            "webp", "tiff", "tif", "tga", "dds", "hdr", "exr",
            // RAW formats (limited support)
            "pnm", "pbm", "pgm", "ppm",
            // Vector formats that can be rasterized
            "svg",
        ].iter().map(|s| s.to_string()).collect()
    }

    /// Detect image format from file extension
    fn detect_format(&self, path: &Path) -> Option<ImageFormat> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        match extension.as_str() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "png" => Some(ImageFormat::Png),
            "gif" => Some(ImageFormat::Gif),
            "bmp" => Some(ImageFormat::Bmp),
            "ico" => Some(ImageFormat::Ico),
            "tiff" | "tif" => Some(ImageFormat::Tiff),
            "webp" => Some(ImageFormat::WebP),
            "tga" => Some(ImageFormat::Tga),
            "dds" => Some(ImageFormat::Dds),
            "hdr" => Some(ImageFormat::Hdr),
            "exr" => Some(ImageFormat::OpenExr),
            "pnm" | "pbm" | "pgm" | "ppm" => Some(ImageFormat::Pnm),
            _ => None,
        }
    }

    /// Extract EXIF data from image file (simplified - no EXIF support yet)
    async fn extract_exif_data(&self, _path: &Path) -> HashMap<String, String> {
        // TODO: Add EXIF support with proper crate
        HashMap::new()
    }

    /// Get color depth from ColorType
    fn get_color_depth(&self, color_type: ColorType) -> u8 {
        match color_type {
            ColorType::L8 => 8,
            ColorType::La8 => 16,
            ColorType::Rgb8 => 24,
            ColorType::Rgba8 => 32,
            ColorType::L16 => 16,
            ColorType::La16 => 32,
            ColorType::Rgb16 => 48,
            ColorType::Rgba16 => 64,
            ColorType::Rgb32F => 96,
            ColorType::Rgba32F => 128,
            _ => 8, // Default fallback
        }
    }

    /// Check if ColorType has alpha channel
    fn has_alpha_channel(&self, color_type: ColorType) -> bool {
        matches!(color_type, ColorType::La8 | ColorType::Rgba8 | ColorType::La16 | ColorType::Rgba16 | ColorType::Rgba32F)
    }

    /// Parse creation date from EXIF data
    fn parse_creation_date(&self, exif_data: &HashMap<String, String>) -> Option<chrono::DateTime<chrono::Utc>> {
        // Try common EXIF date fields
        let date_fields = ["DateTime", "DateTimeOriginal", "DateTimeDigitized"];
        
        for field in &date_fields {
            if let Some(date_str) = exif_data.get(*field) {
                // EXIF date format: "YYYY:MM:DD HH:MM:SS"
                if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S") {
                    return Some(chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc));
                }
            }
        }
        
        None
    }

    /// Extract camera information from EXIF data
    fn extract_camera_info(&self, exif_data: &HashMap<String, String>) -> (Option<String>, Option<String>) {
        let make = exif_data.get("Make").cloned();
        let model = exif_data.get("Model").cloned();
        (make, model)
    }
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileViewer for ImageViewer {
    fn capabilities(&self) -> ViewerCapabilities {
        ViewerCapabilities {
            name: "Image Viewer".to_string(),
            description: "View images with metadata extraction and EXIF support".to_string(),
            supported_extensions: self.get_supported_extensions(),
            max_file_size: self.max_file_size,
            supports_search: false, // Images don't support text search
            supports_editing: false,
        }
    }

    fn can_handle(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();
            self.get_supported_extensions().contains(&ext_lower)
        } else {
            false
        }
    }

    async fn view_file(
        &self,
        path: &Path,
        options: ViewerOptions,
    ) -> Result<ViewerContent, ViewerError> {
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ViewerError::FileNotFound {
                    path: path.to_string_lossy().to_string(),
                }
            } else {
                ViewerError::Io(e)
            }
        })?;

        let file_size = metadata.len();
        let max_size = options.max_size.unwrap_or(self.max_file_size);

        if file_size > max_size {
            return Err(ViewerError::FileTooLarge {
                size: file_size,
                max_size,
            });
        }

        // Open and read image in a blocking task to avoid blocking the async runtime
        let path_clone = path.to_path_buf();
        let image_info = tokio::task::spawn_blocking(move || -> Result<_, ViewerError> {
            // Open image file
            let img_reader = ImageReader::open(&path_clone)
                .map_err(|e| ViewerError::ImageError { message: e.to_string() })?;
                
            // Get format before consuming the reader
            let format = img_reader.format()
                .map(|f| format!("{:?}", f))
                .unwrap_or_else(|| "Unknown".to_string());
                
            // Decode image to get dimensions and format info
            let img = img_reader.decode().map_err(|e| ViewerError::ImageError { message: e.to_string() })?;
            
            // Get basic image information
            let (width, height) = img.dimensions();
            let color_type = img.color();
            
            Ok((width, height, color_type, format))
        }).await.map_err(|e| ViewerError::Other(format!("Failed to process image: {}", e)))??;

        let (width, height, color_type, format) = image_info;
        let color_depth = self.get_color_depth(color_type);
        let has_alpha = self.has_alpha_channel(color_type);

        // Extract EXIF data
        let exif_data = self.extract_exif_data(path).await;
        let creation_date = self.parse_creation_date(&exif_data);
        let (camera_make, camera_model) = self.extract_camera_info(&exif_data);

        let image_metadata = if !exif_data.is_empty() || creation_date.is_some() || camera_make.is_some() {
            Some(ImageMetadata {
                exif: exif_data,
                creation_date,
                camera_make,
                camera_model,
            })
        } else {
            None
        };

        Ok(ViewerContent::Image {
            width,
            height,
            format,
            color_depth,
            has_alpha,
            metadata: image_metadata,
        })
    }

    async fn search(
        &self,
        _path: &Path,
        _query: &str,
        _options: SearchOptions,
    ) -> Result<Vec<SearchResult>, ViewerError> {
        // Images don't support text-based search
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_image_viewer_capabilities() {
        let viewer = ImageViewer::new();
        let caps = viewer.capabilities();
        
        assert_eq!(caps.name, "Image Viewer");
        assert!(caps.supported_extensions.contains(&"jpg".to_string()));
        assert!(caps.supported_extensions.contains(&"png".to_string()));
        assert!(!caps.supports_search);
    }

    #[tokio::test]
    async fn test_can_handle() {
        let viewer = ImageViewer::new();
        
        assert!(viewer.can_handle(Path::new("test.jpg")));
        assert!(viewer.can_handle(Path::new("test.png")));
        assert!(viewer.can_handle(Path::new("test.gif")));
        assert!(!viewer.can_handle(Path::new("test.txt")));
        assert!(!viewer.can_handle(Path::new("test.exe")));
    }

    #[tokio::test]
    async fn test_format_detection() {
        let viewer = ImageViewer::new();
        
        assert_eq!(viewer.detect_format(Path::new("test.jpg")), Some(ImageFormat::Jpeg));
        assert_eq!(viewer.detect_format(Path::new("test.png")), Some(ImageFormat::Png));
        assert_eq!(viewer.detect_format(Path::new("test.gif")), Some(ImageFormat::Gif));
        assert_eq!(viewer.detect_format(Path::new("test.txt")), None);
    }

    #[tokio::test]
    async fn test_color_depth_and_alpha() {
        let viewer = ImageViewer::new();
        
        assert_eq!(viewer.get_color_depth(ColorType::Rgb8), 24);
        assert_eq!(viewer.get_color_depth(ColorType::Rgba8), 32);
        assert!(viewer.has_alpha_channel(ColorType::Rgba8));
        assert!(!viewer.has_alpha_channel(ColorType::Rgb8));
    }

    #[tokio::test]
    async fn test_view_synthetic_image() {
        let viewer = ImageViewer::new();
        
        // Create a simple test image
        let img = ImageBuffer::from_fn(100, 100, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255u8, 255u8, 255u8])
            } else {
                Rgb([0u8, 0u8, 0u8])
            }
        });
        
        // Save to temporary file
        let mut temp_file = NamedTempFile::with_suffix(".png").unwrap();
        img.save(temp_file.path()).unwrap();
        
        let options = ViewerOptions::default();
        let result = viewer.view_file(temp_file.path(), options).await.unwrap();
        
        match result {
            ViewerContent::Image { width, height, format, color_depth, has_alpha, .. } => {
                assert_eq!(width, 100);
                assert_eq!(height, 100);
                assert_eq!(format, "Png");
                assert_eq!(color_depth, 24); // RGB8
                assert!(!has_alpha);
            }
            _ => panic!("Expected image content"),
        }
    }
}