//! Archive commands for browsing and extracting archive files
//!
//! Provides Tauri IPC commands for interacting with archive files
//! including listing contents, extracting files, and creating archives.

use nimbus_archive::{
    ArchiveFactory, ArchiveEntry, ExtractionOptions, 
    OverwritePolicy, ProgressInfo
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{State, Emitter};
use tokio::sync::mpsc;

/// Serializable version of ArchiveEntry for Tauri IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntryResponse {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
    pub modified: Option<String>, // ISO 8601 timestamp
    pub is_directory: bool,
    pub compression_method: Option<String>,
    pub crc32: Option<u32>,
    pub is_encrypted: bool,
}

impl From<ArchiveEntry> for ArchiveEntryResponse {
    fn from(entry: ArchiveEntry) -> Self {
        Self {
            path: entry.path,
            name: entry.name,
            size: entry.size,
            compressed_size: entry.compressed_size,
            modified: entry.modified.map(|dt| dt.to_rfc3339()),
            is_directory: entry.is_directory,
            compression_method: entry.compression_method,
            crc32: entry.crc32,
            is_encrypted: entry.is_encrypted,
        }
    }
}

/// Options for archive extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionOptionsRequest {
    pub preserve_paths: bool,
    pub overwrite_policy: String, // "skip", "overwrite", "rename", "ask"
    pub entries: Option<Vec<String>>, // Specific entries to extract (None = all)
}

impl From<ExtractionOptionsRequest> for ExtractionOptions {
    fn from(opts: ExtractionOptionsRequest) -> Self {
        let policy = match opts.overwrite_policy.as_str() {
            "skip" => OverwritePolicy::Skip,
            "overwrite" => OverwritePolicy::Overwrite,
            "rename" => OverwritePolicy::Rename,
            _ => OverwritePolicy::Ask, // Default to ask
        };

        Self {
            preserve_paths: opts.preserve_paths,
            overwrite_policy: policy,
            entries: opts.entries,
            create_subfolder: false,
            password: None,
        }
    }
}

/// Progress information for extraction operations
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionProgress {
    pub operation_id: String,
    pub files_processed: u64,
    pub total_files: u64,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub current_file: Option<String>,
    pub status: String, // "running", "completed", "error", "cancelled"
}

/// Global state for tracking active operations
#[derive(Default)]
pub struct OperationState {
    pub operations: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<()>>>>,
}

/// List the contents of an archive file
#[tauri::command]
pub async fn list_archive_contents(
    archive_path: String,
    internal_path: Option<String>,
) -> Result<Vec<ArchiveEntryResponse>, String> {
    let path = PathBuf::from(archive_path);
    
    // Verify archive exists
    if !path.exists() {
        return Err(format!("Archive file not found: {}", path.display()));
    }

    // Create archive reader based on format
    let reader = ArchiveFactory::create_reader(path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    // List entries
    let entries = reader.list_entries()
        .await
        .map_err(|e| format!("Failed to list archive contents: {}", e))?;

    // Filter entries if internal_path is specified
    let filtered_entries = if let Some(internal_dir) = internal_path {
        entries.into_iter()
            .filter(|entry| entry.path.starts_with(&internal_dir))
            .collect()
    } else {
        entries
    };

    // Convert to response format
    Ok(filtered_entries.into_iter().map(|e| e.into()).collect())
}

/// Extract files from an archive
#[tauri::command]
pub async fn extract_archive(
    app_handle: tauri::AppHandle,
    operation_state: State<'_, OperationState>,
    archive_path: String,
    destination: String,
    options: ExtractionOptionsRequest,
) -> Result<String, String> {
    let archive_path = PathBuf::from(archive_path);
    let destination = PathBuf::from(destination);
    
    // Verify inputs
    if !archive_path.exists() {
        return Err(format!("Archive file not found: {}", archive_path.display()));
    }
    
    if !destination.exists() {
        return Err(format!("Destination directory not found: {}", destination.display()));
    }

    // Create operation ID
    let operation_id = uuid::Uuid::new_v4().to_string();
    
    // Create cancellation channel
    let (cancel_tx, _cancel_rx) = mpsc::unbounded_channel::<()>();
    
    // Store operation in global state
    {
        let mut operations = operation_state.operations.lock().unwrap();
        operations.insert(operation_id.clone(), cancel_tx);
    }

    // Create archive reader
    let reader = ArchiveFactory::create_reader(archive_path.clone())
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let extraction_options = ExtractionOptions::from(options);
    let op_id = operation_id.clone();
    let app_handle_clone = app_handle.clone();
    
    // Start extraction in background task
    tokio::spawn(async move {
        let progress_callback = {
            let op_id = op_id.clone();
            let app_handle = app_handle_clone.clone();
            
            Box::new(move |progress: ProgressInfo| {
                let progress_data = ExtractionProgress {
                    operation_id: op_id.clone(),
                    files_processed: progress.files_processed as u64,
                    total_files: progress.total_files as u64,
                    bytes_processed: progress.bytes_processed,
                    total_bytes: progress.total_bytes,
                    current_file: Some(progress.current_file.clone()),
                    status: "running".to_string(),
                };
                
                // Emit progress event to frontend
                let _ = app_handle.emit("archive-extraction-progress", &progress_data);
            }) as Box<dyn Fn(ProgressInfo) + Send + Sync>
        };

        // Perform extraction
        let result = reader.extract(&destination, extraction_options, Some(progress_callback)).await;
        
        // Send completion/error event
        let final_progress = match result {
            Ok(_) => ExtractionProgress {
                operation_id: op_id.clone(),
                files_processed: 0, // Will be filled by actual progress
                total_files: 0,
                bytes_processed: 0,
                total_bytes: 0,
                current_file: None,
                status: "completed".to_string(),
            },
            Err(e) => {
                eprintln!("Extraction failed: {}", e);
                ExtractionProgress {
                    operation_id: op_id.clone(),
                    files_processed: 0,
                    total_files: 0,
                    bytes_processed: 0,
                    total_bytes: 0,
                    current_file: None,
                    status: format!("error: {}", e),
                }
            }
        };
        
        let _ = app_handle.emit("archive-extraction-progress", &final_progress);
        
        // Clean up operation from state
        // Note: We'll need to access the operation_state here, but it's not easily accessible
        // in this closure. For now, operations will be cleaned up manually or on app shutdown.
    });

    Ok(operation_id)
}

/// Cancel an ongoing extraction operation
#[tauri::command]
pub async fn cancel_extraction(
    operation_state: State<'_, OperationState>,
    operation_id: String,
) -> Result<(), String> {
    let mut operations = operation_state.operations.lock().unwrap();
    
    if let Some(cancel_tx) = operations.remove(&operation_id) {
        let _ = cancel_tx.send(());
        Ok(())
    } else {
        Err(format!("Operation not found: {}", operation_id))
    }
}

/// Extract a single entry from an archive to memory (for preview)
#[tauri::command]
pub async fn extract_entry_to_memory(
    archive_path: String,
    entry_path: String,
    max_size: Option<u64>, // Maximum size to read (for safety)
) -> Result<Vec<u8>, String> {
    let path = PathBuf::from(archive_path);
    let _max_size = max_size.unwrap_or(10 * 1024 * 1024); // Default 10MB limit
    
    // Verify archive exists
    if !path.exists() {
        return Err(format!("Archive file not found: {}", path.display()));
    }

    // Create archive reader
    let reader = ArchiveFactory::create_reader(path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    // Extract to memory (ignore max_size for now as the API doesn't support it)
    reader.extract_entry_to_memory(&entry_path)
        .await
        .map_err(|e| format!("Failed to extract entry: {}", e))
}

/// Get information about an archive file
#[tauri::command]
pub async fn get_archive_info(archive_path: String) -> Result<ArchiveInfoResponse, String> {
    let path = PathBuf::from(archive_path);
    
    if !path.exists() {
        return Err(format!("Archive file not found: {}", path.display()));
    }

    let reader = ArchiveFactory::create_reader(path.clone())
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let entries = reader.list_entries()
        .await
        .map_err(|e| format!("Failed to list archive contents: {}", e))?;

    let total_files = entries.iter().filter(|e| !e.is_directory).count() as u64;
    let total_directories = entries.iter().filter(|e| e.is_directory).count() as u64;
    let total_size = entries.iter().map(|e| e.size).sum::<u64>();
    let compressed_size = entries.iter().map(|e| e.compressed_size).sum::<u64>();

    Ok(ArchiveInfoResponse {
        format: format!("{:?}", reader.format()),
        total_entries: entries.len() as u64,
        total_files,
        total_directories,
        total_size,
        compressed_size,
        compression_ratio: if total_size > 0 {
            Some((compressed_size as f64 / total_size as f64) * 100.0)
        } else {
            None
        },
    })
}

/// Archive information response
#[derive(Debug, Serialize)]
pub struct ArchiveInfoResponse {
    pub format: String,
    pub total_entries: u64,
    pub total_files: u64,
    pub total_directories: u64,
    pub total_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: Option<f64>, // Percentage
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::io::Write;

    async fn create_test_archive() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = temp_dir.path().join("test.zip");
        
        // Create a simple ZIP file
        let file = fs::File::create(&zip_path).expect("Failed to create zip file");
        let mut zip_writer = zip::ZipWriter::new(file);
        
        zip_writer.start_file("test.txt", zip::write::SimpleFileOptions::default())
            .expect("Failed to start file");
        zip_writer.write_all(b"Test content").expect("Failed to write content");
        
        zip_writer.finish().expect("Failed to finish zip");
        
        (temp_dir, zip_path)
    }

    #[tokio::test]
    async fn test_list_archive_contents_command() {
        let (_temp_dir, zip_path) = create_test_archive().await;
        
        let result = list_archive_contents(
            zip_path.to_string_lossy().to_string(),
            None
        ).await;
        
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test.txt");
        assert!(!entries[0].is_directory);
    }

    #[tokio::test]
    async fn test_get_archive_info_command() {
        let (_temp_dir, zip_path) = create_test_archive().await;
        
        let result = get_archive_info(zip_path.to_string_lossy().to_string()).await;
        
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, "Zip");
        assert_eq!(info.total_entries, 1);
        assert_eq!(info.total_files, 1);
        assert_eq!(info.total_directories, 0);
    }

    #[tokio::test]
    async fn test_extract_entry_to_memory_command() {
        let (_temp_dir, zip_path) = create_test_archive().await;
        
        let result = extract_entry_to_memory(
            zip_path.to_string_lossy().to_string(),
            "test.txt".to_string(),
            None
        ).await;
        
        assert!(result.is_ok());
        let content = result.unwrap();
        assert_eq!(content, b"Test content");
    }
}