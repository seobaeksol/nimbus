//! Search command handlers

use super::CommandResult;
use nimbus_search::{
    SearchEngine, SearchQuery, SearchResult, SearchOptions,
    SizeFilter, DateFilter, FileTypeFilter, FileCategory, SizeUnit, DateType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use tauri::{command, AppHandle, Emitter};
use tokio::sync::mpsc;

/// Active search operations
static ACTIVE_SEARCHES: OnceLock<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>> = OnceLock::new();

fn active_searches() -> &'static Mutex<HashMap<String, tokio::task::JoinHandle<()>>> {
    ACTIVE_SEARCHES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Search query from frontend (using string timestamps for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryRequest {
    pub root_path: String,
    pub name_pattern: Option<String>,
    pub content_pattern: Option<String>,
    pub size_filter: Option<SizeFilterRequest>,
    pub date_filter: Option<DateFilterRequest>,
    pub file_type_filter: Option<FileTypeFilterRequest>,
    pub options: SearchOptionsRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeFilterRequest {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub unit: String, // "bytes", "kb", "mb", "gb"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilterRequest {
    pub date_type: String, // "modified", "created", "accessed"
    pub start_date: Option<String>, // ISO 8601 string
    pub end_date: Option<String>,   // ISO 8601 string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeFilterRequest {
    pub extensions: Vec<String>,
    pub categories: Vec<String>, // category names as strings
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptionsRequest {
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

/// Search result for frontend (using string timestamps for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultResponse {
    pub search_id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: Option<String>, // ISO 8601 string
    pub created: Option<String>,  // ISO 8601 string
    pub is_directory: bool,
    pub matches: Vec<ContentMatchResponse>,
    pub relevance_score: i64,     // Higher score = more relevant
    pub match_type: String,       // Match type as string for frontend
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMatchResponse {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

impl From<SizeFilterRequest> for SizeFilter {
    fn from(req: SizeFilterRequest) -> Self {
        let unit = match req.unit.to_lowercase().as_str() {
            "kb" => SizeUnit::KB,
            "mb" => SizeUnit::MB,
            "gb" => SizeUnit::GB,
            _ => SizeUnit::Bytes,
        };
        
        SizeFilter {
            min_size: req.min_size,
            max_size: req.max_size,
            unit,
        }
    }
}

impl TryFrom<DateFilterRequest> for DateFilter {
    type Error = String;
    
    fn try_from(req: DateFilterRequest) -> Result<Self, Self::Error> {
        let date_type = match req.date_type.to_lowercase().as_str() {
            "modified" => DateType::Modified,
            "created" => DateType::Created,
            "accessed" => DateType::Accessed,
            _ => return Err("Invalid date type".to_string()),
        };
        
        let parse_date = |date_str: Option<String>| -> Result<Option<SystemTime>, String> {
            match date_str {
                Some(_s) => {
                    // Try to parse ISO 8601 format
                    // For simplicity, use a basic parsing approach
                    // In production, consider using chrono crate
                    Ok(Some(SystemTime::UNIX_EPOCH)) // Placeholder implementation
                },
                None => Ok(None),
            }
        };
        
        let start_date = parse_date(req.start_date)?;
        let end_date = parse_date(req.end_date)?;
        
        Ok(DateFilter {
            date_type,
            start_date,
            end_date,
        })
    }
}

impl From<FileTypeFilterRequest> for FileTypeFilter {
    fn from(req: FileTypeFilterRequest) -> Self {
        let categories: Vec<FileCategory> = req.categories
            .into_iter()
            .filter_map(|cat| match cat.to_lowercase().as_str() {
                "documents" => Some(FileCategory::Documents),
                "images" => Some(FileCategory::Images),
                "audio" => Some(FileCategory::Audio),
                "video" => Some(FileCategory::Video),
                "archives" => Some(FileCategory::Archives),
                "code" => Some(FileCategory::Code),
                _ => None,
            })
            .collect();
        
        FileTypeFilter {
            extensions: req.extensions,
            categories,
        }
    }
}

impl From<SearchOptionsRequest> for SearchOptions {
    fn from(req: SearchOptionsRequest) -> Self {
        SearchOptions {
            case_sensitive: req.case_sensitive,
            use_regex: req.use_regex,
            use_fuzzy: req.use_fuzzy,
            fuzzy_threshold: req.fuzzy_threshold,
            include_hidden: req.include_hidden,
            follow_symlinks: req.follow_symlinks,
            max_results: req.max_results,
            max_depth: req.max_depth,
            sort_by_relevance: req.sort_by_relevance,
        }
    }
}

fn system_time_to_iso_string(time: Option<SystemTime>) -> Option<String> {
    time.and_then(|t| {
        t.duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| format!("{}ms", d.as_millis()))
    })
}

impl From<(String, SearchResult)> for SearchResultResponse {
    fn from((search_id, result): (String, SearchResult)) -> Self {
        let match_type_str = match result.match_type {
            nimbus_search::MatchType::ExactName => "exact_name",
            nimbus_search::MatchType::FuzzyName => "fuzzy_name", 
            nimbus_search::MatchType::Content => "content",
            nimbus_search::MatchType::Extension => "extension",
            nimbus_search::MatchType::Directory => "directory",
        };
        
        SearchResultResponse {
            search_id,
            path: result.path.to_string_lossy().to_string(),
            name: result.name,
            size: result.size,
            modified: system_time_to_iso_string(result.modified),
            created: system_time_to_iso_string(result.created),
            is_directory: result.is_directory,
            matches: result.matches.into_iter().map(|m| ContentMatchResponse {
                line_number: m.line_number,
                line_content: m.line_content,
                match_start: m.match_start,
                match_end: m.match_end,
            }).collect(),
            relevance_score: result.relevance_score,
            match_type: match_type_str.to_string(),
        }
    }
}

/// Start a search operation
#[command]
pub async fn start_search(
    app: AppHandle,
    query: SearchQueryRequest,
) -> CommandResult<String> {
    let search_id = uuid::Uuid::new_v4().to_string();
    
    // Convert request to internal types
    let size_filter = query.size_filter.map(SizeFilter::from);
    let date_filter = if let Some(df) = query.date_filter {
        match DateFilter::try_from(df) {
            Ok(filter) => Some(filter),
            Err(e) => return Err(format!("Invalid date filter: {}", e)),
        }
    } else {
        None
    };
    let file_type_filter = query.file_type_filter.map(FileTypeFilter::from);
    
    let internal_query = SearchQuery {
        root_path: PathBuf::from(query.root_path),
        name_pattern: query.name_pattern,
        content_pattern: query.content_pattern,
        size_filter,
        date_filter,
        file_type_filter,
        options: SearchOptions::from(query.options),
    };
    
    let engine = SearchEngine::new();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Start search in background
    let search_id_clone = search_id.clone();
    let app_clone = app.clone();
    let handle = tokio::spawn(async move {
        // Start the search
        if let Err(e) = engine.search(internal_query, tx).await {
            // Send error event
            let error_event = format!("search-error-{}", search_id_clone);
            let _ = app_clone.emit(&error_event, format!("Search failed: {}", e));
            return;
        }
        
        // Process results
        while let Some(result) = rx.recv().await {
            match result {
                Ok(search_result) => {
                    let response = SearchResultResponse::from((search_id_clone.clone(), search_result));
                    let event_name = format!("search-result-{}", search_id_clone);
                    
                    if app_clone.emit(&event_name, &response).is_err() {
                        // Frontend disconnected, stop search
                        break;
                    }
                },
                Err(e) => {
                    let error_event = format!("search-error-{}", search_id_clone);
                    let _ = app_clone.emit(&error_event, format!("Search error: {}", e));
                }
            }
        }
        
        // Send completion event
        let complete_event = format!("search-complete-{}", search_id_clone);
        let _ = app_clone.emit(&complete_event, ());
        
        // Remove from active searches
        active_searches().lock().unwrap().remove(&search_id_clone);
    });
    
    // Store the handle
    active_searches().lock().unwrap().insert(search_id.clone(), handle);
    
    Ok(search_id)
}

/// Cancel a search operation
#[command]
pub async fn cancel_search(search_id: String) -> CommandResult<()> {
    let mut searches = active_searches().lock().unwrap();
    if let Some(handle) = searches.remove(&search_id) {
        handle.abort();
        Ok(())
    } else {
        Err("Search not found".to_string())
    }
}

/// Get list of active search operations
#[command]
pub async fn get_active_searches() -> CommandResult<Vec<String>> {
    let searches = active_searches().lock().unwrap();
    Ok(searches.keys().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_size_filter_conversion() {
        let req = SizeFilterRequest {
            min_size: Some(100),
            max_size: Some(1000),
            unit: "mb".to_string(),
        };
        
        let filter = SizeFilter::from(req);
        assert_eq!(filter.min_size, Some(100));
        assert_eq!(filter.max_size, Some(1000));
        assert!(matches!(filter.unit, SizeUnit::MB));
    }
    
    #[test]
    fn test_file_type_filter_conversion() {
        let req = FileTypeFilterRequest {
            extensions: vec!["rs".to_string(), "js".to_string()],
            categories: vec!["code".to_string(), "images".to_string()],
        };
        
        let filter = FileTypeFilter::from(req);
        assert_eq!(filter.extensions, vec!["rs", "js"]);
        assert_eq!(filter.categories.len(), 2);
        assert!(filter.categories.contains(&FileCategory::Code));
        assert!(filter.categories.contains(&FileCategory::Images));
    }
    
    #[test]
    fn test_search_options_conversion() {
        let req = SearchOptionsRequest {
            case_sensitive: true,
            use_regex: false,
            include_hidden: true,
            follow_symlinks: false,
            max_results: Some(100),
            max_depth: Some(5),
        };
        
        let options = SearchOptions::from(req);
        assert!(options.case_sensitive);
        assert!(!options.use_regex);
        assert!(options.include_hidden);
        assert!(!options.follow_symlinks);
        assert_eq!(options.max_results, Some(100));
        assert_eq!(options.max_depth, Some(5));
    }
}