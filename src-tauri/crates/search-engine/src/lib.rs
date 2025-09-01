//! Parallel search engine for file system operations
//!
//! This crate provides high-performance file searching using parallel directory
//! traversal and streaming results. It supports name patterns, content search,
//! and various filters for size, date, and file types.

use jwalk::WalkDir;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::SystemTime;
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

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub max_results: Option<usize>,
    pub max_depth: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            use_regex: false,
            include_hidden: false,
            follow_symlinks: false,
            max_results: None,
            max_depth: None,
        }
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub is_directory: bool,
    pub matches: Vec<ContentMatch>,
}

/// Content match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMatch {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

/// Search engine implementation
pub struct SearchEngine {
    // Future: use ignore_builder for gitignore support
    #[allow(dead_code)]
    ignore_builder: ignore::WalkBuilder,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self {
            ignore_builder: ignore::WalkBuilder::new(""),
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
        let name_regex = if let Some(pattern) = name_pattern {
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
        
        // Configure walker
        let mut walker = WalkDir::new(&root_path);
        if let Some(depth) = options.max_depth {
            walker = walker.max_depth(depth);
        }
        if options.follow_symlinks {
            walker = walker.follow_links(true);
        }
        
        // Collect entries for parallel processing
        let entries: Vec<_> = walker
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                // Filter hidden files
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
        
        // Use rayon for parallel processing
        let (tx, rx) = mpsc::channel();
        let result_count = std::sync::atomic::AtomicUsize::new(0);
        
        entries.into_par_iter().for_each_with(tx, |tx, entry| {
            // Check if we've hit max results limit
            if let Some(max) = options.max_results {
                if result_count.load(std::sync::atomic::Ordering::Relaxed) >= max {
                    return;
                }
            }
            
            match self.process_entry(
                entry,
                &name_regex,
                &content_regex,
                &size_filter,
                &date_filter,
                &file_type_filter,
                &options,
            ) {
                Ok(Some(result)) => {
                    result_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = tx.send(Ok(result));
                }
                Ok(None) => {
                    // No match, continue
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        });
        
        // Send results through async channel
        for result in rx {
            if result_sender.send(result).is_err() {
                // Receiver dropped, stop search
                return Err(SearchError::Cancelled);
            }
        }
        
        Ok(())
    }
    
    fn process_entry(
        &self,
        entry: jwalk::DirEntry<((), ())>,
        name_regex: &Option<Regex>,
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
        
        // Check name pattern
        if let Some(regex) = name_regex {
            if !regex.is_match(&file_name) {
                return Ok(None);
            }
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
        
        // Check content if needed
        let mut content_matches = Vec::new();
        if let Some(regex) = content_regex {
            if metadata.is_file() {
                content_matches = self.search_file_content(&path, regex)?;
                if content_matches.is_empty() {
                    return Ok(None);
                }
            }
        }
        
        // Create result
        Ok(Some(SearchResult {
            path: path.to_path_buf(),
            name: file_name,
            size: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            is_directory: metadata.is_dir(),
            matches: content_matches,
        }))
    }
    
    fn search_file_content(&self, path: &Path, regex: &Regex) -> Result<Vec<ContentMatch>, SearchError> {
        let mut matches = Vec::new();
        
        // Only search text files (basic heuristic)
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Skip binary file extensions
        if matches!(extension.as_str(), "exe" | "dll" | "so" | "dylib" | "bin" | "jpg" | "png" | "gif" | "mp4" | "mp3" | "zip" | "rar" | "7z") {
            return Ok(matches);
        }
        
        // Limit file size for content search (max 10MB)
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > 10 * 1024 * 1024 {
                return Ok(matches);
            }
        }
        
        // Read file content
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return Ok(matches), // Skip binary files or unreadable files
        };
        
        // Search for matches line by line
        for (line_number, line) in content.lines().enumerate() {
            for mat in regex.find_iter(line) {
                matches.push(ContentMatch {
                    line_number: line_number + 1,
                    line_content: line.to_string(),
                    match_start: mat.start(),
                    match_end: mat.end(),
                });
            }
        }
        
        Ok(matches)
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