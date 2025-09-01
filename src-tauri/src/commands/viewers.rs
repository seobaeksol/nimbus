use crate::commands::CommandResult;
use tauri::AppHandle;
use nimbus_file_viewers::{ViewerFactory, ViewerOptions, SearchOptions, ViewerContent, FileViewer};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Global viewer factory instance
static VIEWER_FACTORY: std::sync::OnceLock<Arc<Mutex<ViewerFactory>>> = std::sync::OnceLock::new();

/// Get or initialize the global viewer factory
fn get_viewer_factory() -> Arc<Mutex<ViewerFactory>> {
    VIEWER_FACTORY.get_or_init(|| {
        Arc::new(Mutex::new(ViewerFactory::new()))
    }).clone()
}

/// Get all available viewers and their capabilities
#[tauri::command]
pub async fn get_viewer_capabilities() -> CommandResult<Vec<serde_json::Value>> {
    let factory = get_viewer_factory();
    let factory_guard = factory.lock().map_err(|e| format!("Failed to lock viewer factory: {}", e))?;
    
    let viewers = factory_guard.get_all_viewers();
    let mut capabilities = Vec::new();
    
    for viewer in viewers {
        let caps = viewer.capabilities();
        capabilities.push(serde_json::json!({
            "name": caps.name,
            "description": caps.description,
            "supported_extensions": caps.supported_extensions,
            "max_file_size": caps.max_file_size,
            "supports_search": caps.supports_search,
            "supports_editing": caps.supports_editing
        }));
    }
    
    Ok(capabilities)
}

/// Check which viewer can handle a specific file
#[tauri::command]
pub async fn find_viewer_for_file(path: String) -> CommandResult<Option<String>> {
    let factory = get_viewer_factory();
    let factory_guard = factory.lock().map_err(|e| format!("Failed to lock viewer factory: {}", e))?;
    
    let file_path = PathBuf::from(path);
    if let Some(viewer) = factory_guard.find_viewer(&file_path) {
        let caps = viewer.capabilities();
        Ok(Some(caps.name))
    } else {
        Ok(None)
    }
}

/// View a file using the appropriate viewer
#[tauri::command]
pub async fn view_file(
    _app: AppHandle,
    path: String,
    options: Option<serde_json::Value>
) -> CommandResult<serde_json::Value> {
    let file_path = PathBuf::from(&path);
    
    // Parse viewer options
    let viewer_options = if let Some(opts) = options {
        ViewerOptions {
            encoding: opts.get("encoding").and_then(|v| v.as_str()).map(String::from),
            max_size: opts.get("max_size").and_then(|v| v.as_u64()),
            offset: opts.get("offset").and_then(|v| v.as_u64()),
            length: opts.get("length").and_then(|v| v.as_u64()),
            syntax_highlighting: opts.get("syntax_highlighting")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        }
    } else {
        ViewerOptions::default()
    };

    // Find and use appropriate viewer without holding the lock
    let factory = get_viewer_factory();
    let viewer_result = {
        let factory_guard = factory.lock().map_err(|e| format!("Failed to lock viewer factory: {}", e))?;
        // Get viewer capabilities to determine which viewer to use
        factory_guard.find_viewer(&file_path)
            .map(|v| v.capabilities().name.clone())
            .ok_or_else(|| format!("No viewer available for file: {}", path))
    }?;

    // Create a new viewer instance based on the type
    let content = match viewer_result.as_str() {
        "Text Viewer" => {
            let viewer = nimbus_file_viewers::TextViewer::new();
            viewer.view_file(&file_path, viewer_options).await.map_err(|e| e.to_string())?
        }
        "Image Viewer" => {
            let viewer = nimbus_file_viewers::ImageViewer::new();
            viewer.view_file(&file_path, viewer_options).await.map_err(|e| e.to_string())?
        }
        "Binary Viewer" => {
            let viewer = nimbus_file_viewers::BinaryViewer::new();
            viewer.view_file(&file_path, viewer_options).await.map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Unknown viewer type: {}", viewer_result))
    };
    
    // Process the content
    let result = match content {
                ViewerContent::Text { content, encoding, language, line_count } => {
                    serde_json::json!({
                        "type": "text",
                        "content": content,
                        "encoding": encoding,
                        "language": language,
                        "line_count": line_count
                    })
                }
                ViewerContent::Image { width, height, format, color_depth, has_alpha, metadata } => {
                    let mut result = serde_json::json!({
                        "type": "image",
                        "width": width,
                        "height": height,
                        "format": format,
                        "color_depth": color_depth,
                        "has_alpha": has_alpha
                    });
                    
                    if let Some(meta) = metadata {
                        result["metadata"] = serde_json::json!({
                            "exif": meta.exif,
                            "creation_date": meta.creation_date,
                            "camera_make": meta.camera_make,
                            "camera_model": meta.camera_model
                        });
                    }
                    
                    result
                }
                ViewerContent::Binary { data, offset, total_size, display_format } => {
                    let format_info = match display_format {
                        nimbus_file_viewers::BinaryDisplayFormat::Hex { bytes_per_row } => {
                            serde_json::json!({
                                "type": "hex",
                                "bytes_per_row": bytes_per_row
                            })
                        }
                        nimbus_file_viewers::BinaryDisplayFormat::Ascii => {
                            serde_json::json!({
                                "type": "ascii"
                            })
                        }
                        nimbus_file_viewers::BinaryDisplayFormat::Mixed { bytes_per_row } => {
                            serde_json::json!({
                                "type": "mixed",
                                "bytes_per_row": bytes_per_row
                            })
                        }
                    };
                    
                    serde_json::json!({
                        "type": "binary",
                        "data": data,
                        "offset": offset,
                        "total_size": total_size,
                        "display_format": format_info
                    })
                }
            };
            
    Ok(result)
}

/// Search within a file using the appropriate viewer
#[tauri::command]
pub async fn search_in_file(
    _app: AppHandle,
    path: String,
    query: String,
    options: Option<serde_json::Value>
) -> CommandResult<Vec<serde_json::Value>> {
    let file_path = PathBuf::from(&path);
    
    // Find and use appropriate viewer without holding the lock
    let factory = get_viewer_factory();
    let viewer_result = {
        let factory_guard = factory.lock().map_err(|e| format!("Failed to lock viewer factory: {}", e))?;
        // Get viewer capabilities to determine which viewer to use
        factory_guard.find_viewer(&file_path)
            .map(|v| v.capabilities().name.clone())
            .ok_or_else(|| format!("No viewer available for file: {}", path))
    }?;

    // Check if viewer supports search
    let supports_search = match viewer_result.as_str() {
        "Text Viewer" => true,
        "Binary Viewer" => true,
        "Image Viewer" => false,
        _ => false,
    };
    
    if !supports_search {
        return Err(format!("Viewer for {} does not support search", path));
    }
    
    // Parse search options
    let search_options = if let Some(opts) = options {
        SearchOptions {
            case_sensitive: opts.get("case_sensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            regex: opts.get("regex")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            max_results: opts.get("max_results")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(1000),
        }
    } else {
        SearchOptions::default()
    };
    
    // Create a new viewer instance and perform search
    let results = match viewer_result.as_str() {
        "Text Viewer" => {
            let viewer = nimbus_file_viewers::TextViewer::new();
            viewer.search(&file_path, &query, search_options).await.map_err(|e| e.to_string())?
        }
        "Binary Viewer" => {
            let viewer = nimbus_file_viewers::BinaryViewer::new();
            viewer.search(&file_path, &query, search_options).await.map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Search not supported for viewer: {}", viewer_result))
    };

    let json_results = results.into_iter().map(|result| {
        serde_json::json!({
            "line_number": result.line_number,
            "offset": result.offset,
            "length": result.length,
            "context_before": result.context_before,
            "matched_text": result.matched_text,
            "context_after": result.context_after
        })
    }).collect();

    Ok(json_results)
}

/// Read a portion of a binary file (for hex viewer pagination)
#[tauri::command]
pub async fn read_binary_chunk(
    path: String,
    offset: u64,
    length: u64
) -> CommandResult<serde_json::Value> {
    let file_path = PathBuf::from(&path);
    
    let viewer_options = ViewerOptions {
        encoding: None,
        max_size: None,
        offset: Some(offset),
        length: Some(length),
        syntax_highlighting: false,
    };
    
    // Use binary viewer directly
    let viewer = nimbus_file_viewers::BinaryViewer::new();
    match viewer.view_file(&file_path, viewer_options).await.map_err(|e| e.to_string())? {
        ViewerContent::Binary { data, offset, total_size, display_format } => {
            let format_info = match display_format {
                nimbus_file_viewers::BinaryDisplayFormat::Hex { bytes_per_row } => {
                    serde_json::json!({
                        "type": "hex",
                        "bytes_per_row": bytes_per_row
                    })
                }
                nimbus_file_viewers::BinaryDisplayFormat::Ascii => {
                    serde_json::json!({
                        "type": "ascii"
                    })
                }
                nimbus_file_viewers::BinaryDisplayFormat::Mixed { bytes_per_row } => {
                    serde_json::json!({
                        "type": "mixed",
                        "bytes_per_row": bytes_per_row
                    })
                }
            };
            
            Ok(serde_json::json!({
                "data": data,
                "offset": offset,
                "total_size": total_size,
                "display_format": format_info
            }))
        }
        _ => Err("File is not binary or viewer returned unexpected content type".to_string()),
    }
}

/// Get file viewer statistics
#[tauri::command]
pub async fn get_viewer_stats() -> CommandResult<serde_json::Value> {
    let factory = get_viewer_factory();
    let factory_guard = factory.lock().map_err(|e| format!("Failed to lock viewer factory: {}", e))?;
    
    let viewers = factory_guard.get_all_viewers();
    let total_extensions: std::collections::HashSet<String> = viewers.iter()
        .flat_map(|v| v.capabilities().supported_extensions.into_iter())
        .collect();
    
    Ok(serde_json::json!({
        "total_viewers": viewers.len(),
        "total_supported_extensions": total_extensions.len(),
        "supported_extensions": total_extensions.into_iter().collect::<Vec<_>>(),
        "viewers": viewers.iter().map(|v| {
            let caps = v.capabilities();
            serde_json::json!({
                "name": caps.name,
                "extensions_count": caps.supported_extensions.len(),
                "max_file_size": caps.max_file_size,
                "supports_search": caps.supports_search,
                "supports_editing": caps.supports_editing
            })
        }).collect::<Vec<_>>()
    }))
}

/// Utility function to format file size for display
#[tauri::command]
pub async fn format_file_size(size: u64) -> CommandResult<String> {
    Ok(nimbus_file_viewers::utils::format_file_size(size))
}

/// Utility function to detect MIME type
#[tauri::command]
pub async fn detect_mime_type(path: String) -> CommandResult<Option<String>> {
    let file_path = std::path::Path::new(&path);
    Ok(nimbus_file_viewers::utils::detect_mime_type(file_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_get_viewer_capabilities() {
        let result = get_viewer_capabilities().await.unwrap();
        assert!(!result.is_empty());
        
        // Should have at least text, image, and binary viewers
        assert!(result.len() >= 3);
    }

    #[tokio::test]
    async fn test_find_viewer_for_file() {
        let result = find_viewer_for_file("test.txt".to_string()).await.unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("Text"));
        
        let result = find_viewer_for_file("test.jpg".to_string()).await.unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("Image"));
    }

    #[tokio::test]
    async fn test_format_file_size() {
        assert_eq!(format_file_size(1024).await.unwrap(), "1.0 KB");
        assert_eq!(format_file_size(1048576).await.unwrap(), "1.0 MB");
        assert_eq!(format_file_size(500).await.unwrap(), "500 B");
    }

    #[tokio::test]
    async fn test_detect_mime_type() {
        let result = detect_mime_type("test.txt".to_string()).await.unwrap();
        assert_eq!(result, Some("text/plain".to_string()));
        
        let result = detect_mime_type("test.jpg".to_string()).await.unwrap();
        assert_eq!(result, Some("image/jpeg".to_string()));
        
        let result = detect_mime_type("test.unknown".to_string()).await.unwrap();
        assert_eq!(result, None);
    }
}