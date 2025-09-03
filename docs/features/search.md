# File Search System

## Overview

Nimbus provides a comprehensive file search system with fuzzy matching, content search, and real-time results streaming. The search system combines a powerful Rust backend using parallel directory traversal (`jwalk` + `rayon`) with an intuitive React frontend for seamless user experience.

## Current Implementation Status

The search system is **75% complete** with the following features implemented:
- ✅ Fuzzy search with configurable thresholds (0-100)
- ✅ Content search with match highlighting
- ✅ Relevance scoring and result ranking
- ✅ Real-time search result streaming
- ✅ Redux state management for search operations
- ✅ React hooks for component integration
- ✅ Command pattern for search panel operations
- ✅ TypeScript type safety throughout
- ✅ Directory caching with 30-minute TTL
- ⏳ Advanced filters (size, date, file type) - Backend ready, UI integration pending
- ⏳ Saved searches and search history - Not yet implemented

## Architecture Overview

The search system follows a multi-layered architecture:

```
React UI Components → Redux State → Search Hooks → Search Service → Tauri IPC → Rust Search Engine
     ↓                  ↓              ↓              ↓              ↓              ↓
SearchPanel       searchSlice      useSearch    searchService    IPC Events    SearchEngine
SearchResults     ↓              ↓              ↓              ↓              ↓
SearchInterface   Real-time     Hook methods   Event stream   JSON messages  Parallel search
                  state updates                               via Tauri      jwalk + rayon
```

### Key Components

#### Frontend (TypeScript/React)
- **SearchPanel**: Advanced search form with filters and options
- **SearchResults**: Real-time results display with relevance scoring  
- **SearchInterface**: Integrated search experience combining panel and results
- **Integration SearchPanel**: Main UI integration wrapper for multi-panel layout
- **useSearch Hook**: React hook providing search functionality to components
- **searchSlice**: Redux state management for search operations

#### Backend (Rust)
- **SearchEngine**: Core search logic with fuzzy matching and parallel processing
- **Directory Caching**: LRU cache with 30-minute TTL for directory metadata
- **Content Search**: Streaming content search with match highlighting
- **IPC Events**: Real-time result streaming through Tauri events

## Search Interface

### Integrated Search Panel

The main search interface is accessible through:
- **Ctrl+Shift+F**: Open search panel overlay
- **Command Palette**: "Toggle Search Panel" command
- **Escape**: Close search panel

### Quick Search (In-Directory)

The current implementation provides comprehensive search capabilities:

```typescript
// Core Search Interfaces
export interface SearchQuery {
  rootPath: string;
  namePattern?: string;
  contentPattern?: string;
  sizeFilter?: SizeFilter;
  dateFilter?: DateFilter;
  fileTypeFilter?: FileTypeFilter;
  options: SearchOptions;
}

export interface SearchOptions {
  useFuzzy: boolean;
  fuzzyThreshold: number;        // 0-100, default 60
  sortByRelevance: boolean;
  caseSensitive: boolean;
  useRegex: boolean;
  includeHidden: boolean;
  followSymlinks: boolean;
  maxResults: number;
  maxDepth?: number;
}

export interface SearchResult {
  path: string;
  name: string;
  size: number;
  modified: string;
  fileType: 'file' | 'directory';
  matchType: MatchType;
  relevanceScore: number;
  contentMatches?: ContentMatch[];
}

export type MatchType = 'ExactName' | 'FuzzyName' | 'Content' | 'Extension' | 'Directory';

export interface ContentMatch {
  lineNumber: number;
  lineContent: string;
  matchStart: number;
  matchEnd: number;
}
```

#### Features
- **Incremental Filtering**: Results update as you type
- **Pattern Highlighting**: Visual highlighting of matching text in file names
- **Navigation**: Use arrow keys to navigate through matches
- **Multiple Matches**: Jump between multiple matches in long filenames

#### Activation
- **Ctrl+F**: Open quick search in active pane
- **Type-ahead**: Start typing to activate quick search automatically
- **Escape**: Clear search and return to full directory view

### Advanced Search Dialog

The comprehensive search interface provides access to all search capabilities:

```
┌─ Advanced File Search ────────────────────────────────────────┐
│                                                               │
│ Search Location: [/home/user/documents    ] [Browse...]       │
│                                                               │
│ ┌─ Search Criteria ────────────────────────────────────────┐  │
│ │                                                          │  │
│ │ Filename: [*.txt                    ] □ Case sensitive  │  │
│ │           □ Whole words  □ Regular expression           │  │
│ │                                                          │  │
│ │ Content:  [function main            ] □ Case sensitive  │  │
│ │           □ Whole words  □ Regular expression           │  │
│ │                                                          │  │
│ │ File Size: □ Between [1MB] and [10MB]                   │  │
│ │                                                          │  │
│ │ Date Modified: □ Between [2023-01-01] and [2024-01-01]  │  │
│ │                                                          │  │
│ │ File Types: □ Documents  □ Images  □ Archives  □ Code    │  │
│ │             [Custom extensions: rs,toml,md    ]         │  │
│ │                                                          │  │
│ │ Advanced:   □ Include hidden files                       │  │
│ │             □ Follow symbolic links                      │  │
│ │             □ Search in archives                         │  │
│ │                                                          │  │
│ └──────────────────────────────────────────────────────────┘  │
│                                                               │
│ [Search] [Clear] [Save Query] [Load Query] [Cancel]          │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

## Search Criteria

### Filename Search

#### Pattern Types
```rust
pub enum PatternType {
    Wildcard,    // *.txt, file?.doc
    Regex,       // ^[A-Z]+\.(jpg|png)$
    Substring,   // partial filename matches
    Exact,       // exact filename match
}

pub struct FilenameSearch {
    pattern: String,
    pattern_type: PatternType,
    case_sensitive: bool,
    whole_words: bool,
}
```

#### Wildcard Support
- **`*`**: Match any sequence of characters
- **`?`**: Match any single character  
- **`[abc]`**: Match any character in brackets
- **`[a-z]`**: Match any character in range
- **`[!abc]`**: Match any character not in brackets

#### Examples
- `*.{jpg,png,gif}`: All image files with common extensions
- `IMG_????.JPG`: Camera photos with 4-digit numbers
- `[Dd]ocument*`: Files starting with "Document" or "document"
- `report_[0-9][0-9][0-9][0-9].pdf`: Reports with 4-digit years

### Content Search

High-performance text search within file contents:

```rust
pub struct ContentSearch {
    query: String,
    case_sensitive: bool,
    whole_words: bool,
    regex: bool,
    file_types: Vec<String>,
    max_file_size: Option<u64>,
    encoding: EncodingHint,
}

pub enum EncodingHint {
    Utf8,
    Ascii,
    Latin1,
    Auto,          // Attempt automatic detection
    Binary,        // Search in binary files
}
```

#### Supported File Types
- **Text Files**: .txt, .md, .rst, .log
- **Source Code**: .rs, .js, .py, .c, .cpp, .java, .go
- **Configuration**: .json, .yaml, .toml, .xml, .ini
- **Documents**: .csv (plain text formats only)
- **Custom**: User-defined text file extensions

#### Binary File Handling
- **Automatic Detection**: Skip binary files by default
- **Force Text**: Treat all files as text (useful for mixed content)
- **Binary Search**: Search for byte sequences in binary files
- **Encoding Detection**: Attempt to detect file encoding automatically

### Size Filters

```rust
pub struct SizeFilter {
    min_size: Option<u64>,
    max_size: Option<u64>,
    unit: SizeUnit,
}

pub enum SizeUnit {
    Bytes,
    KB,
    MB,
    GB,
}
```

#### Predefined Size Categories
- **Tiny**: < 1KB (empty files, small text files)
- **Small**: 1KB - 100KB (documents, small images)
- **Medium**: 100KB - 10MB (photos, music files)
- **Large**: 10MB - 1GB (videos, large documents)
- **Huge**: > 1GB (archives, disk images)

### Date Filters

```rust
pub struct DateFilter {
    date_type: DateType,
    range_type: DateRangeType,
    start_date: Option<DateTime<Local>>,
    end_date: Option<DateTime<Local>>,
}

pub enum DateType {
    Modified,
    Created,
    Accessed,
}

pub enum DateRangeType {
    Absolute,      // Specific date range
    Relative,      // Last N days/weeks/months
    Predefined,    // Today, yesterday, this week, etc.
}
```

#### Predefined Date Ranges
- **Today**: Files modified today
- **Yesterday**: Files modified yesterday
- **This Week**: Files modified in the current week
- **Last 7 Days**: Files modified in the last 7 days
- **This Month**: Files modified in the current month
- **Last 30 Days**: Files modified in the last 30 days

### File Type Filters

```rust
pub struct FileTypeFilter {
    categories: Vec<FileCategory>,
    extensions: Vec<String>,
    mime_types: Vec<String>,
    exclude_types: Vec<String>,
}

pub enum FileCategory {
    Documents,     // pdf, doc, docx, odt, rtf
    Images,        // jpg, png, gif, bmp, svg
    Audio,         // mp3, wav, flac, ogg
    Video,         // mp4, avi, mkv, mov
    Archives,      // zip, tar, 7z, rar
    Code,          // Source code files
    Executables,   // exe, dmg, deb, rpm
    Data,          // csv, json, xml, sql
}
```

## Search Implementation

### Rust Backend Implementation

The search engine is implemented in `src-tauri/crates/search-engine/src/lib.rs`:

```rust
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use jwalk::WalkDir;
use lru::LruCache;
use rayon::prelude::*;
use std::time::{Duration, Instant};

pub struct SearchEngine {
    directory_cache: LruCache<PathBuf, (Instant, Vec<PathBuf>)>,
    fuzzy_matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        // Use cached directory listing or scan fresh
        let entries = if let Some((cached_time, cached_entries)) = 
            self.directory_cache.get(&query.root_path.into()) {
            if cached_time.elapsed() < Duration::from_secs(30 * 60) { // 30 min TTL
                cached_entries.clone()
            } else {
                self.scan_directory(&query.root_path)
            }
        } else {
            self.scan_directory(&query.root_path)
        };

        // Parallel search with relevance scoring
        entries.par_iter()
            .filter_map(|path| self.evaluate_match(path, query))
            .collect::<Vec<_>>()
            .into_iter()
            .sorted_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap())
            .take(query.options.max_results)
            .collect()
    }

    fn evaluate_match(&self, path: &Path, query: &SearchQuery) -> Option<SearchResult> {
        let file_name = path.file_name()?.to_str()?;
        let mut relevance_score = 0;
        let mut match_type = MatchType::Directory;
        let mut content_matches = Vec::new();

        // Name pattern matching
        if let Some(ref name_pattern) = query.name_pattern {
            if file_name.contains(name_pattern) {
                relevance_score = 100; // Exact match
                match_type = MatchType::ExactName;
            } else if query.options.use_fuzzy {
                if let Some(score) = self.fuzzy_matcher.fuzzy_match(file_name, name_pattern) {
                    if score >= query.options.fuzzy_threshold as i64 {
                        relevance_score = ((score as f32 / 100.0) * 100.0) as u32;
                        match_type = MatchType::FuzzyName;
                    }
                }
            }
        }

        // Content search
        if let Some(ref content_pattern) = query.content_pattern {
            if path.is_file() {
                if let Ok(matches) = self.search_file_content(path, content_pattern) {
                    if !matches.is_empty() {
                        relevance_score += matches.len() as u32 * 10; // 10 points per match
                        match_type = MatchType::Content;
                        content_matches = matches;
                    }
                }
            }
        }

        if relevance_score > 0 {
            Some(SearchResult {
                path: path.to_string_lossy().to_string(),
                name: file_name.to_string(),
                size: path.metadata().ok()?.len(),
                modified: format!("{:?}", path.metadata().ok()?.modified().ok()?),
                file_type: if path.is_dir() { "directory" } else { "file" }.to_string(),
                match_type,
                relevance_score,
                content_matches: if content_matches.is_empty() { None } else { Some(content_matches) },
            })
        } else {
            None
        }
    }
}
```

### Performance Optimizations

#### Directory Traversal
- **Parallel Walking**: Use all CPU cores for directory traversal
- **Efficient Filtering**: Apply filters early to reduce I/O
- **Skip Uninteresting Directories**: Skip .git, node_modules, etc.
- **Respect .gitignore**: Option to respect gitignore patterns

#### Content Search Optimization
```rust
use memmap2::MmapOptions;

async fn search_file_content(
    path: &Path, 
    query: &str,
    options: &ContentSearchOptions
) -> Result<Vec<Match>, SearchError> {
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    
    if options.regex {
        search_regex_in_memory(&mmap, query, options)
    } else {
        search_substring_in_memory(&mmap, query, options)
    }
}
```

**Techniques**:
- **Memory Mapping**: Use memory-mapped files for large files
- **Boyer-Moore Algorithm**: Efficient string searching for large content
- **Parallel Content Search**: Search multiple files simultaneously
- **Early Termination**: Stop search after finding N matches (configurable)

#### Caching Strategy
```rust
pub struct SearchCache {
    directory_metadata: LruCache<PathBuf, DirectoryMetadata>,
    file_content_hashes: LruCache<PathBuf, (SystemTime, ContentHash)>,
    search_results: LruCache<QueryHash, CachedSearchResults>,
}
```

- **Directory Metadata Caching**: Cache directory modification times
- **Content Hash Caching**: Cache content hashes to detect unchanged files
- **Result Caching**: Cache recent search results for similar queries
- **Incremental Updates**: Update caches incrementally for modified directories

## Search Results

### Results Display

```typescript
interface SearchResult {
    path: string;
    filename: string;
    size: number;
    modified: Date;
    fileType: FileType;
    matches?: ContentMatch[];
    relevanceScore: number;
}

interface ContentMatch {
    lineNumber: number;
    columnStart: number;
    columnEnd: number;
    context: string;      // Surrounding text
    snippet: string;      // Highlighted match
}
```

### Results Interface

```
┌─ Search Results: "function main" ────────────────────────────────┐
│                                                                  │
│ Found 127 matches in 45 files (2.3 seconds)        [Cancel]     │
│                                                                  │
│ ┌─ Results ──────────────────────────────────────────────────┐   │
│ │ 📄 main.rs                   /home/user/project/src/       │   │
│ │    Line 15: fn main() {                                    │   │
│ │    Line 42: // Main function entry point                   │   │
│ │                                                            │   │
│ │ 📄 lib.rs                    /home/user/project/src/       │   │
│ │    Line 8: pub fn main_loop() -> Result<(), Error> {      │   │
│ │                                                            │   │
│ │ 📄 readme.md                 /home/user/project/           │   │
│ │    Line 25: The main function handles initialization       │   │
│ │                                                            │   │
│ └────────────────────────────────────────────────────────────┘   │
│                                                                  │
│ ┌─ Actions ──────────────────────────────────────────────────┐   │
│ │ [Open File] [Show in Pane] [Copy Path] [Export Results]   │   │
│ └────────────────────────────────────────────────────────────┘   │
│                                                                  │
│                                           [Close] [New Search]   │
└──────────────────────────────────────────────────────────────────┘
```

### Result Actions

#### Navigation Actions
- **Open File**: Open file in built-in viewer
- **Edit File**: Open file in default editor  
- **Show in Pane**: Navigate pane to file location and select file
- **Open Directory**: Navigate to containing directory

#### Integration Actions
- **Copy Path**: Copy full file path to clipboard
- **Copy Results**: Copy search results as text
- **Export Results**: Save results to CSV or text file
- **Create Collection**: Create file collection from results

## Advanced Features

### Saved Searches

```rust
pub struct SavedSearch {
    name: String,
    description: Option<String>,
    query: SearchQuery,
    created: DateTime<Local>,
    last_used: DateTime<Local>,
    use_count: u32,
}
```

#### Features
- **Named Queries**: Save complex searches with descriptive names
- **Query Templates**: Create template searches for common patterns
- **Recent Searches**: Automatically save recent search queries
- **Search History**: Browse and reuse previous searches

### Search Scopes

```rust
pub enum SearchScope {
    CurrentDirectory,
    CurrentDirectoryRecursive,
    SelectedDirectories(Vec<PathBuf>),
    Bookmarks,
    RecentLocations,
    WholeSystem,
    CustomPaths(Vec<PathBuf>),
}
```

#### Predefined Scopes
- **Current Pane**: Search only the active pane's current directory
- **Both Panes**: Search directories open in both panes
- **Bookmarks**: Search all bookmarked locations
- **Recent**: Search recently visited directories
- **Custom**: User-defined search locations

### Content Indexing (Optional)

For users with extremely large file collections:

```rust
pub struct ContentIndex {
    file_index: HashMap<PathBuf, FileMetadata>,
    word_index: HashMap<String, Vec<FileReference>>,
    last_updated: DateTime<Local>,
}
```

#### Features
- **Full-Text Indexing**: Build searchable index of file contents
- **Incremental Updates**: Update index for changed files only
- **Background Processing**: Build index during idle time
- **Index Statistics**: Show index size, coverage, and freshness

#### Performance Benefits
- **Instant Results**: Search indexed content in milliseconds
- **Fuzzy Matching**: Find approximate matches and typos
- **Relevance Ranking**: Sort results by relevance score
- **Phrase Search**: Search for exact phrases efficiently

## Search Performance

### Benchmarks

Performance targets for different scenarios:

| Scenario | File Count | Target Time | Memory Usage |
|----------|------------|-------------|---------------|
| Small directory | < 1,000 | < 100ms | < 10MB |
| Medium directory | 1,000 - 10,000 | < 500ms | < 50MB |
| Large directory | 10,000 - 100,000 | < 2s | < 200MB |
| Huge directory | > 100,000 | < 10s | < 500MB |

### Optimization Strategies

#### CPU Optimization
- **Parallel Processing**: Utilize all CPU cores effectively
- **SIMD Instructions**: Use vector instructions for string searching
- **Algorithm Selection**: Choose optimal algorithms based on query type
- **Early Termination**: Stop processing when enough results found

#### I/O Optimization
- **Batch Operations**: Group file system operations
- **Prefetch Strategy**: Read-ahead for sequential access patterns
- **Skip Binary Files**: Avoid reading non-text files for content search
- **Compressed Files**: Handle compressed content efficiently

#### Memory Optimization
- **Streaming Results**: Don't store all results in memory
- **Bounded Caches**: Limit cache sizes to prevent memory exhaustion
- **Memory Mapping**: Use OS virtual memory for large files
- **Garbage Collection**: Clean up resources during long searches

---

The search system in Nimbus provides both simplicity for casual users and power for advanced use cases, ensuring fast and accurate results across all types of file collections.