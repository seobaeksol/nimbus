# File Search

## Overview

Nimbus provides a powerful, high-performance file search system that significantly outperforms default OS searches through optimized Rust implementations and parallel processing. The search system is designed to handle large directory structures efficiently while providing real-time results and flexible search criteria.

## Search Interface

### Quick Search (In-Directory)

Quick search provides instant filtering of the current directory contents:

```typescript
interface QuickSearchState {
    pattern: string;
    caseSensitive: boolean;
    wholeWords: boolean;
    regex: boolean;
    matches: FileInfo[];
    activeIndex: number;
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

### Parallel Processing Architecture

```rust
pub struct SearchEngine {
    thread_pool: rayon::ThreadPool,
    cancellation_token: CancellationToken,
    progress_sender: mpsc::UnboundedSender<SearchProgress>,
}

impl SearchEngine {
    pub async fn search(&self, query: SearchQuery) -> SearchResults {
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        
        // Spawn parallel directory walker
        let walker_handle = tokio::spawn(self.parallel_walk(query.clone(), result_tx));
        
        // Spawn result aggregator
        let aggregator_handle = tokio::spawn(self.aggregate_results(result_rx));
        
        // Stream results as they arrive
        SearchResults {
            results: aggregator_handle,
            progress: progress_rx,
            cancellation: self.cancellation_token.clone(),
        }
    }
    
    async fn parallel_walk(
        &self,
        query: SearchQuery,
        result_sender: mpsc::UnboundedSender<SearchResult>
    ) {
        use jwalk::WalkDir;
        
        WalkDir::new(&query.root_path)
            .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus::get()))
            .process_read_dir(|_depth, _path, _read_dir_state, children| {
                children.par_iter().for_each(|child| {
                    if self.cancellation_token.is_cancelled() {
                        return;
                    }
                    
                    if let Ok(entry) = child {
                        if self.matches_criteria(&entry, &query) {
                            let result = SearchResult::from_entry(entry, &query);
                            let _ = result_sender.send(result);
                        }
                    }
                });
            });
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