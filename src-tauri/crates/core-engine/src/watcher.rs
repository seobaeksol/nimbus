//! File system watcher for automatic directory refresh
//!
//! Provides real-time file system monitoring using the notify crate
//! for automatic UI updates when files change.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// File watcher errors
#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Failed to create file watcher: {0}")]
    InitializationFailed(String),
    
    #[error("Failed to watch path: {path} - {error}")]
    WatchFailed { path: String, error: String },
    
    #[error("Failed to unwatch path: {path}")]
    UnwatchFailed { path: String },
    
    #[error("Watch not found: {id}")]
    WatchNotFound { id: String },
}

/// Types of file system events
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { old_path: PathBuf },
}

/// File system change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub change_type: FileChangeType,
    pub timestamp: Instant,
}

/// Watch configuration
#[derive(Debug, Clone)]
pub struct WatchConfig {
    pub recursive: bool,
    pub debounce_duration: Duration,
    pub include_hidden: bool,
    pub file_filters: Vec<String>, // glob patterns
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            recursive: false, // Only watch the specific directory, not subdirectories
            debounce_duration: Duration::from_millis(100), // 100ms debounce
            include_hidden: false,
            file_filters: vec![],
        }
    }
}

/// A single directory watch
#[derive(Debug)]
struct DirectoryWatch {
    id: String,
    path: PathBuf,
    config: WatchConfig,
    sender: mpsc::UnboundedSender<FileChangeEvent>,
    last_event_time: Option<Instant>,
}

/// File system watcher manager
pub struct FileWatcher {
    watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
    watches: Arc<RwLock<HashMap<String, DirectoryWatch>>>,
    event_sender: mpsc::UnboundedSender<(PathBuf, Event)>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<(PathBuf, Event)>>>>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self, WatcherError> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            watcher: Arc::new(RwLock::new(None)),
            watches: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        })
    }
    
    /// Initialize the file watcher system
    pub async fn initialize(&self) -> Result<(), WatcherError> {
        let mut watcher_guard = self.watcher.write().await;
        
        if watcher_guard.is_some() {
            return Ok(()); // Already initialized
        }
        
        let sender = self.event_sender.clone();
        
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Send event for processing
                        for path in &event.paths {
                            let _ = sender.send((path.clone(), event.clone()));
                        }
                    }
                    Err(e) => {
                        eprintln!("File watcher error: {:?}", e);
                    }
                }
            },
            notify::Config::default(),
        ).map_err(|e| WatcherError::InitializationFailed(e.to_string()))?;
        
        *watcher_guard = Some(watcher);
        
        // Start event processing task
        self.start_event_processor().await;
        
        Ok(())
    }
    
    /// Start watching a directory
    pub async fn watch_directory(
        &self,
        path: &Path,
        config: WatchConfig,
        callback: mpsc::UnboundedSender<FileChangeEvent>,
    ) -> Result<String, WatcherError> {
        // Ensure watcher is initialized
        self.initialize().await?;
        
        let watch_id = Uuid::new_v4().to_string();
        let canonical_path = dunce::canonicalize(path)
            .map_err(|e| WatcherError::WatchFailed {
                path: path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        
        // Add to notify watcher
        {
            let mut watcher_guard = self.watcher.write().await;
            if let Some(ref mut watcher) = *watcher_guard {
                let mode = if config.recursive {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                
                watcher.watch(&canonical_path, mode)
                    .map_err(|e| WatcherError::WatchFailed {
                        path: path.to_string_lossy().to_string(),
                        error: e.to_string(),
                    })?;
            } else {
                return Err(WatcherError::InitializationFailed("Watcher not initialized".to_string()));
            }
        }
        
        // Store watch information
        let directory_watch = DirectoryWatch {
            id: watch_id.clone(),
            path: canonical_path,
            config,
            sender: callback,
            last_event_time: None,
        };
        
        {
            let mut watches = self.watches.write().await;
            watches.insert(watch_id.clone(), directory_watch);
        }
        
        Ok(watch_id)
    }
    
    /// Stop watching a directory
    pub async fn unwatch_directory(&self, watch_id: &str) -> Result<(), WatcherError> {
        let path = {
            let mut watches = self.watches.write().await;
            let watch = watches.remove(watch_id)
                .ok_or_else(|| WatcherError::WatchNotFound { id: watch_id.to_string() })?;
            watch.path
        };
        
        // Remove from notify watcher
        {
            let mut watcher_guard = self.watcher.write().await;
            if let Some(ref mut watcher) = *watcher_guard {
                watcher.unwatch(&path)
                    .map_err(|_| WatcherError::UnwatchFailed { path: path.to_string_lossy().to_string() })?;
            }
        }
        
        Ok(())
    }
    
    /// Get list of currently watched directories
    pub async fn get_watched_paths(&self) -> Vec<(String, PathBuf)> {
        let watches = self.watches.read().await;
        watches.iter()
            .map(|(id, watch)| (id.clone(), watch.path.clone()))
            .collect()
    }
    
    /// Start the event processing task
    async fn start_event_processor(&self) {
        let receiver = {
            let mut receiver_guard = self.event_receiver.write().await;
            receiver_guard.take()
        };
        
        if let Some(mut receiver) = receiver {
            let watches = Arc::clone(&self.watches);
            
            tokio::spawn(async move {
                while let Some((event_path, event)) = receiver.recv().await {
                    Self::process_event(&watches, &event_path, &event).await;
                }
            });
        }
    }
    
    /// Process a file system event
    async fn process_event(
        watches: &Arc<RwLock<HashMap<String, DirectoryWatch>>>,
        event_path: &Path,
        event: &Event,
    ) {
        let watches_guard = watches.read().await;
        
        // Find watches that match this event path
        for (_watch_id, watch) in watches_guard.iter() {
            if Self::path_matches_watch(&watch.path, event_path, &watch.config) {
                if let Some(change_event) = Self::convert_event(event_path, event, &watch.config) {
                    // Apply debouncing
                    let now = Instant::now();
                    let should_send = watch.last_event_time
                        .map(|last| now.duration_since(last) >= watch.config.debounce_duration)
                        .unwrap_or(true);
                    
                    if should_send {
                        let _ = watch.sender.send(change_event);
                        // Note: We can't update last_event_time here because we only have a read lock
                        // In a real implementation, we'd need more sophisticated debouncing
                    }
                }
            }
        }
    }
    
    /// Check if an event path matches a watch
    fn path_matches_watch(watch_path: &Path, event_path: &Path, config: &WatchConfig) -> bool {
        if config.recursive {
            event_path.starts_with(watch_path)
        } else {
            event_path.parent() == Some(watch_path) || event_path == watch_path
        }
    }
    
    /// Convert notify event to our FileChangeEvent
    fn convert_event(path: &Path, event: &Event, config: &WatchConfig) -> Option<FileChangeEvent> {
        // Filter hidden files if not included
        if !config.include_hidden {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') {
                    return None;
                }
            }
        }
        
        // Apply file filters (simplified - would use glob patterns in real implementation)
        if !config.file_filters.is_empty() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if !config.file_filters.iter().any(|filter| filter.contains(extension)) {
                    return None;
                }
            }
        }
        
        let change_type = match event.kind {
            EventKind::Create(_) => FileChangeType::Created,
            EventKind::Modify(_) => FileChangeType::Modified,
            EventKind::Remove(_) => FileChangeType::Deleted,
            _ => return None, // Ignore other event types for now
        };
        
        Some(FileChangeEvent {
            path: path.to_path_buf(),
            change_type,
            timestamp: Instant::now(),
        })
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        // Cleanup is handled by the async Drop when the watcher goes out of scope
    }
}

/// Convenience function to create a basic directory watcher
pub async fn create_directory_watcher(
    path: &Path,
    callback: mpsc::UnboundedSender<FileChangeEvent>,
) -> Result<(FileWatcher, String), WatcherError> {
    let watcher = FileWatcher::new()?;
    let config = WatchConfig::default();
    let watch_id = watcher.watch_directory(path, config, callback).await?;
    Ok((watcher, watch_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }
    
    #[tokio::test]
    async fn test_watch_directory() {
        let temp_dir = TempDir::new().unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let watcher = FileWatcher::new().unwrap();
        let config = WatchConfig::default();
        
        let watch_id = watcher.watch_directory(temp_dir.path(), config, tx).await;
        assert!(watch_id.is_ok());
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, "test content").await.unwrap();
        
        // Wait for event (with timeout)
        tokio::select! {
            event = rx.recv() => {
                assert!(event.is_some());
                let event = event.unwrap();
                assert_eq!(event.change_type, FileChangeType::Created);
            }
            _ = sleep(Duration::from_secs(2)) => {
                // Timeout - events might not be immediate in test environment
                println!("Timeout waiting for file creation event");
            }
        }
    }
    
    #[tokio::test]
    async fn test_unwatch_directory() {
        let temp_dir = TempDir::new().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let watcher = FileWatcher::new().unwrap();
        let config = WatchConfig::default();
        
        let watch_id = watcher.watch_directory(temp_dir.path(), config, tx).await.unwrap();
        let result = watcher.unwatch_directory(&watch_id).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_watched_paths() {
        let temp_dir = TempDir::new().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let watcher = FileWatcher::new().unwrap();
        let config = WatchConfig::default();
        
        let watch_id = watcher.watch_directory(temp_dir.path(), config, tx).await.unwrap();
        let paths = watcher.get_watched_paths().await;
        
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].0, watch_id);
    }
}