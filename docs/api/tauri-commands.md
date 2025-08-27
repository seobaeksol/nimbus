# Tauri Commands API

## Overview

Tauri commands provide the bridge between the React frontend and Rust backend. All commands are asynchronous and use JSON serialization for data exchange. Commands are organized by functionality and follow consistent patterns for error handling and data validation.

## File System Commands

### Directory Operations

#### `list_dir`
Lists the contents of a directory.

```rust
#[tauri::command]
pub async fn list_dir(path: String) -> Result<Vec<FileInfo>, FileError>
```

**Parameters:**
- `path` (string): Absolute path to the directory

**Returns:**
- `FileInfo[]`: Array of file and directory information

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const files = await invoke<FileInfo[]>('list_dir', {
    path: '/home/user/documents'
});
```

#### `create_directory`
Creates a new directory.

```rust
#[tauri::command]
pub async fn create_directory(path: String, name: String) -> Result<(), FileError>
```

**Parameters:**
- `path` (string): Parent directory path
- `name` (string): Name of the new directory

**Example:**
```typescript
await invoke('create_directory', {
    path: '/home/user/documents',
    name: 'new_folder'
});
```

#### `get_directory_size`
Calculates the total size of a directory recursively.

```rust
#[tauri::command]
pub async fn get_directory_size(path: String) -> Result<DirectorySizeInfo, FileError>
```

**Returns:**
- `DirectorySizeInfo`: Size information including file count and total bytes

### File Operations

#### `copy_files`
Copies files from source to destination.

```rust
#[tauri::command]
pub async fn copy_files(
    sources: Vec<String>,
    destination: String,
    options: CopyOptions
) -> Result<String, FileError>
```

**Parameters:**
- `sources` (string[]): Array of source file paths
- `destination` (string): Destination directory path
- `options` (CopyOptions): Copy operation options

**CopyOptions:**
```typescript
interface CopyOptions {
    overwritePolicy: 'ask' | 'overwrite' | 'skip' | 'rename';
    preserveTimestamps: boolean;
    verifyIntegrity: boolean;
    showProgress: boolean;
}
```

**Returns:**
- `string`: Operation ID for tracking progress

**Example:**
```typescript
const operationId = await invoke<string>('copy_files', {
    sources: ['/path/to/file1.txt', '/path/to/file2.txt'],
    destination: '/path/to/destination/',
    options: {
        overwritePolicy: 'ask',
        preserveTimestamps: true,
        verifyIntegrity: false,
        showProgress: true
    }
});
```

#### `move_files`
Moves files from source to destination.

```rust
#[tauri::command]
pub async fn move_files(
    sources: Vec<String>,
    destination: String,
    options: MoveOptions
) -> Result<String, FileError>
```

Similar parameters and options to `copy_files`.

#### `delete_files`
Deletes files or directories.

```rust
#[tauri::command]
pub async fn delete_files(
    paths: Vec<String>,
    options: DeleteOptions
) -> Result<String, FileError>
```

**DeleteOptions:**
```typescript
interface DeleteOptions {
    useTrash: boolean;        // Move to trash instead of permanent deletion
    force: boolean;           // Delete read-only files
    recursive: boolean;       // Delete directories recursively
    secure: boolean;          // Secure deletion (overwrite data)
}
```

#### `rename_file`
Renames a file or directory.

```rust
#[tauri::command]
pub async fn rename_file(
    old_path: String,
    new_name: String
) -> Result<String, FileError>
```

**Returns:**
- `string`: New file path after rename

### File Information

#### `get_file_info`
Retrieves detailed information about a file or directory.

```rust
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, FileError>
```

**FileInfo Structure:**
```typescript
interface FileInfo {
    name: string;
    path: string;
    size: number;
    sizeFormatted: string;
    modified: string;        // ISO 8601 timestamp
    created?: string;        // ISO 8601 timestamp
    accessed?: string;       // ISO 8601 timestamp
    fileType: 'file' | 'directory' | 'symlink';
    extension?: string;
    mimeType?: string;
    permissions: FilePermissions;
    isHidden: boolean;
    isReadonly: boolean;
    owner?: string;
    group?: string;
}

interface FilePermissions {
    read: boolean;
    write: boolean;
    execute: boolean;
    ownerRead: boolean;
    ownerWrite: boolean;
    ownerExecute: boolean;
    groupRead: boolean;
    groupWrite: boolean;
    groupExecute: boolean;
    otherRead: boolean;
    otherWrite: boolean;
    otherExecute: boolean;
}
```

#### `get_file_hash`
Calculates hash checksums for a file.

```rust
#[tauri::command]
pub async fn get_file_hash(
    path: String,
    algorithm: HashAlgorithm
) -> Result<String, FileError>
```

**HashAlgorithm:**
```typescript
type HashAlgorithm = 'md5' | 'sha1' | 'sha256' | 'sha512';
```

## Search Commands

#### `start_search`
Initiates a file search operation.

```rust
#[tauri::command]
pub async fn start_search(query: SearchQuery) -> Result<String, SearchError>
```

**SearchQuery:**
```typescript
interface SearchQuery {
    rootPath: string;
    namePattern?: string;
    contentPattern?: string;
    sizeFilter?: SizeFilter;
    dateFilter?: DateFilter;
    fileTypeFilter?: FileTypeFilter;
    options: SearchOptions;
}

interface SearchOptions {
    caseSensitive: boolean;
    useRegex: boolean;
    includeHidden: boolean;
    followSymlinks: boolean;
    maxResults?: number;
    maxDepth?: number;
}

interface SizeFilter {
    minSize?: number;
    maxSize?: number;
    unit: 'bytes' | 'kb' | 'mb' | 'gb';
}

interface DateFilter {
    dateType: 'modified' | 'created' | 'accessed';
    startDate?: string;      // ISO 8601
    endDate?: string;        // ISO 8601
}

interface FileTypeFilter {
    extensions: string[];
    categories: FileCategory[];
}

type FileCategory = 'documents' | 'images' | 'audio' | 'video' | 'archives' | 'code';
```

**Returns:**
- `string`: Search ID for tracking results

#### `cancel_search`
Cancels an ongoing search operation.

```rust
#[tauri::command]
pub async fn cancel_search(search_id: String) -> Result<(), SearchError>
```

## Archive Commands

#### `list_archive_contents`
Lists the contents of an archive file.

```rust
#[tauri::command]
pub async fn list_archive_contents(
    archive_path: String,
    internal_path: Option<String>
) -> Result<Vec<ArchiveEntry>, ArchiveError>
```

**ArchiveEntry:**
```typescript
interface ArchiveEntry {
    path: string;
    name: string;
    size: number;
    compressedSize: number;
    modified?: string;
    isDirectory: boolean;
    compressionMethod?: string;
    crc32?: number;
    isEncrypted: boolean;
}
```

#### `extract_archive`
Extracts files from an archive.

```rust
#[tauri::command]
pub async fn extract_archive(
    archive_path: String,
    entries: Option<Vec<String>>,
    destination: String,
    options: ExtractionOptions
) -> Result<String, ArchiveError>
```

**ExtractionOptions:**
```typescript
interface ExtractionOptions {
    preservePaths: boolean;
    overwritePolicy: 'ask' | 'overwrite' | 'skip' | 'rename';
    password?: string;
    createSubfolder: boolean;
}
```

#### `create_archive`
Creates a new archive from selected files.

```rust
#[tauri::command]
pub async fn create_archive(
    files: Vec<String>,
    archive_path: String,
    format: ArchiveFormat,
    options: CompressionOptions
) -> Result<String, ArchiveError>
```

**CompressionOptions:**
```typescript
interface CompressionOptions {
    level: number;           // 0-9 compression level
    method: string;          // compression method
    password?: string;       // for encrypted archives
    solid: boolean;          // solid compression (7z)
}

type ArchiveFormat = 'zip' | 'tar' | 'tar.gz' | 'tar.bz2' | '7z';
```

## Remote File System Commands

#### `connect_remote`
Establishes a connection to a remote server.

```rust
#[tauri::command]
pub async fn connect_remote(config: RemoteConnectionConfig) -> Result<String, RemoteError>
```

**RemoteConnectionConfig:**
```typescript
interface RemoteConnectionConfig {
    protocol: 'ftp' | 'sftp' | 'webdav';
    host: string;
    port?: number;
    username: string;
    password?: string;
    keyFile?: string;        // SSH private key file path
    passive?: boolean;       // FTP passive mode
    timeout?: number;        // Connection timeout in seconds
}
```

**Returns:**
- `string`: Connection ID for subsequent operations

#### `disconnect_remote`
Closes a remote connection.

```rust
#[tauri::command]
pub async fn disconnect_remote(connection_id: String) -> Result<(), RemoteError>
```

#### `list_remote_dir`
Lists contents of a remote directory.

```rust
#[tauri::command]
pub async fn list_remote_dir(
    connection_id: String,
    path: String
) -> Result<Vec<RemoteFileInfo>, RemoteError>
```

#### `download_remote_file`
Downloads a file from remote server.

```rust
#[tauri::command]
pub async fn download_remote_file(
    connection_id: String,
    remote_path: String,
    local_path: String,
    options: TransferOptions
) -> Result<String, RemoteError>
```

#### `upload_remote_file`
Uploads a file to remote server.

```rust
#[tauri::command]
pub async fn upload_remote_file(
    connection_id: String,
    local_path: String,
    remote_path: String,
    options: TransferOptions
) -> Result<String, RemoteError>
```

**TransferOptions:**
```typescript
interface TransferOptions {
    overwrite: boolean;
    preserveTimestamps: boolean;
    verifyIntegrity: boolean;
    resume: boolean;         // Resume interrupted transfers
}
```

## File Viewer Commands

#### `read_text_file`
Reads a text file with encoding detection.

```rust
#[tauri::command]
pub async fn read_text_file(
    path: String,
    encoding: Option<String>,
    max_size: Option<u64>
) -> Result<TextFileContent, ViewerError>
```

**TextFileContent:**
```typescript
interface TextFileContent {
    content: string;
    encoding: string;
    lineCount: number;
    size: number;
    isTruncated: boolean;
}
```

#### `get_image_info`
Retrieves information about an image file.

```rust
#[tauri::command]
pub async fn get_image_info(path: String) -> Result<ImageInfo, ViewerError>
```

**ImageInfo:**
```typescript
interface ImageInfo {
    width: number;
    height: number;
    format: string;
    colorDepth: number;
    hasAlpha: boolean;
    exifData?: Record<string, any>;
}
```

#### `read_hex_data`
Reads binary data for hex viewer.

```rust
#[tauri::command]
pub async fn read_hex_data(
    path: String,
    offset: u64,
    length: u32
) -> Result<HexData, ViewerError>
```

**HexData:**
```typescript
interface HexData {
    data: number[];          // Byte array
    offset: number;
    length: number;
    fileSize: number;
}
```

## System Commands

#### `get_system_info`
Retrieves system information.

```rust
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, SystemError>
```

**SystemInfo:**
```typescript
interface SystemInfo {
    platform: string;
    version: string;
    arch: string;
    hostname: string;
    username: string;
    homeDir: string;
    tempDir: string;
    drives: DriveInfo[];
}

interface DriveInfo {
    path: string;
    label?: string;
    fsType: string;
    totalSpace: number;
    freeSpace: number;
    isRemovable: boolean;
}
```

#### `open_external`
Opens a file or URL with the default system application.

```rust
#[tauri::command]
pub async fn open_external(path: String) -> Result<(), SystemError>
```

#### `reveal_in_explorer`
Shows a file in the system file manager.

```rust
#[tauri::command]
pub async fn reveal_in_explorer(path: String) -> Result<(), SystemError>
```

## Error Handling

### Error Types

All commands use structured error types:

```typescript
interface FileError {
    kind: 'NotFound' | 'PermissionDenied' | 'AlreadyExists' | 'InvalidPath' | 'IoError';
    message: string;
    path?: string;
}

interface SearchError {
    kind: 'InvalidQuery' | 'IoError' | 'Cancelled';
    message: string;
}

interface ArchiveError {
    kind: 'UnsupportedFormat' | 'PasswordRequired' | 'InvalidPassword' | 'CorruptedArchive' | 'ExtractionFailed';
    message: string;
}

interface RemoteError {
    kind: 'ConnectionFailed' | 'AuthenticationFailed' | 'NetworkError' | 'ServerError';
    message: string;
    serverMessage?: string;
}
```

### Error Handling Example

```typescript
try {
    const files = await invoke<FileInfo[]>('list_dir', { path: '/invalid/path' });
} catch (error: any) {
    if (error.kind === 'NotFound') {
        console.error('Directory not found:', error.path);
    } else if (error.kind === 'PermissionDenied') {
        console.error('Permission denied:', error.message);
    } else {
        console.error('Unexpected error:', error.message);
    }
}
```

## Event System

Some commands emit events for progress tracking and real-time updates:

### Progress Events

```typescript
import { listen } from '@tauri-apps/api/event';

// Listen for file operation progress
const unlisten = await listen<ProgressEvent>('file-operation-progress', (event) => {
    const progress = event.payload;
    console.log(`Progress: ${progress.current}/${progress.total}`);
});

interface ProgressEvent {
    operationId: string;
    operationType: 'copy' | 'move' | 'delete' | 'extract' | 'compress';
    current: number;
    total: number;
    currentFile?: string;
    bytesProcessed: number;
    bytesTotal: number;
    speed: number;           // bytes per second
    eta: number;             // seconds remaining
    status: 'running' | 'paused' | 'completed' | 'error' | 'cancelled';
}
```

### Search Result Events

```typescript
// Listen for search results
const unlisten = await listen<SearchResult>('search-result', (event) => {
    const result = event.payload;
    addSearchResult(result);
});

interface SearchResult {
    searchId: string;
    path: string;
    name: string;
    size: number;
    modified: string;
    matches?: ContentMatch[];
}
```

---

This API documentation provides comprehensive coverage of all Tauri commands available in Nimbus. All commands are designed to be type-safe, with proper error handling and consistent patterns across the application.