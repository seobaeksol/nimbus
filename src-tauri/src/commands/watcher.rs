//! File watcher Tauri commands
//!
//! Provides file system watching commands for automatic directory refresh
//! and real-time file change notifications.

use core_engine::{FileWatcher, FileChangeEvent, WatchConfig, create_directory_watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tauri::{AppHandle, Emitter, State};

/// File watcher state shared across the application
#[derive(Default)]
pub struct WatcherState {
    pub watchers: Arc<RwLock<HashMap<String, Arc<FileWatcher>>>>,
    pub event_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<FileChangeEvent>>>>,
}

/// Watch configuration for Tauri commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfigOptions {
    pub recursive: bool,
    pub debounce_ms: u64,
    pub include_hidden: bool,
    pub file_filters: Vec<String>,
}

impl From<WatchConfigOptions> for WatchConfig {
    fn from(options: WatchConfigOptions) -> Self {
        Self {
            recursive: options.recursive,
            debounce_duration: std::time::Duration::from_millis(options.debounce_ms),
            include_hidden: options.include_hidden,
            file_filters: options.file_filters,
        }
    }
}

/// File change event for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEventData {
    pub path: String,
    pub change_type: String, // "created", "modified", "deleted", "renamed"
    pub old_path: Option<String>, // for rename events
    pub timestamp: String, // ISO 8601 timestamp
}

impl From<FileChangeEvent> for FileChangeEventData {
    fn from(event: FileChangeEvent) -> Self {
        let (change_type, old_path) = match event.change_type {
            core_engine::FileChangeType::Created => ("created".to_string(), None),
            core_engine::FileChangeType::Modified => ("modified".to_string(), None),
            core_engine::FileChangeType::Deleted => ("deleted".to_string(), None),
            core_engine::FileChangeType::Renamed { old_path } => {
                ("renamed".to_string(), Some(old_path.to_string_lossy().to_string()))
            }
        };
        
        Self {
            path: event.path.to_string_lossy().to_string(),
            change_type,
            old_path,
            timestamp: chrono::DateTime::<chrono::Utc>::from(
                std::time::SystemTime::now()
            ).to_rfc3339(),
        }
    }
}

/// Start watching a directory for file changes
#[tauri::command]
pub async fn start_directory_watch(
    path: String,
    options: Option<WatchConfigOptions>,
    app_handle: AppHandle,
    watcher_state: State<'_, WatcherState>,
) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    let config: WatchConfig = options.unwrap_or(WatchConfigOptions {
        recursive: false,
        debounce_ms: 100,
        include_hidden: false,
        file_filters: vec![],
    }).into();
    
    // Create event channel for this watch
    let (tx, mut rx) = mpsc::unbounded_channel::<FileChangeEvent>();
    
    // Create the watcher
    let (watcher, watch_id) = create_directory_watcher(&path_buf, tx)
        .await
        .map_err(|e| format!("Failed to create directory watcher: {}", e))?;
    
    let watcher = Arc::new(watcher);
    let watch_id_clone = watch_id.clone();
    let app_handle_clone = app_handle.clone();
    
    // Store the watcher
    {
        let mut watchers = watcher_state.watchers.write().await;
        watchers.insert(watch_id.clone(), watcher);
    }
    
    // Spawn task to handle file change events
    let watch_id_for_task = watch_id.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let event_data = FileChangeEventData::from(event);
            
            // Emit event to frontend
            if let Err(e) = app_handle_clone.emit(&format!("file-change-{}", watch_id_for_task), &event_data) {
                eprintln!("Failed to emit file change event: {}", e);
            }
            
            // Also emit to general file-change channel
            if let Err(e) = app_handle_clone.emit("file-change", &event_data) {
                eprintln!("Failed to emit file change event to general channel: {}", e);
            }
        }
    });
    
    Ok(watch_id)
}

/// Stop watching a directory
#[tauri::command]
pub async fn stop_directory_watch(
    watch_id: String,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    let mut watchers = watcher_state.watchers.write().await;
    
    if let Some(watcher) = watchers.remove(&watch_id) {
        // The watcher will be automatically cleaned up when dropped
        drop(watcher);
        Ok(())
    } else {
        Err(format!("Watch with ID '{}' not found", watch_id))
    }
}

/// Get list of active directory watches
#[tauri::command]
pub async fn list_directory_watches(
    watcher_state: State<'_, WatcherState>,
) -> Result<Vec<(String, String)>, String> {
    let watchers = watcher_state.watchers.read().await;
    let mut watch_list = Vec::new();
    
    for (watch_id, watcher) in watchers.iter() {
        let watched_paths = watcher.get_watched_paths().await;
        for (_, path) in watched_paths {
            watch_list.push((watch_id.clone(), path.to_string_lossy().to_string()));
        }
    }
    
    Ok(watch_list)
}

/// Performance monitoring command to get cache statistics
#[tauri::command]
pub async fn get_performance_stats(
    filesystem_state: State<'_, super::files::FileSystemState>,
) -> Result<Option<core_engine::PerformanceStats>, String> {
    let fs = filesystem_state.filesystem.read().await;
    Ok(fs.get_performance_stats().await)
}

/// Get cache hit ratio for performance monitoring
#[tauri::command]
pub async fn get_cache_hit_ratio(
    filesystem_state: State<'_, super::files::FileSystemState>,
) -> Result<f64, String> {
    let fs = filesystem_state.filesystem.read().await;
    Ok(fs.get_cache_hit_ratio().await)
}

/// Invalidate cache for a specific directory path
#[tauri::command]
pub async fn invalidate_directory_cache(
    path: String,
    filesystem_state: State<'_, super::files::FileSystemState>,
) -> Result<(), String> {
    let fs = filesystem_state.filesystem.read().await;
    let path_buf = PathBuf::from(path);
    fs.invalidate_cache(&path_buf);
    Ok(())
}

/// Watch configuration validation helper
fn validate_watch_config(config: &WatchConfigOptions) -> Result<(), String> {
    if config.debounce_ms < 10 {
        return Err("Debounce time must be at least 10ms".to_string());
    }
    
    if config.debounce_ms > 10000 {
        return Err("Debounce time must be less than 10 seconds".to_string());
    }
    
    for filter in &config.file_filters {
        if filter.is_empty() {
            return Err("File filters cannot be empty strings".to_string());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_watch_config_conversion() {
        let options = WatchConfigOptions {
            recursive: true,
            debounce_ms: 250,
            include_hidden: true,
            file_filters: vec!["*.txt".to_string(), "*.rs".to_string()],
        };
        
        let config: WatchConfig = options.into();
        assert!(config.recursive);
        assert_eq!(config.debounce_duration.as_millis(), 250);
        assert!(config.include_hidden);
        assert_eq!(config.file_filters.len(), 2);
    }
    
    #[test]
    fn test_watch_config_validation() {
        let valid_config = WatchConfigOptions {
            recursive: false,
            debounce_ms: 100,
            include_hidden: false,
            file_filters: vec!["*.txt".to_string()],
        };
        assert!(validate_watch_config(&valid_config).is_ok());
        
        let invalid_debounce = WatchConfigOptions {
            recursive: false,
            debounce_ms: 5, // Too low
            include_hidden: false,
            file_filters: vec![],
        };
        assert!(validate_watch_config(&invalid_debounce).is_err());
        
        let empty_filter = WatchConfigOptions {
            recursive: false,
            debounce_ms: 100,
            include_hidden: false,
            file_filters: vec!["".to_string()], // Empty filter
        };
        assert!(validate_watch_config(&empty_filter).is_err());
    }
    
    #[test]
    fn test_file_change_event_conversion() {
        use std::path::PathBuf;
        
        let event = FileChangeEvent {
            path: PathBuf::from("/test/file.txt"),
            change_type: core_engine::FileChangeType::Created,
            timestamp: std::time::Instant::now(),
        };
        
        let event_data: FileChangeEventData = event.into();
        assert_eq!(event_data.change_type, "created");
        assert!(event_data.old_path.is_none());
        assert!(!event_data.timestamp.is_empty());
    }
    
    #[tokio::test]
    async fn test_watcher_state_operations() {
        let state = WatcherState::default();
        
        // Initially empty
        let watchers = state.watchers.read().await;
        assert!(watchers.is_empty());
        drop(watchers);
        
        // Can add and retrieve watchers
        let temp_dir = TempDir::new().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let (watcher, watch_id) = create_directory_watcher(temp_dir.path(), tx).await.unwrap();
        
        {
            let mut watchers = state.watchers.write().await;
            watchers.insert(watch_id.clone(), Arc::new(watcher));
        }
        
        let watchers = state.watchers.read().await;
        assert!(watchers.contains_key(&watch_id));
    }
}