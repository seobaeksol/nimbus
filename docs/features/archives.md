# Archive Support

## Overview

Nimbus treats archives as virtual filesystems, allowing users to browse, extract, and manipulate archive contents without external tools. This seamless integration means archives appear as regular directories in the file manager, supporting all standard operations like copy, move, and search within archived content.

## Supported Formats

### Core Archive Types

| Format | Extensions | Read | Write | Compression | Notes |
|--------|------------|------|-------|-------------|-------|
| **ZIP** | .zip | ✅ | ✅ | Deflate, Store | Full support including password protection |
| **TAR** | .tar | ✅ | ✅ | None | Uncompressed TAR archives |
| **TAR.GZ** | .tar.gz, .tgz | ✅ | ✅ | Gzip | Most common Linux archive format |
| **TAR.BZ2** | .tar.bz2, .tbz | ✅ | ✅ | Bzip2 | Higher compression than gzip |
| **TAR.XZ** | .tar.xz | ✅ | ❌ | XZ/LZMA | Read-only support initially |
| **7-Zip** | .7z | ✅ | ❌ | LZMA, LZMA2 | Read-only via libarchive |
| **RAR** | .rar | ✅ | ❌ | RAR | Read-only via unrar library |

### Library Implementation

```rust
pub enum ArchiveFormat {
    Zip,
    Tar(CompressionType),
    SevenZ,
    Rar,
}

pub enum CompressionType {
    None,
    Gzip,
    Bzip2,
    Xz,
    Lzma,
}

pub trait ArchiveReader: Send + Sync {
    async fn open(path: &Path) -> Result<Self, ArchiveError> where Self: Sized;
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError>;
    async fn extract_entry(&self, entry: &ArchiveEntry, destination: &Path) -> Result<(), ArchiveError>;
    async fn read_entry_data(&self, entry: &ArchiveEntry) -> Result<Vec<u8>, ArchiveError>;
    fn format(&self) -> ArchiveFormat;
    fn is_encrypted(&self) -> bool;
}
```

#### ZIP Implementation
```toml
[dependencies]
zip = { version = "0.6", features = ["deflate", "bzip2", "time"] }
```

```rust
pub struct ZipReader {
    archive: zip::ZipArchive<File>,
    path: PathBuf,
}

impl ArchiveReader for ZipReader {
    async fn list_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        let mut entries = Vec::new();
        
        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;
            let entry = ArchiveEntry {
                path: PathBuf::from(file.name()),
                size_uncompressed: file.size(),
                size_compressed: file.compressed_size(),
                modified: file.last_modified().to_time(),
                is_directory: file.is_dir(),
                is_encrypted: file.is_encrypted(),
                compression_method: file.compression().into(),
                crc32: Some(file.crc32()),
            };
            entries.push(entry);
        }
        
        Ok(entries)
    }
}
```

#### TAR Implementation  
```toml
[dependencies]
tar = "0.4"
flate2 = "1.0"    # For .tar.gz
bzip2 = "0.4"     # For .tar.bz2
```

```rust
pub struct TarReader {
    archive: tar::Archive<Box<dyn Read + Send>>,
    compression: CompressionType,
    path: PathBuf,
}

impl TarReader {
    pub async fn open(path: &Path) -> Result<Self, ArchiveError> {
        let file = File::open(path)?;
        let reader: Box<dyn Read + Send> = match detect_compression(path)? {
            CompressionType::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
            CompressionType::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
            CompressionType::None => Box::new(file),
            _ => return Err(ArchiveError::UnsupportedFormat),
        };
        
        Ok(TarReader {
            archive: tar::Archive::new(reader),
            compression: detect_compression(path)?,
            path: path.to_path_buf(),
        })
    }
}
```

## Virtual Filesystem Integration

### Archive Filesystem Implementation

Archives are integrated into the main filesystem abstraction:

```rust
pub struct ArchiveFileSystem {
    reader: Box<dyn ArchiveReader>,
    entry_cache: Arc<RwLock<HashMap<PathBuf, ArchiveEntry>>>,
    extraction_cache: Arc<RwLock<LruCache<PathBuf, Vec<u8>>>>,
}

impl FileSystem for ArchiveFileSystem {
    async fn list_dir(&self, path: &Path) -> Result<Vec<FileInfo>, Error> {
        let entries = self.reader.list_entries().await?;
        let mut files = Vec::new();
        
        for entry in entries {
            if entry.path.parent() == Some(path) {
                files.push(FileInfo {
                    name: entry.path.file_name().unwrap().to_string_lossy().to_string(),
                    path: entry.path.clone(),
                    size: entry.size_uncompressed,
                    modified: entry.modified,
                    file_type: if entry.is_directory { FileType::Directory } else { FileType::File },
                    permissions: FilePermissions::readonly(),
                    metadata: Some(FileMetadata::Archive(entry)),
                });
            }
        }
        
        Ok(files)
    }
    
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, Error> {
        // Check extraction cache first
        if let Some(data) = self.extraction_cache.read().unwrap().get(path) {
            return Ok(data.clone());
        }
        
        // Find entry in archive
        let entries = self.reader.list_entries().await?;
        let entry = entries.iter()
            .find(|e| &e.path == path)
            .ok_or(Error::FileNotFound)?;
        
        // Extract and cache
        let data = self.reader.read_entry_data(entry).await?;
        self.extraction_cache.write().unwrap().put(path.to_path_buf(), data.clone());
        
        Ok(data)
    }
}
```

### Path Resolution

Archive paths use a special scheme to distinguish them from regular filesystem paths:

```rust
pub enum VirtualPath {
    Local(PathBuf),
    Archive {
        archive_path: PathBuf,
        internal_path: PathBuf,
    },
    Remote {
        connection_id: String,
        path: String,
    },
}

impl VirtualPath {
    pub fn parse(path_str: &str) -> Result<Self, PathError> {
        if path_str.contains("!/") {
            // Archive path: /path/to/archive.zip!/internal/path
            let parts: Vec<&str> = path_str.splitn(2, "!/").collect();
            Ok(VirtualPath::Archive {
                archive_path: PathBuf::from(parts[0]),
                internal_path: PathBuf::from(parts[1]),
            })
        } else if path_str.starts_with("://") {
            // Remote path: sftp://server/path
            // ... remote path parsing
        } else {
            Ok(VirtualPath::Local(PathBuf::from(path_str)))
        }
    }
}
```

### Breadcrumb Navigation

Archives appear seamlessly in the path breadcrumb:

```
Home > Documents > Projects > backup.zip > src > main.rs
                               ^^^^^^^^^
                            Archive boundary
```

```typescript
interface PathSegment {
    name: string;
    path: string;
    type: 'directory' | 'archive' | 'archive_entry';
    clickable: boolean;
}

// Example breadcrumb for archive content
const breadcrumbSegments: PathSegment[] = [
    { name: 'Home', path: '/home/user', type: 'directory', clickable: true },
    { name: 'Documents', path: '/home/user/Documents', type: 'directory', clickable: true },
    { name: 'backup.zip', path: '/home/user/Documents/backup.zip', type: 'archive', clickable: true },
    { name: 'src', path: '/home/user/Documents/backup.zip!/src', type: 'archive_entry', clickable: true },
    { name: 'main.rs', path: '/home/user/Documents/backup.zip!/src/main.rs', type: 'archive_entry', clickable: false }
];
```

## Archive Operations

### Browsing Archives

#### Entry into Archives
- **Double-click**: Enter archive like a directory
- **Enter key**: Navigate into archive with keyboard
- **Context menu**: "Browse archive" option
- **Drag & drop**: Drop files onto archive to add (if supported)

#### Visual Indicators
- **Archive Icons**: Distinct icons for different archive types
- **Path Indicator**: Special notation in path bar (archive.zip!/)
- **Status Bar**: Show archive information (compressed size, entries count)
- **Loading States**: Progress indication while reading large archives

### File Extraction

#### Extraction Methods
```rust
pub enum ExtractionMode {
    SelectedFiles,    // Extract only selected files
    EntireArchive,    // Extract all contents
    FolderStructure,  // Extract folder and its contents
}

pub struct ExtractionOptions {
    destination: PathBuf,
    mode: ExtractionMode,
    overwrite_policy: OverwritePolicy,
    preserve_paths: bool,
    preserve_timestamps: bool,
    create_subfolder: bool,
}
```

#### Extraction Operations
- **F5 (Copy)**: Extract selected files to opposite pane
- **Drag & Drop**: Drag files out of archive to extract
- **Context Menu**: "Extract to..." with destination chooser
- **Bulk Extract**: "Extract All" for entire archive

#### Progress Tracking
```rust
pub struct ExtractionProgress {
    pub current_file: String,
    pub files_extracted: u32,
    pub total_files: u32,
    pub bytes_extracted: u64,
    pub total_bytes: u64,
    pub speed: u64,  // bytes per second
    pub eta: Duration,
}
```

### Archive Creation

#### Supported Creation Formats
- **ZIP**: Full creation support with compression options
- **TAR.GZ**: Create compressed TAR archives
- **TAR.BZ2**: Create bzip2-compressed TAR archives

#### Creation Interface
```
┌─ Create Archive ──────────────────────────────────────────────┐
│                                                               │
│ Archive Name: [backup_2024_01_15.zip        ]                │
│                                                               │
│ Format: ● ZIP    ○ TAR.GZ    ○ TAR.BZ2                       │
│                                                               │
│ ┌─ Compression Settings ─────────────────────────────────┐    │
│ │ Level: [████████░░] Fast ←→ Best                       │    │
│ │ Method: [Deflate ▼]                                    │    │
│ │ □ Create solid archive (7z only)                       │    │
│ │ □ Encrypt with password                                 │    │
│ └─────────────────────────────────────────────────────────┘    │
│                                                               │
│ ┌─ Options ─────────────────────────────────────────────┐    │
│ │ ☑ Store full paths                                     │    │
│ │ ☑ Include hidden files                                 │    │
│ │ ☑ Follow symbolic links                                │    │
│ │ □ Include file attributes                               │    │
│ └─────────────────────────────────────────────────────────┘    │
│                                                               │
│ Selected: 47 files, 128 directories (2.3 GB)                │
│                                                               │
│ [Create] [Cancel] [Advanced...]                              │
└───────────────────────────────────────────────────────────────┘
```

#### Creation Process
```rust
pub async fn create_archive(
    files: Vec<PathBuf>,
    destination: PathBuf,
    format: ArchiveFormat,
    options: CompressionOptions,
    progress_callback: impl Fn(CreationProgress)
) -> Result<(), ArchiveError> {
    match format {
        ArchiveFormat::Zip => create_zip_archive(files, destination, options, progress_callback).await,
        ArchiveFormat::Tar(compression) => create_tar_archive(files, destination, compression, options, progress_callback).await,
        _ => Err(ArchiveError::UnsupportedFormat),
    }
}
```

## Advanced Features

### Password-Protected Archives

#### ZIP Password Support
```rust
impl ZipReader {
    pub async fn open_with_password(path: &Path, password: &str) -> Result<Self, ArchiveError> {
        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        // Verify password by trying to read first encrypted entry
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            if entry.is_encrypted() {
                entry.read_with_password(password.as_bytes())?;
                break;
            }
        }
        
        Ok(ZipReader { archive, path: path.to_path_buf() })
    }
}
```

#### Password Management
- **Password Prompt**: Secure password input dialog
- **Session Storage**: Keep passwords in memory for session duration
- **Keychain Integration**: Store passwords in OS keychain (optional)
- **Master Password**: Encrypt stored passwords with master password

### Multi-Volume Archives

Support for archives split across multiple files:

```rust
pub struct MultiVolumeArchive {
    volumes: Vec<PathBuf>,
    current_volume: usize,
    readers: HashMap<usize, Box<dyn ArchiveReader>>,
}

impl MultiVolumeArchive {
    pub fn detect_volumes(path: &Path) -> Result<Vec<PathBuf>, ArchiveError> {
        // Detect .zip, .z01, .z02, etc.
        // Or .rar, .r00, .r01, etc.
        let base_path = path.with_extension("");
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        match extension {
            "zip" => detect_zip_volumes(&base_path),
            "rar" => detect_rar_volumes(&base_path),
            _ => Ok(vec![path.to_path_buf()]),
        }
    }
}
```

### Archive Integrity Verification

#### Checksum Verification
```rust
pub async fn verify_archive_integrity(
    archive_path: &Path,
    progress_callback: impl Fn(VerificationProgress)
) -> Result<IntegrityReport, ArchiveError> {
    let reader = ArchiveReader::open(archive_path).await?;
    let entries = reader.list_entries().await?;
    let mut report = IntegrityReport::new();
    
    for entry in entries {
        let calculated_crc = calculate_entry_crc(&reader, &entry).await?;
        let stored_crc = entry.crc32.unwrap_or(0);
        
        if calculated_crc != stored_crc {
            report.corrupted_entries.push(entry.path.clone());
        }
        
        progress_callback(VerificationProgress {
            current_entry: entry.path.clone(),
            completed: report.total_checked,
            total: entries.len(),
        });
    }
    
    Ok(report)
}
```

#### Health Check
- **CRC Verification**: Verify stored checksums against calculated values
- **Structure Validation**: Ensure archive structure is valid
- **Compression Integrity**: Verify compressed data can be decompressed
- **Metadata Consistency**: Check internal metadata consistency

### Archive Comparison

Compare contents of different archives:

```rust
pub struct ArchiveComparison {
    pub identical_files: Vec<String>,
    pub different_files: Vec<FileDifference>,
    pub left_only: Vec<String>,
    pub right_only: Vec<String>,
    pub summary: ComparisonSummary,
}

pub async fn compare_archives(
    left_archive: &Path,
    right_archive: &Path,
    options: ComparisonOptions
) -> Result<ArchiveComparison, ArchiveError> {
    // Compare file listings, sizes, dates, and optionally content
}
```

## Performance Optimization

### Caching Strategy

```rust
pub struct ArchiveCache {
    // Cache archive directory listings
    directory_cache: LruCache<PathBuf, Vec<ArchiveEntry>>,
    
    // Cache small extracted files
    file_cache: LruCache<PathBuf, Vec<u8>>,
    
    // Cache archive metadata
    metadata_cache: LruCache<PathBuf, ArchiveMetadata>,
    
    // Track modification times
    mtime_cache: HashMap<PathBuf, SystemTime>,
}
```

**Cache Invalidation**:
- Monitor archive file modification times
- Invalidate cache when archive is modified
- LRU eviction for memory management
- Configurable cache sizes

### Memory Management

#### Large Archive Handling
- **Streaming Extraction**: Don't load entire archives into memory
- **Lazy Loading**: Load archive entries on-demand
- **Memory Limits**: Configurable limits for cache sizes
- **Background Cleanup**: Clean up unused cache entries

#### Performance Monitoring
```rust
pub struct ArchivePerformanceMetrics {
    pub extraction_time: Duration,
    pub listing_time: Duration,
    pub memory_usage: usize,
    pub cache_hit_rate: f64,
    pub io_operations: u32,
}
```

### Background Processing

#### Preemptive Caching
- **Directory Listing Cache**: Cache frequently accessed archive listings  
- **Thumbnail Generation**: Generate thumbnails for images in archives
- **Content Indexing**: Index text content in archives for search
- **Metadata Extraction**: Extract metadata in background

## Error Handling

### Error Types
```rust
#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("Archive format not supported: {0}")]
    UnsupportedFormat(String),
    
    #[error("Archive is password protected")]
    PasswordRequired,
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Archive is corrupted")]
    CorruptedArchive,
    
    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),
    
    #[error("Insufficient disk space")]
    InsufficientSpace,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Recovery Strategies
- **Partial Extraction**: Continue extraction despite individual file failures
- **Corruption Repair**: Attempt to recover readable portions of corrupted archives  
- **Alternative Readers**: Try different libraries for problematic archives
- **User Guidance**: Provide clear instructions for manual recovery

---

Archive support in Nimbus provides seamless integration with the virtual filesystem, enabling users to work with archived content as naturally as regular directories while maintaining high performance and reliability.