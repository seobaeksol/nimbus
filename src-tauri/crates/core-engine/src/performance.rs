//! Performance optimizations for file operations
//!
//! Provides caching, batching, and other performance enhancements
//! for handling large directories efficiently.

use crate::{FileError, FileInfo};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, Semaphore};

/// Cache entry for directory listings
#[derive(Debug, Clone)]
struct CacheEntry {
    files: Vec<FileInfo>,
    timestamp: Instant,
    path_hash: u64,
}

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum number of entries to cache
    pub max_cache_entries: usize,
    /// How long to keep cache entries
    pub cache_ttl: Duration,
    /// Maximum concurrent file operations
    pub max_concurrent_ops: usize,
    /// Enable parallel directory reading
    pub enable_parallel_reads: bool,
    /// Chunk size for batch operations
    pub batch_chunk_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_cache_entries: 100,
            cache_ttl: Duration::from_secs(30),
            max_concurrent_ops: 10,
            enable_parallel_reads: true,
            batch_chunk_size: 50,
        }
    }
}

/// Performance-optimized file operations
pub struct PerformanceOptimizer {
    config: PerformanceConfig,
    directory_cache: DashMap<PathBuf, CacheEntry>,
    operation_semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<PerformanceStats>>,
}

/// Performance statistics
#[derive(Debug, Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PerformanceStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_operations: u64,
    pub total_duration: Duration,
    pub concurrent_operations: u64,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            operation_semaphore: Arc::new(Semaphore::new(config.max_concurrent_ops)),
            directory_cache: DashMap::new(),
            config,
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }
    
    /// Get cached directory listing if available and valid
    pub async fn get_cached_directory(&self, path: &Path) -> Option<Vec<FileInfo>> {
        let path_buf = path.to_path_buf();
        
        if let Some(entry) = self.directory_cache.get(&path_buf) {
            let now = Instant::now();
            if now.duration_since(entry.timestamp) < self.config.cache_ttl {
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                }
                
                return Some(entry.files.clone());
            } else {
                // Entry expired, remove it
                self.directory_cache.remove(&path_buf);
            }
        }
        
        // Update stats for cache miss
        {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
        }
        
        None
    }
    
    /// Cache directory listing
    pub async fn cache_directory(&self, path: &Path, files: Vec<FileInfo>) {
        // Check cache size limit
        if self.directory_cache.len() >= self.config.max_cache_entries {
            self.cleanup_old_cache_entries().await;
        }
        
        let path_hash = self.calculate_path_hash(path);
        let entry = CacheEntry {
            files,
            timestamp: Instant::now(),
            path_hash,
        };
        
        self.directory_cache.insert(path.to_path_buf(), entry);
    }
    
    /// Invalidate cache for a specific path
    pub fn invalidate_cache(&self, path: &Path) {
        self.directory_cache.remove(path);
        
        // Also remove any cached subdirectories
        let path_str = path.to_string_lossy();
        self.directory_cache.retain(|cached_path, _| {
            !cached_path.to_string_lossy().starts_with(&*path_str)
        });
    }
    
    /// Perform an operation with concurrency limiting
    pub async fn execute_with_limits<F, T>(&self, operation: F) -> Result<T, FileError>
    where
        F: std::future::Future<Output = Result<T, FileError>>,
    {
        let _permit = self.operation_semaphore.acquire().await
            .map_err(|e| FileError::Other(format!("Failed to acquire operation permit: {}", e)))?;
        
        let start_time = Instant::now();
        
        // Update concurrent operations count
        {
            let mut stats = self.stats.write().await;
            stats.concurrent_operations += 1;
        }
        
        let result = operation.await;
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.concurrent_operations -= 1;
            stats.total_operations += 1;
            stats.total_duration += start_time.elapsed();
        }
        
        result
    }
    
    /// Batch process multiple paths
    pub async fn batch_process<F, T>(&self, paths: Vec<PathBuf>, mut processor: F) -> Vec<Result<T, FileError>>
    where
        F: FnMut(&Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, FileError>> + Send>> + Send,
        T: Send + 'static,
    {
        let chunk_size = self.config.batch_chunk_size;
        let mut results = Vec::with_capacity(paths.len());
        
        for chunk in paths.chunks(chunk_size) {
            if self.config.enable_parallel_reads {
                // Process chunk in parallel
                let chunk_results = futures::future::join_all(
                    chunk.iter().map(|path| {
                        let processor_result = processor(path);
                        self.execute_with_limits(async move { processor_result.await })
                    })
                ).await;
                
                results.extend(chunk_results.into_iter());
            } else {
                // Process chunk sequentially
                for path in chunk {
                    let result = self.execute_with_limits(processor(path)).await;
                    results.push(result);
                }
            }
        }
        
        results
    }
    
    /// Get performance statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        *self.stats.read().await
    }
    
    /// Reset performance statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = PerformanceStats::default();
    }
    
    /// Calculate cache hit ratio
    pub async fn get_cache_hit_ratio(&self) -> f64 {
        let stats = self.stats.read().await;
        let total_requests = stats.cache_hits + stats.cache_misses;
        
        if total_requests == 0 {
            0.0
        } else {
            stats.cache_hits as f64 / total_requests as f64
        }
    }
    
    /// Clean up old cache entries
    async fn cleanup_old_cache_entries(&self) {
        let now = Instant::now();
        let ttl = self.config.cache_ttl;
        
        self.directory_cache.retain(|_, entry| {
            now.duration_since(entry.timestamp) < ttl
        });
        
        // If we still have too many entries, remove oldest ones
        if self.directory_cache.len() >= self.config.max_cache_entries {
            let mut entries: Vec<_> = self.directory_cache.iter()
                .map(|entry| (entry.key().clone(), entry.value().timestamp))
                .collect();
            
            // Sort by timestamp (oldest first)
            entries.sort_by_key(|(_, timestamp)| *timestamp);
            
            // Remove oldest entries until we're under the limit
            let entries_to_remove = entries.len() - (self.config.max_cache_entries * 3 / 4); // Remove 25% of limit
            for (path, _) in entries.into_iter().take(entries_to_remove) {
                self.directory_cache.remove(&path);
            }
        }
    }
    
    /// Calculate hash for path (simple implementation)
    fn calculate_path_hash(&self, path: &Path) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        hasher.finish()
    }
}

/// Optimized directory reader with chunked loading
pub struct ChunkedDirectoryReader {
    optimizer: Arc<PerformanceOptimizer>,
    chunk_size: usize,
}

impl ChunkedDirectoryReader {
    pub fn new(optimizer: Arc<PerformanceOptimizer>, chunk_size: usize) -> Self {
        Self {
            optimizer,
            chunk_size,
        }
    }
    
    /// Read directory in chunks for better performance with large directories
    pub async fn read_directory_chunked(
        &self,
        path: &Path,
    ) -> Result<Vec<Vec<FileInfo>>, FileError> {
        // First check cache
        if let Some(cached_files) = self.optimizer.get_cached_directory(path).await {
            return Ok(vec![cached_files]); // Return as single chunk
        }
        
        // Read directory entries
        let mut dir_entries = tokio::fs::read_dir(path).await
            .map_err(|e| FileError::Io(e))?;
        
        let mut all_entries = Vec::new();
        while let Some(entry) = dir_entries.next_entry().await
            .map_err(|e| FileError::Io(e))? {
            all_entries.push(entry);
        }
        
        // Process entries in chunks
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        
        for entry in all_entries {
            if let Ok(metadata) = entry.metadata().await {
                // Convert to FileInfo (simplified)
                if let Some(file_name) = entry.file_name().to_str() {
                    let file_info = FileInfo {
                        name: file_name.to_string(),
                        path: entry.path().to_string_lossy().to_string(),
                        size: if metadata.is_file() { metadata.len() } else { 0 },
                        size_formatted: crate::utils::format_file_size(
                            if metadata.is_file() { metadata.len() } else { 0 }
                        ),
                        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        created: metadata.created().ok(),
                        accessed: metadata.accessed().ok(),
                        file_type: if metadata.is_file() { 
                            crate::FileType::File 
                        } else if metadata.is_dir() { 
                            crate::FileType::Directory 
                        } else { 
                            crate::FileType::Symlink 
                        },
                        extension: crate::utils::get_extension(&entry.path()),
                        permissions: crate::FilePermissions {
                            read: !metadata.permissions().readonly(),
                            write: !metadata.permissions().readonly(),
                            execute: false, // Simplified
                        },
                        is_hidden: crate::utils::is_hidden(file_name),
                        is_readonly: metadata.permissions().readonly(),
                    };
                    
                    current_chunk.push(file_info);
                    
                    if current_chunk.len() >= self.chunk_size {
                        chunks.push(current_chunk);
                        current_chunk = Vec::new();
                    }
                }
            }
        }
        
        // Add remaining items
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        // Cache the complete directory listing
        if !chunks.is_empty() {
            let all_files: Vec<FileInfo> = chunks.iter().flatten().cloned().collect();
            self.optimizer.cache_directory(path, all_files).await;
        }
        
        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let stats = optimizer.get_stats().await;
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }
    
    #[tokio::test]
    async fn test_directory_caching() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let temp_dir = TempDir::new().unwrap();
        let test_files = vec![
            FileInfo {
                name: "test1.txt".to_string(),
                path: temp_dir.path().join("test1.txt").to_string_lossy().to_string(),
                size: 100,
                size_formatted: "100 B".to_string(),
                modified: SystemTime::now(),
                created: Some(SystemTime::now()),
                accessed: Some(SystemTime::now()),
                file_type: crate::FileType::File,
                extension: Some("txt".to_string()),
                permissions: crate::FilePermissions {
                    read: true,
                    write: true,
                    execute: false,
                },
                is_hidden: false,
                is_readonly: false,
            }
        ];
        
        // Cache the files
        optimizer.cache_directory(temp_dir.path(), test_files.clone()).await;
        
        // Retrieve from cache
        let cached_files = optimizer.get_cached_directory(temp_dir.path()).await;
        assert!(cached_files.is_some());
        assert_eq!(cached_files.unwrap().len(), 1);
        
        let stats = optimizer.get_stats().await;
        assert_eq!(stats.cache_hits, 1);
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let temp_dir = TempDir::new().unwrap();
        let test_files = vec![];
        
        // Cache the files
        optimizer.cache_directory(temp_dir.path(), test_files).await;
        
        // Verify cached
        assert!(optimizer.get_cached_directory(temp_dir.path()).await.is_some());
        
        // Invalidate cache
        optimizer.invalidate_cache(temp_dir.path());
        
        // Verify cache is empty
        assert!(optimizer.get_cached_directory(temp_dir.path()).await.is_none());
    }
    
    #[tokio::test]
    async fn test_concurrent_operations() {
        let config = PerformanceConfig {
            max_concurrent_ops: 2,
            ..Default::default()
        };
        let optimizer = PerformanceOptimizer::new(config);
        
        let results = futures::future::join_all(vec![
            optimizer.execute_with_limits(async { Ok(1) }),
            optimizer.execute_with_limits(async { Ok(2) }),
            optimizer.execute_with_limits(async { Ok(3) }),
        ]).await;
        
        assert!(results.iter().all(|r| r.is_ok()));
        
        let stats = optimizer.get_stats().await;
        assert_eq!(stats.total_operations, 3);
    }
}