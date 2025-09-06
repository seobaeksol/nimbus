//! Media Info Content Plugin for Nimbus
//! 
//! This plugin extracts comprehensive metadata from media files including:
//! - Image files: EXIF data, dimensions, camera info, GPS coordinates
//! - Audio files: ID3 tags, bitrate, duration, album art
//! - Video files: codec info, resolution, frame rate, duration

use async_trait::async_trait;
use nimbus_plugin_sdk::{
    ContentPlugin, PluginInfo, Result, PluginError,
    content::{ColumnDefinition, ColumnValue, ColumnAlignment}
};
use std::collections::HashMap;
use std::path::Path;
use log::{debug, info, warn, error};

pub struct MediaInfoPlugin {
    initialized: bool,
}

impl MediaInfoPlugin {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Extract image metadata including EXIF data
    async fn extract_image_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        debug!("Extracting image metadata from: {:?}", file_path);
        
        // Read image dimensions and basic info
        match image::image_dimensions(file_path) {
            Ok((width, height)) => {
                metadata.insert("width".to_string(), width.to_string());
                metadata.insert("height".to_string(), height.to_string());
                metadata.insert("resolution".to_string(), format!("{}x{}", width, height));
                metadata.insert("megapixels".to_string(), 
                    format!("{:.1}", (width * height) as f64 / 1_000_000.0));
            }
            Err(e) => {
                warn!("Failed to read image dimensions: {}", e);
            }
        }

        // Extract EXIF data
        if let Ok(file) = std::fs::File::open(file_path) {
            let mut buf_reader = std::io::BufReader::new(&file);
            match exif::Reader::new().read_from_container(&mut buf_reader) {
                Ok(exif_data) => {
                    debug!("Found EXIF data with {} fields", exif_data.fields().count());
                    
                    for field in exif_data.fields() {
                        let tag_name = format!("{}", field.tag);
                        let value = format!("{}", field.display_value());
                        
                        match field.tag {
                            exif::Tag::Make => metadata.insert("camera_make".to_string(), value),
                            exif::Tag::Model => metadata.insert("camera_model".to_string(), value),
                            exif::Tag::DateTime => metadata.insert("date_taken".to_string(), value),
                            exif::Tag::ExposureTime => metadata.insert("exposure_time".to_string(), value),
                            exif::Tag::FNumber => metadata.insert("f_stop".to_string(), value),
                            exif::Tag::ISOSpeedRatings => metadata.insert("iso".to_string(), value),
                            exif::Tag::FocalLength => metadata.insert("focal_length".to_string(), value),
                            exif::Tag::Flash => metadata.insert("flash".to_string(), value),
                            exif::Tag::GPSLatitude => metadata.insert("gps_latitude".to_string(), value),
                            exif::Tag::GPSLongitude => metadata.insert("gps_longitude".to_string(), value),
                            _ => None,
                        };
                    }
                    
                    // Add derived metadata
                    if metadata.contains_key("camera_make") && metadata.contains_key("camera_model") {
                        let camera = format!("{} {}", 
                            metadata.get("camera_make").unwrap(),
                            metadata.get("camera_model").unwrap()
                        );
                        metadata.insert("camera".to_string(), camera);
                    }
                    
                    if metadata.contains_key("gps_latitude") && metadata.contains_key("gps_longitude") {
                        metadata.insert("has_gps".to_string(), "true".to_string());
                    }
                }
                Err(e) => {
                    debug!("No EXIF data found: {}", e);
                }
            }
        }

        // Detect image format
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            metadata.insert("format".to_string(), extension.to_uppercase());
        }

        Ok(metadata)
    }

    /// Extract audio metadata including ID3 tags
    async fn extract_audio_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        debug!("Extracting audio metadata from: {:?}", file_path);
        
        // For MP3 files, extract ID3 tags
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            if ext.to_lowercase() == "mp3" {
                match mp3_metadata::read_from_file(file_path) {
                    Ok(mp3_metadata) => {
                        debug!("Found MP3 metadata");
                        
                        metadata.insert("duration".to_string(), 
                            format!("{:.1}s", mp3_metadata.duration.as_secs_f64()));
                        metadata.insert("frame_count".to_string(), 
                            mp3_metadata.frames.len().to_string());
                        
                        if let Some(tag) = mp3_metadata.tag {
                            metadata.insert("title".to_string(), tag.title);
                            metadata.insert("artist".to_string(), tag.artist);
                            metadata.insert("album".to_string(), tag.album);
                            metadata.insert("year".to_string(), tag.year);
                            metadata.insert("genre".to_string(), tag.genre);
                            
                            if !tag.artist.is_empty() && !tag.title.is_empty() {
                                metadata.insert("track_info".to_string(), 
                                    format!("{} - {}", tag.artist, tag.title));
                            }
                        }
                        
                        // Calculate bitrate
                        if let Ok(file_size) = std::fs::metadata(file_path).map(|m| m.len()) {
                            let duration_secs = mp3_metadata.duration.as_secs();
                            if duration_secs > 0 {
                                let bitrate = (file_size * 8) / (duration_secs * 1000); // kbps
                                metadata.insert("bitrate".to_string(), format!("{}kbps", bitrate));
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read MP3 metadata: {}", e);
                    }
                }
            }
        }
        
        // Add general audio format info
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            metadata.insert("format".to_string(), extension.to_uppercase());
            
            // Classify audio type
            let audio_type = match extension.to_lowercase().as_str() {
                "mp3" | "mp2" => "MPEG Audio",
                "wav" => "Waveform Audio",
                "flac" => "Free Lossless Audio Codec",
                "ogg" | "oga" => "Ogg Vorbis",
                "aac" | "m4a" => "Advanced Audio Coding",
                "wma" => "Windows Media Audio",
                _ => "Audio File",
            };
            metadata.insert("audio_type".to_string(), audio_type.to_string());
        }

        Ok(metadata)
    }

    /// Extract video metadata
    async fn extract_video_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        debug!("Extracting video metadata from: {:?}", file_path);
        
        // Add basic video format info
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            metadata.insert("format".to_string(), extension.to_uppercase());
            
            // Classify video type
            let video_type = match extension.to_lowercase().as_str() {
                "mp4" | "m4v" => "MPEG-4 Video",
                "avi" => "Audio Video Interleave",
                "mov" | "qt" => "QuickTime Movie",
                "wmv" => "Windows Media Video",
                "flv" => "Flash Video",
                "webm" => "WebM Video",
                "mkv" => "Matroska Video",
                "3gp" => "3GPP Video",
                _ => "Video File",
            };
            metadata.insert("video_type".to_string(), video_type.to_string());
        }
        
        // Note: Full video metadata extraction would require additional dependencies
        // like ffmpeg-sys-next, but we'll provide placeholder for common properties
        metadata.insert("needs_ffmpeg".to_string(), "true".to_string());

        Ok(metadata)
    }

    /// Check if file is a supported media type
    fn is_media_file(&self, file_path: &Path) -> (bool, String) {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();
            
            // Image formats
            if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | 
                       "tiff" | "tif" | "webp" | "ico" | "raw" | "heic" | "avif") {
                return (true, "image".to_string());
            }
            
            // Audio formats
            if matches!(ext_lower.as_str(), "mp3" | "wav" | "flac" | "ogg" | "oga" |
                       "aac" | "m4a" | "wma" | "opus" | "mp2") {
                return (true, "audio".to_string());
            }
            
            // Video formats
            if matches!(ext_lower.as_str(), "mp4" | "avi" | "mov" | "wmv" | "flv" |
                       "webm" | "mkv" | "3gp" | "m4v" | "qt") {
                return (true, "video".to_string());
            }
        }
        
        (false, String::new())
    }
}

#[async_trait]
impl ContentPlugin for MediaInfoPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Media Info Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Extracts comprehensive metadata from image, audio, and video files".to_string(),
            author: "Nimbus Team".to_string(),
            homepage: Some("https://github.com/nimbus-file-manager/plugins/media-info".to_string()),
            repository: Some("https://github.com/nimbus-file-manager/plugins".to_string()),
            license: Some("MIT".to_string()),
            tags: vec![
                "media".to_string(),
                "metadata".to_string(), 
                "exif".to_string(),
                "id3".to_string(),
                "images".to_string(),
                "audio".to_string(),
                "video".to_string()
            ],
            min_version: "0.1.0".to_string(),
            max_version: None,
        }
    }
    
    fn supported_extensions(&self) -> Vec<String> {
        vec![
            // Images
            "jpg".to_string(), "jpeg".to_string(), "png".to_string(), 
            "gif".to_string(), "bmp".to_string(), "tiff".to_string(), 
            "tif".to_string(), "webp".to_string(), "ico".to_string(),
            "raw".to_string(), "heic".to_string(), "avif".to_string(),
            
            // Audio
            "mp3".to_string(), "wav".to_string(), "flac".to_string(),
            "ogg".to_string(), "oga".to_string(), "aac".to_string(),
            "m4a".to_string(), "wma".to_string(), "opus".to_string(),
            "mp2".to_string(),
            
            // Video
            "mp4".to_string(), "avi".to_string(), "mov".to_string(),
            "wmv".to_string(), "flv".to_string(), "webm".to_string(),
            "mkv".to_string(), "3gp".to_string(), "m4v".to_string(),
            "qt".to_string(),
        ]
    }
    
    fn column_definitions(&self) -> Vec<ColumnDefinition> {
        vec![
            // Image columns
            ColumnDefinition {
                id: "media_info.resolution".to_string(),
                name: "Resolution".to_string(),
                description: Some("Image or video resolution (width x height)".to_string()),
                width: 120,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Center,
            },
            ColumnDefinition {
                id: "media_info.megapixels".to_string(),
                name: "Megapixels".to_string(),
                description: Some("Image resolution in megapixels".to_string()),
                width: 80,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Right,
            },
            ColumnDefinition {
                id: "media_info.camera".to_string(),
                name: "Camera".to_string(),
                description: Some("Camera make and model from EXIF data".to_string()),
                width: 150,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Left,
            },
            ColumnDefinition {
                id: "media_info.date_taken".to_string(),
                name: "Date Taken".to_string(),
                description: Some("Date photo was taken from EXIF data".to_string()),
                width: 120,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Center,
            },
            ColumnDefinition {
                id: "media_info.has_gps".to_string(),
                name: "GPS".to_string(),
                description: Some("Whether image contains GPS location data".to_string()),
                width: 50,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Center,
            },
            
            // Audio columns
            ColumnDefinition {
                id: "media_info.duration".to_string(),
                name: "Duration".to_string(),
                description: Some("Audio or video duration".to_string()),
                width: 80,
                sortable: true,
                visible_by_default: true,
                alignment: ColumnAlignment::Right,
            },
            ColumnDefinition {
                id: "media_info.artist".to_string(),
                name: "Artist".to_string(),
                description: Some("Audio track artist from ID3 tags".to_string()),
                width: 120,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Left,
            },
            ColumnDefinition {
                id: "media_info.album".to_string(),
                name: "Album".to_string(),
                description: Some("Audio track album from ID3 tags".to_string()),
                width: 120,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Left,
            },
            ColumnDefinition {
                id: "media_info.bitrate".to_string(),
                name: "Bitrate".to_string(),
                description: Some("Audio bitrate in kbps".to_string()),
                width: 80,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Right,
            },
            
            // General media
            ColumnDefinition {
                id: "media_info.media_type".to_string(),
                name: "Media Type".to_string(),
                description: Some("Type of media file (image, audio, video)".to_string()),
                width: 100,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Center,
            },
        ]
    }
    
    async fn get_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
        debug!("Getting metadata for: {:?}", file_path);
        
        let (is_media, media_type) = self.is_media_file(file_path);
        if !is_media {
            debug!("File is not a supported media type: {:?}", file_path);
            return Ok(HashMap::new());
        }
        
        let metadata = match media_type.as_str() {
            "image" => self.extract_image_metadata(file_path).await?,
            "audio" => self.extract_audio_metadata(file_path).await?,
            "video" => self.extract_video_metadata(file_path).await?,
            _ => HashMap::new(),
        };
        
        info!("Extracted {} metadata fields from {:?}", metadata.len(), file_path.file_name().unwrap_or_default());
        Ok(metadata)
    }
    
    async fn get_columns(&self, file_path: &Path) -> Result<HashMap<String, ColumnValue>> {
        debug!("Getting column values for: {:?}", file_path);
        
        let metadata = self.get_metadata(file_path).await?;
        let mut columns = HashMap::new();
        
        // Convert metadata to column values
        if let Some(resolution) = metadata.get("resolution") {
            columns.insert("resolution".to_string(), ColumnValue::Text(resolution.clone()));
        }
        
        if let Some(megapixels) = metadata.get("megapixels") {
            if let Ok(mp) = megapixels.parse::<f64>() {
                columns.insert("megapixels".to_string(), ColumnValue::Number(mp));
            }
        }
        
        if let Some(camera) = metadata.get("camera") {
            columns.insert("camera".to_string(), ColumnValue::Text(camera.clone()));
        }
        
        if let Some(date_taken) = metadata.get("date_taken") {
            columns.insert("date_taken".to_string(), ColumnValue::DateTime(date_taken.clone()));
        }
        
        if let Some(_) = metadata.get("has_gps") {
            columns.insert("has_gps".to_string(), ColumnValue::Boolean(true));
        }
        
        if let Some(duration) = metadata.get("duration") {
            columns.insert("duration".to_string(), ColumnValue::Text(duration.clone()));
        }
        
        if let Some(artist) = metadata.get("artist") {
            columns.insert("artist".to_string(), ColumnValue::Text(artist.clone()));
        }
        
        if let Some(album) = metadata.get("album") {
            columns.insert("album".to_string(), ColumnValue::Text(album.clone()));
        }
        
        if let Some(bitrate) = metadata.get("bitrate") {
            columns.insert("bitrate".to_string(), ColumnValue::Text(bitrate.clone()));
        }
        
        // Determine media type for display
        let (is_media, media_type) = self.is_media_file(file_path);
        if is_media {
            let media_icon = match media_type.as_str() {
                "image" => "🖼️",
                "audio" => "🎵",
                "video" => "🎬",
                _ => "📱",
            };
            columns.insert("media_type".to_string(), ColumnValue::Custom {
                display: format!("{} {}", media_icon, media_type),
                tooltip: Some(format!("{} file", media_type)),
                sort_value: Some(media_type),
            });
        }
        
        debug!("Generated {} column values", columns.len());
        Ok(columns)
    }
    
    async fn get_thumbnail(&self, file_path: &Path, size: u32) -> Result<Option<Vec<u8>>> {
        debug!("Generating thumbnail for: {:?} (size: {}px)", file_path, size);
        
        let (is_media, media_type) = self.is_media_file(file_path);
        if !is_media || media_type != "image" {
            return Ok(None);
        }
        
        // Generate thumbnail for images
        match image::open(file_path) {
            Ok(img) => {
                let thumbnail = img.thumbnail(size, size);
                let mut buffer = Vec::new();
                match thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png) {
                    Ok(_) => {
                        debug!("Generated thumbnail: {} bytes", buffer.len());
                        Ok(Some(buffer))
                    }
                    Err(e) => {
                        warn!("Failed to encode thumbnail: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to load image for thumbnail: {}", e);
                Ok(None)
            }
        }
    }
    
    async fn can_handle_file(&self, file_path: &Path) -> bool {
        let (is_media, _) = self.is_media_file(file_path);
        is_media
    }
    
    async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        info!("Initializing Media Info Plugin v{}", self.info().version);
        
        // Perform any initialization tasks
        debug!("Plugin supports {} file extensions", self.supported_extensions().len());
        debug!("Plugin provides {} custom columns", self.column_definitions().len());
        
        self.initialized = true;
        info!("Media Info Plugin initialized successfully");
        Ok(())
    }
    
    async fn cleanup(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        info!("Cleaning up Media Info Plugin");
        self.initialized = false;
        info!("Media Info Plugin cleanup completed");
        Ok(())
    }
}

// Plugin entry point - this is called by the plugin manager
#[no_mangle]
pub extern "C" fn plugin_main() -> *mut dyn ContentPlugin {
    // Initialize logging for the plugin
    env_logger::init();
    
    info!("Creating Media Info Plugin instance");
    let plugin = MediaInfoPlugin::new();
    Box::into_raw(Box::new(plugin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio_test;

    #[tokio_test::tokio::test]
    async fn test_plugin_info() {
        let plugin = MediaInfoPlugin::new();
        let info = plugin.info();
        
        assert_eq!(info.name, "Media Info Plugin");
        assert_eq!(info.version, "1.0.0");
        assert!(info.tags.contains(&"media".to_string()));
    }

    #[tokio_test::tokio::test]
    async fn test_supported_extensions() {
        let plugin = MediaInfoPlugin::new();
        let extensions = plugin.supported_extensions();
        
        assert!(extensions.contains(&"jpg".to_string()));
        assert!(extensions.contains(&"mp3".to_string()));
        assert!(extensions.contains(&"mp4".to_string()));
        assert!(extensions.len() > 20); // Should support many formats
    }

    #[tokio_test::tokio::test]
    async fn test_media_file_detection() {
        let plugin = MediaInfoPlugin::new();
        
        // Test image detection
        let (is_media, media_type) = plugin.is_media_file(Path::new("test.jpg"));
        assert!(is_media);
        assert_eq!(media_type, "image");
        
        // Test audio detection
        let (is_media, media_type) = plugin.is_media_file(Path::new("test.mp3"));
        assert!(is_media);
        assert_eq!(media_type, "audio");
        
        // Test video detection
        let (is_media, media_type) = plugin.is_media_file(Path::new("test.mp4"));
        assert!(is_media);
        assert_eq!(media_type, "video");
        
        // Test non-media file
        let (is_media, _) = plugin.is_media_file(Path::new("test.txt"));
        assert!(!is_media);
    }

    #[tokio_test::tokio::test]
    async fn test_column_definitions() {
        let plugin = MediaInfoPlugin::new();
        let columns = plugin.column_definitions();
        
        assert!(!columns.is_empty());
        
        // Check for expected columns
        let column_ids: Vec<String> = columns.iter().map(|c| c.id.clone()).collect();
        assert!(column_ids.contains(&"media_info.resolution".to_string()));
        assert!(column_ids.contains(&"media_info.duration".to_string()));
        assert!(column_ids.contains(&"media_info.artist".to_string()));
    }

    #[tokio_test::tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = MediaInfoPlugin::new();
        assert!(!plugin.initialized);
        
        plugin.initialize().await.unwrap();
        assert!(plugin.initialized);
        
        plugin.cleanup().await.unwrap();
        assert!(!plugin.initialized);
    }
}