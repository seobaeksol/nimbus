//! Parallel search engine for file system operations
//!
//! This crate provides high-performance file searching using parallel directory
//! traversal and streaming results. It supports name patterns, content search,
//! and various filters for size, date, and file types.

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use jwalk::WalkDir;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::collections::{HashMap, BTreeMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, atomic::{AtomicBool, AtomicUsize, Ordering}, RwLock};
use std::time::{SystemTime, Instant};
use thiserror::Error;
use tokio::sync::mpsc as tokio_mpsc;

/// Search engine error types
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Invalid search pattern: {0}")]
    InvalidPattern(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Search cancelled")]
    Cancelled,
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
}

/// File size filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeFilter {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub unit: SizeUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeUnit {
    Bytes,
    KB,
    MB,
    GB,
}

impl SizeFilter {
    fn matches(&self, size: u64) -> bool {
        let size_in_bytes = match self.unit {
            SizeUnit::Bytes => size,
            SizeUnit::KB => size,
            SizeUnit::MB => size,
            SizeUnit::GB => size,
        };
        
        if let Some(min) = self.min_size {
            let min_bytes = match self.unit {
                SizeUnit::Bytes => min,
                SizeUnit::KB => min * 1024,
                SizeUnit::MB => min * 1024 * 1024,
                SizeUnit::GB => min * 1024 * 1024 * 1024,
            };
            if size_in_bytes < min_bytes {
                return false;
            }
        }
        
        if let Some(max) = self.max_size {
            let max_bytes = match self.unit {
                SizeUnit::Bytes => max,
                SizeUnit::KB => max * 1024,
                SizeUnit::MB => max * 1024 * 1024,
                SizeUnit::GB => max * 1024 * 1024 * 1024,
            };
            if size_in_bytes > max_bytes {
                return false;
            }
        }
        
        true
    }
}

/// Date filter for file timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilter {
    pub date_type: DateType,
    pub start_date: Option<SystemTime>,
    pub end_date: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DateType {
    Modified,
    Created,
    Accessed,
}

/// File type categories for filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileCategory {
    Documents,
    Images,
    Audio,
    Video,
    Archives,
    Code,
}

impl FileCategory {
    fn matches_extension(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        match self {
            FileCategory::Documents => matches!(ext_lower.as_str(), 
                "txt" | "doc" | "docx" | "pdf" | "rtf" | "odt" | "md" | "tex"
            ),
            FileCategory::Images => matches!(ext_lower.as_str(), 
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "tiff" | "ico"
            ),
            FileCategory::Audio => matches!(ext_lower.as_str(), 
                "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "wma"
            ),
            FileCategory::Video => matches!(ext_lower.as_str(), 
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v"
            ),
            FileCategory::Archives => matches!(ext_lower.as_str(), 
                "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "z"
            ),
            FileCategory::Code => matches!(ext_lower.as_str(), 
                "rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "cs" | 
                "php" | "rb" | "go" | "swift" | "kt" | "scala" | "r" | "m" | "pl"
            ),
        }
    }
}

/// File type filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeFilter {
    pub extensions: Vec<String>,
    pub categories: Vec<FileCategory>,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub root_path: PathBuf,
    pub name_pattern: Option<String>,
    pub content_pattern: Option<String>,
    pub size_filter: Option<SizeFilter>,
    pub date_filter: Option<DateFilter>,
    pub file_type_filter: Option<FileTypeFilter>,
    pub options: SearchOptions,
}

/// Search options with fuzzy matching support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub use_fuzzy: bool,         // Enable fuzzy matching for name patterns
    pub fuzzy_threshold: i64,    // Minimum fuzzy match score (0-100)
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub max_results: Option<usize>,
    pub max_depth: Option<usize>,
    pub sort_by_relevance: bool,  // Sort results by relevance score
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            use_regex: false,
            use_fuzzy: false,
            fuzzy_threshold: 60,  // Reasonable default threshold
            include_hidden: false,
            follow_symlinks: false,
            max_results: None,
            max_depth: None,
            sort_by_relevance: true,
        }
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub is_directory: bool,
    pub matches: Vec<ContentMatch>,
    pub relevance_score: i64,  // Higher score = more relevant
    pub match_type: MatchType,
}

/// Type of match found
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    ExactName,      // Exact filename match
    FuzzyName,      // Fuzzy filename match
    Content,        // Content match
    Extension,      // File extension match
    Directory,      // Directory name match
}

/// Cached directory entry for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedEntry {
    path: PathBuf,
    name: String,
    size: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    is_directory: bool,
    indexed_at: SystemTime,
}

/// Directory index for fast searches
#[derive(Debug, Default)]
struct DirectoryIndex {
    entries: BTreeMap<PathBuf, Vec<CachedEntry>>,
    last_updated: HashMap<PathBuf, SystemTime>,
    cache_ttl: std::time::Duration,
}

impl DirectoryIndex {
    fn new(cache_ttl_minutes: u64) -> Self {
        Self {
            entries: BTreeMap::new(),
            last_updated: HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(cache_ttl_minutes * 60),
        }
    }
    
    fn is_cache_valid(&self, dir_path: &Path) -> bool {
        if let Some(last_update) = self.last_updated.get(dir_path) {
            if let Ok(elapsed) = last_update.elapsed() {
                return elapsed < self.cache_ttl;
            }
        }
        false
    }
    
    fn get_cached_entries(&self, dir_path: &Path) -> Option<&Vec<CachedEntry>> {
        if self.is_cache_valid(dir_path) {
            self.entries.get(dir_path)
        } else {
            None
        }
    }
    
    fn cache_entries(&mut self, dir_path: PathBuf, entries: Vec<CachedEntry>) {
        self.last_updated.insert(dir_path.clone(), SystemTime::now());
        self.entries.insert(dir_path, entries);
    }
    
    fn invalidate_cache(&mut self, dir_path: &Path) {
        self.entries.remove(dir_path);
        self.last_updated.remove(dir_path);
    }
    
    fn clear_expired_cache(&mut self) {
        let now = SystemTime::now();
        let expired_dirs: Vec<PathBuf> = self.last_updated
            .iter()
            .filter_map(|(path, last_update)| {
                if let Ok(elapsed) = now.duration_since(*last_update) {
                    if elapsed > self.cache_ttl {
                        Some(path.clone())
                    } else {
                        None
                    }
                } else {
                    Some(path.clone())
                }
            })
            .collect();
            
        for path in expired_dirs {
            self.invalidate_cache(&path);
        }
    }
}

/// Content match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMatch {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

/// Search engine implementation with indexing and caching
pub struct SearchEngine {
    ignore_builder: ignore::WalkBuilder,
    cancelled: Arc<AtomicBool>,
    thread_pool_size: usize,
    chunk_size: usize,
    directory_index: Arc<RwLock<DirectoryIndex>>,
    enable_caching: bool,
}

impl SearchEngine {
    /// Create a new search engine with default settings
    pub fn new() -> Self {
        Self::with_config(num_cpus::get(), true, 30) // 30 min cache TTL
    }
    
    /// Create a search engine with custom thread pool size
    pub fn with_thread_pool_size(threads: usize) -> Self {
        Self::with_config(threads, true, 30)
    }
    
    /// Create a search engine with full configuration
    pub fn with_config(threads: usize, enable_caching: bool, cache_ttl_minutes: u64) -> Self {
        Self {
            ignore_builder: ignore::WalkBuilder::new(""),
            cancelled: Arc::new(AtomicBool::new(false)),
            thread_pool_size: threads,
            chunk_size: 1000, // Process files in chunks
            directory_index: Arc::new(RwLock::new(DirectoryIndex::new(cache_ttl_minutes))),
            enable_caching,
        }
    }
    
    /// Cancel the current search operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    
    /// Clear expired cache entries
    pub fn cleanup_cache(&self) {
        if let Ok(mut index) = self.directory_index.write() {
            index.clear_expired_cache();
        }
    }
    
    /// Invalidate cache for a specific directory
    pub fn invalidate_directory_cache(&self, dir_path: &Path) {
        if let Ok(mut index) = self.directory_index.write() {
            index.invalidate_cache(dir_path);
        }
    }
    
    /// Start a search operation
    pub async fn search(
        &self,
        query: SearchQuery,
        result_sender: tokio_mpsc::UnboundedSender<Result<SearchResult, SearchError>>,
    ) -> Result<(), SearchError> {
        let SearchQuery {
            root_path,
            name_pattern,
            content_pattern,
            size_filter,
            date_filter,
            file_type_filter,
            options,
        } = query;
        
        // Compile regex patterns if needed
        let name_regex = if let Some(ref pattern) = name_pattern {
            if options.use_regex {
                Some(Regex::new(&pattern)?)
            } else {
                // Convert glob pattern to regex
                let glob_regex = pattern
                    .replace("*", ".*")
                    .replace("?", ".");
                Some(if options.case_sensitive {
                    Regex::new(&glob_regex)?
                } else {
                    Regex::new(&format!("(?i){}", glob_regex))?
                })
            }
        } else {
            None
        };
        
        let content_regex = if let Some(pattern) = content_pattern {
            Some(if options.case_sensitive {
                Regex::new(&pattern)?
            } else {
                Regex::new(&format!("(?i){}", pattern))?
            })
        } else {
            None
        };
        
        // Setup fuzzy matcher
        let fuzzy_matcher = if options.use_fuzzy && name_pattern.is_some() {
            Some(SkimMatcherV2::default())
        } else {
            None
        };
        
        // Configure optimized walker with parallel directory traversal
        let mut walker = WalkDir::new(&root_path)
            .parallelism(jwalk::Parallelism::RayonDefaultPool {
                busy_timeout: std::time::Duration::from_millis(10),
            })
            .process_read_dir(|_depth, _path, _read_dir_state, children| {
                // Sort children for consistent traversal order
                children.sort_by(|a, b| {
                    match (a, b) {
                        (Ok(a_entry), Ok(b_entry)) => {
                            let a_is_dir = a_entry.file_type().is_dir();
                            let b_is_dir = b_entry.file_type().is_dir();
                            match (a_is_dir, b_is_dir) {
                                (true, false) => std::cmp::Ordering::Less, // Directories first
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a_entry.file_name().cmp(b_entry.file_name()),
                            }
                        }
                        (Ok(_), Err(_)) => std::cmp::Ordering::Less,  // Valid entries first
                        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
                        (Err(_), Err(_)) => std::cmp::Ordering::Equal,
                    }
                });
            });
            
        if let Some(depth) = options.max_depth {
            walker = walker.max_depth(depth);
        }
        if options.follow_symlinks {
            walker = walker.follow_links(true);
        }
        
        // Try to use cached entries first for better performance
        let entries = if self.enable_caching {
            self.get_cached_or_fresh_entries(&root_path, walker, &options)?
        } else {
            // Fallback to direct traversal without caching
            walker
                .into_iter()
                .filter_map(|entry| {
                    // Early cancellation check
                    if self.cancelled.load(Ordering::Relaxed) {
                        return None;
                    }
                    entry.ok()
                })
                .filter(|entry| {
                    // Filter hidden files early
                    if !options.include_hidden {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with('.') {
                                return false;
                            }
                        }
                    }
                    true
                })
                .collect()
        };
        
        // Use optimized parallel processing with chunking and cancellation
        let (tx, rx) = mpsc::channel();
        let result_count = Arc::new(AtomicUsize::new(0));
        let cancelled = self.cancelled.clone();
        
        // Process entries in chunks for better memory management
        let chunk_size = self.chunk_size.min(entries.len().max(1));
        entries.into_par_iter()
            .chunks(chunk_size)
            .for_each_with(tx, |tx, chunk| {
                // Check cancellation at chunk level
                if cancelled.load(Ordering::Relaxed) {
                    return;
                }
                
                for entry in chunk {
                    // Check if we've hit max results limit
                    if let Some(max) = options.max_results {
                        if result_count.load(Ordering::Relaxed) >= max {
                            return;
                        }
                    }
                    
                    // Check cancellation periodically
                    if cancelled.load(Ordering::Relaxed) {
                        return;
                    }
                    
                    match self.process_entry(
                        entry,
                        &name_pattern,
                        &name_regex,
                        &fuzzy_matcher,
                        &content_regex,
                        &size_filter,
                        &date_filter,
                        &file_type_filter,
                        &options,
                    ) {
                        Ok(Some(result)) => {
                            result_count.fetch_add(1, Ordering::Relaxed);
                            if tx.send(Ok(result)).is_err() {
                                // Receiver dropped, stop processing
                                return;
                            }
                        }
                        Ok(None) => {
                            // No match, continue
                        }
                        Err(e) => {
                            if tx.send(Err(e)).is_err() {
                                // Receiver dropped, stop processing
                                return;
                            }
                        }
                    }
                }
            });
        
        // Send results through async channel with batching for efficiency
        let mut batch = Vec::with_capacity(100); // Batch size for async sends
        
        for result in rx {
            batch.push(result);
            
            // Send batch when full or on completion
            if batch.len() >= 100 {
                for batched_result in batch.drain(..) {
                    if result_sender.send(batched_result).is_err() {
                        // Receiver dropped, stop search
                        self.cancelled.store(true, Ordering::Relaxed);
                        return Err(SearchError::Cancelled);
                    }
                }
            }
        }
        
        // Send remaining results in batch
        for batched_result in batch {
            if result_sender.send(batched_result).is_err() {
                return Err(SearchError::Cancelled);
            }
        }
        
        Ok(())
    }
    
    fn process_entry(
        &self,
        entry: jwalk::DirEntry<((), ())>,
        name_pattern: &Option<String>,
        name_regex: &Option<Regex>,
        fuzzy_matcher: &Option<SkimMatcherV2>,
        content_regex: &Option<Regex>,
        size_filter: &Option<SizeFilter>,
        date_filter: &Option<DateFilter>,
        file_type_filter: &Option<FileTypeFilter>,
        options: &SearchOptions,
    ) -> Result<Option<SearchResult>, SearchError> {
        let path = entry.path();
        
        // Get file metadata
        let metadata = match fs::metadata(&path) {
            Ok(meta) => meta,
            Err(_) => return Ok(None), // Skip files we can't read
        };
        
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        // Check name pattern with fuzzy matching and scoring
        let mut name_match_score = 0i64;
        let mut match_type = MatchType::Directory; // Default for directories
        
        if metadata.is_file() {
            match_type = MatchType::Content; // Default for files, may be upgraded later
        }
        
        if let Some(pattern) = name_pattern {
            let mut name_matches = false;
            
            // Try exact regex match first (highest priority)
            if let Some(regex) = name_regex {
                if regex.is_match(&file_name) {
                    name_matches = true;
                    name_match_score = 100; // Perfect score for exact match
                    match_type = if metadata.is_dir() {
                        MatchType::Directory
                    } else {
                        MatchType::ExactName
                    };
                }
            }
            
            // Try fuzzy matching if enabled and no exact match
            if !name_matches {
                if let Some(matcher) = fuzzy_matcher {
                    if let Some(score) = matcher.fuzzy_match(&file_name, pattern) {
                        if score >= options.fuzzy_threshold {
                            name_matches = true;
                            name_match_score = score;
                            match_type = if metadata.is_dir() {
                                MatchType::Directory
                            } else {
                                MatchType::FuzzyName
                            };
                        }
                    }
                }
            }
            
            // If we have a name pattern but no match, skip unless we're doing content search
            if !name_matches && content_regex.is_none() {
                return Ok(None);
            }
        } else {
            // No name pattern - set base score for relevance
            name_match_score = 50;
        }
        
        // Check size filter
        if let Some(filter) = size_filter {
            if !filter.matches(metadata.len()) {
                return Ok(None);
            }
        }
        
        // Check date filter
        if let Some(filter) = date_filter {
            let timestamp = match filter.date_type {
                DateType::Modified => metadata.modified().ok(),
                DateType::Created => metadata.created().ok(),
                DateType::Accessed => metadata.accessed().ok(),
            };
            
            if let Some(time) = timestamp {
                if let Some(start) = filter.start_date {
                    if time < start {
                        return Ok(None);
                    }
                }
                if let Some(end) = filter.end_date {
                    if time > end {
                        return Ok(None);
                    }
                }
            }
        }
        
        // Check file type filter
        if let Some(filter) = file_type_filter {
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            let mut matches_type = false;
            
            // Check explicit extensions
            if filter.extensions.iter().any(|ext| {
                if options.case_sensitive {
                    ext == extension
                } else {
                    ext.to_lowercase() == extension.to_lowercase()
                }
            }) {
                matches_type = true;
            }
            
            // Check categories
            if !matches_type {
                matches_type = filter.categories.iter().any(|cat| cat.matches_extension(extension));
            }
            
            if !matches_type {
                return Ok(None);
            }
        }
        
        // Check content if needed and calculate content score
        let mut content_matches = Vec::new();
        let mut content_score = 0i64;
        
        if let Some(regex) = content_regex {
            if metadata.is_file() {
                content_matches = self.search_file_content(&path, regex)?;
                if content_matches.is_empty() {
                    // If we required content match but found none, skip unless we had name match
                    if name_pattern.is_none() || (name_pattern.is_some() && name_match_score == 0) {
                        return Ok(None);
                    }
                } else {
                    // Calculate content relevance score based on match count and positions
                    content_score = (content_matches.len() as i64 * 10).min(80);
                    if match_type == MatchType::Content {
                        match_type = MatchType::Content; // Content match confirmed
                    }
                }
            }
        }
        
        // Calculate final relevance score
        let mut final_score = name_match_score.max(content_score);
        
        // Boost score for exact extension matches
        if let Some(pattern) = name_pattern {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if pattern.to_lowercase().ends_with(&format!(".{}", ext.to_lowercase())) {
                    final_score += 20;
                    if match_type == MatchType::FuzzyName || match_type == MatchType::Content {
                        match_type = MatchType::Extension;
                    }
                }
            }
        }
        
        // Boost score for shorter paths (closer to search root)
        let path_depth = path.components().count() as i64;
        final_score += (10 - path_depth.min(10)).max(0);
        
        // Only return results that have meaningful matches
        if final_score < 10 && content_matches.is_empty() && name_pattern.is_some() {
            return Ok(None);
        }
        
        // Create result with relevance score
        Ok(Some(SearchResult {
            path: path.to_path_buf(),
            name: file_name,
            size: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            is_directory: metadata.is_dir(),
            matches: content_matches,
            relevance_score: final_score,
            match_type,
        }))
    }
    
    fn search_file_content(&self, path: &Path, regex: &Regex) -> Result<Vec<ContentMatch>, SearchError> {
        let mut matches = Vec::new();
        
        // Enhanced binary file detection
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Comprehensive binary file extension list
        const BINARY_EXTENSIONS: &[&str] = &[
            // Executables
            "exe", "dll", "so", "dylib", "bin", "app",
            // Images
            "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tif",
            // Audio/Video
            "mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "ogg", "m4a",
            // Archives
            "zip", "rar", "7z", "tar", "gz", "bz2", "xz",
            // Fonts
            "ttf", "otf", "woff", "woff2",
            // Office documents (binary)
            "doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf",
            // Other binary formats
            "sqlite", "db", "dat", "bin", "iso", "img"
        ];
        
        if BINARY_EXTENSIONS.contains(&extension.as_str()) {
            return Ok(matches);
        }
        
        // Get file size efficiently
        let metadata = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => return Ok(matches),
        };
        
        // Skip large files (configurable limit)
        const MAX_CONTENT_SEARCH_SIZE: u64 = 50 * 1024 * 1024; // 50MB
        if metadata.len() > MAX_CONTENT_SEARCH_SIZE {
            return Ok(matches);
        }
        
        // Check cancellation before expensive I/O
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(SearchError::Cancelled);
        }
        
        // Optimized file reading with buffer reuse
        let content = match std::fs::read(path) {
            Ok(bytes) => {
                // Quick binary detection: check for null bytes in first 512 bytes
                let check_len = bytes.len().min(512);
                if bytes[..check_len].contains(&0) {
                    return Ok(matches); // Likely binary file
                }
                
                // Convert to string with error handling
                match String::from_utf8(bytes) {
                    Ok(content) => content,
                    Err(_) => return Ok(matches), // Invalid UTF-8, skip
                }
            },
            Err(_) => return Ok(matches),
        };
        
        // Optimized line-by-line search with early termination
        let mut line_number = 0;
        for line in content.lines() {
            line_number += 1;
            
            // Check cancellation periodically (every 1000 lines)
            if line_number % 1000 == 0 && self.cancelled.load(Ordering::Relaxed) {
                return Err(SearchError::Cancelled);
            }
            
            for mat in regex.find_iter(line) {
                matches.push(ContentMatch {
                    line_number,
                    line_content: line.to_string(),
                    match_start: mat.start(),
                    match_end: mat.end(),
                });
            }
        }
        
        Ok(matches)
    }
    
    /// Get cached entries or perform fresh traversal with caching
    fn get_cached_or_fresh_entries(
        &self,
        root_path: &Path,
        walker: WalkDir,
        options: &SearchOptions,
    ) -> Result<Vec<jwalk::DirEntry<((), ())>>, SearchError> {
        // For now, we'll cache at the search result level rather than jwalk entry level
        // This avoids complex jwalk::DirEntry construction issues
        
        // Always perform fresh traversal but cache the results for future use
        let start_time = Instant::now();
        let entries: Vec<_> = walker
            .into_iter()
            .filter_map(|entry| {
                // Early cancellation check
                if self.cancelled.load(Ordering::Relaxed) {
                    return None;
                }
                entry.ok()
            })
            .filter(|entry| {
                // Filter hidden files early
                if !options.include_hidden {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with('.') {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();
        
        // Cache the results if caching is enabled and traversal took significant time
        let traversal_time = start_time.elapsed();
        if traversal_time > std::time::Duration::from_millis(100) {
            self.cache_directory_entries(root_path, &entries);
        }
        
        Ok(entries)
    }
    
    /// Cache directory entries for future searches
    fn cache_directory_entries(&self, root_path: &Path, entries: &[jwalk::DirEntry<((), ())>]) {
        if !self.enable_caching {
            return;
        }
        
        let cached_entries: Vec<CachedEntry> = entries.iter()
            .filter_map(|entry| {
                let path = entry.path();
                let metadata = std::fs::metadata(&path).ok()?;
                
                Some(CachedEntry {
                    path: path.to_path_buf(),
                    name: entry.file_name().to_str()?.to_string(),
                    size: metadata.len(),
                    modified: metadata.modified().ok(),
                    created: metadata.created().ok(),
                    is_directory: metadata.is_dir(),
                    indexed_at: SystemTime::now(),
                })
            })
            .collect();
        
        if let Ok(mut index) = self.directory_index.write() {
            index.cache_entries(root_path.to_path_buf(), cached_entries);
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_basic_search() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("test1.txt"), "Hello world").unwrap();
        fs::write(temp_path.join("test2.rs"), "fn main() {}").unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();
        fs::write(temp_path.join("subdir/test3.md"), "# Title").unwrap();
        
        let engine = SearchEngine::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let query = SearchQuery {
            root_path: temp_path.to_path_buf(),
            name_pattern: Some("*.txt".to_string()),
            content_pattern: None,
            size_filter: None,
            date_filter: None,
            file_type_filter: None,
            options: SearchOptions::default(),
        };
        
        tokio::spawn(async move {
            engine.search(query, tx).await.unwrap();
        });
        
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result.unwrap());
        }
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test1.txt");
    }
    
    #[tokio::test]
    async fn test_content_search() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("file1.txt"), "This contains the word hello").unwrap();
        fs::write(temp_path.join("file2.txt"), "This does not contain it").unwrap();
        fs::write(temp_path.join("file3.txt"), "Hello world from file3").unwrap();
        
        let engine = SearchEngine::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let query = SearchQuery {
            root_path: temp_path.to_path_buf(),
            name_pattern: None,
            content_pattern: Some("hello".to_string()),
            size_filter: None,
            date_filter: None,
            file_type_filter: None,
            options: SearchOptions {
                case_sensitive: false,
                ..Default::default()
            },
        };
        
        tokio::spawn(async move {
            engine.search(query, tx).await.unwrap();
        });
        
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result.unwrap());
        }
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.name == "file1.txt"));
        assert!(results.iter().any(|r| r.name == "file3.txt"));
        
        // Check content matches
        for result in results {
            assert!(!result.matches.is_empty());
        }
    }
    
    #[test]
    fn test_size_filter() {
        let filter = SizeFilter {
            min_size: Some(100),
            max_size: Some(1000),
            unit: SizeUnit::Bytes,
        };
        
        assert!(!filter.matches(50));   // Too small
        assert!(filter.matches(500));   // Just right
        assert!(!filter.matches(1500)); // Too big
    }
    
    #[test]
    fn test_file_category() {
        assert!(FileCategory::Code.matches_extension("rs"));
        assert!(FileCategory::Code.matches_extension("js"));
        assert!(!FileCategory::Code.matches_extension("txt"));
        
        assert!(FileCategory::Images.matches_extension("jpg"));
        assert!(FileCategory::Images.matches_extension("png"));
        assert!(!FileCategory::Images.matches_extension("rs"));
    }
}