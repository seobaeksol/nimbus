# Development Roadmap

## Overview

Nimbus follows an **incremental development approach** where each phase builds upon the previous one, ensuring a usable application at every milestone. This strategy allows for early user feedback and reduces risk through continuous validation.

## Development Phases

### Phase 1: Core Prototype (2-3 weeks)
**Objective**: Establish foundational architecture and prove IPC communication

#### Deliverables
- [x] **Tauri Project Setup**: Basic project structure with Rust backend and React frontend
- [x] **Minimal Multi-Panel UI**: Simple side-by-side layout without advanced features  
- [x] **Basic IPC Communication**: "Hello World" commands between React and Rust
- [x] **Local Directory Listing**: Single-tab file browsing for local filesystem only
- [x] **Basic Navigation**: Folder entry and parent directory navigation

#### Technical Goals
```rust
// Core commands implemented
#[tauri::command]
fn list_dir(path: String) -> Result<Vec<FileInfo>, String>

#[tauri::command] 
fn navigate_to(path: String) -> Result<Vec<FileInfo>, String>

#[tauri::command]
fn get_parent_dir(path: String) -> Result<String, String>
```

#### Success Criteria
- Application launches successfully on all three platforms
- User can browse local directories in multi-panel interface
- File list displays name, size, type, and modification date
- Navigation works correctly (double-click to enter folders)

---

### Phase 2: File Operations & Tabs (3-4 weeks)
**Objective**: Implement core file management functionality with multi-tab support

#### Deliverables
- [x] **File Operations**: Copy, move, delete, rename, create folder with confirmation dialogs
- [x] **Multi-tab Support**: Multiple tabs per pane with independent navigation
- [x] **Context Menu**: Right-click menu with relevant file operations
- [x] **File Selection**: Single and multi-select with keyboard shortcuts
- [x] **Basic Icons**: File type icons and folder representations
- [x] **Column Sorting**: Sort files by name, size, date, type

#### Technical Implementation
```rust
// File operations
#[tauri::command]
async fn copy_files(src_paths: Vec<String>, dst_path: String) -> Result<(), String>

#[tauri::command]
async fn move_files(src_paths: Vec<String>, dst_path: String) -> Result<(), String>

#[tauri::command]
async fn delete_files(paths: Vec<String>) -> Result<(), String>

#[tauri::command]
async fn create_folder(path: String, name: String) -> Result<(), String>
```

#### UI Features
- Tab management (Ctrl+T new tab, Ctrl+W close tab)
- Progress dialogs for long operations
- Error handling with user-friendly messages
- Keyboard shortcuts (F5 copy, F8 delete, F6 move)

#### Success Criteria
- All basic file operations work reliably
- Multi-tab interface functions without memory leaks
- Error conditions handled gracefully
- Performance acceptable for directories with thousands of files

---

### Phase 3: Recursive Search (2-3 weeks)
**Objective**: High-performance file search with streaming results

#### Deliverables
- [x] **Search Dialog**: User interface for entering search criteria
- [x] **Filename Search**: Pattern matching with wildcards and regex support
- [x] **Parallel Processing**: Multi-threaded directory traversal using `jwalk`
- [x] **Streaming Results**: Real-time result display as files are found
- [x] **Search Cancellation**: Ability to stop long-running searches
- [x] **Results Navigation**: Jump to file location from search results

#### Technical Implementation
```rust
use jwalk::WalkDir;
use tokio::sync::mpsc;

#[tauri::command]
async fn start_search(query: SearchQuery) -> Result<String, String> {
    let search_id = Uuid::new_v4().to_string();
    // Spawn background search task
    // Stream results via Tauri events
}

#[tauri::command]
async fn cancel_search(search_id: String) -> Result<(), String>
```

#### Performance Targets
- **Small directories** (< 1K files): < 100ms
- **Medium directories** (< 100K files): < 2 seconds  
- **Large directories** (< 1M files): < 10 seconds
- **Memory usage**: < 100MB during search operations

#### Success Criteria
- Search significantly faster than native OS search
- UI remains responsive during search operations
- Results appear immediately as they're found
- Memory usage remains bounded for large searches

---

### Phase 4: Archive Support (3-4 weeks)
**Objective**: Virtual filesystem for common archive formats

#### Deliverables
- [x] **ZIP Support**: Browse and extract using `zip` crate
- [x] **TAR Support**: Handle .tar, .tar.gz, .tar.bz2 formats
- [x] **7z Support**: Integration via `sevenz-rust` or external binary
- [x] **RAR Support**: Read-only via `unrar` crate
- [x] **Virtual Navigation**: Browse archives like regular directories
- [x] **Extraction Operations**: Extract files and folders from archives
- [x] **Archive Creation**: Create ZIP archives from selected files

#### Archive Integration
```rust
// Virtual filesystem implementation
pub enum FileSystemType {
    Local(LocalFileSystem),
    Archive(ArchiveFileSystem),
    Remote(RemoteFileSystem),
}

impl FileSystem for FileSystemType {
    async fn list_dir(&self, path: &Path) -> Result<Vec<FileInfo>, Error> {
        match self {
            FileSystemType::Archive(fs) => fs.list_dir(path).await,
            // ... other implementations
        }
    }
}
```

#### Features
- Seamless copy operations between archives and local filesystem
- Progress indication for extraction operations
- Archive integrity verification
- Support for password-protected archives (ZIP)

#### Success Criteria
- Browse large archives (>10K files) without performance degradation
- Extract operations complete successfully with progress feedback
- Create archives with compression options
- Handle nested archives (archive within archive)

---

### Phase 5: FTP/SFTP Integration (3-4 weeks)
**Objective**: Remote file system access with secure protocols

#### Deliverables
- [x] **Connection Manager**: Save and manage remote connections
- [x] **SFTP Support**: Secure file transfer with SSH key authentication
- [x] **FTP Support**: Basic and anonymous FTP connections
- [x] **Remote Navigation**: Browse remote directories like local ones
- [x] **Transfer Operations**: Upload/download with progress tracking
- [x] **Connection Persistence**: Maintain connections across operations

#### Security Implementation
```rust
// Secure credential storage
use keyring::Entry;

pub struct ConnectionManager {
    keyring: Entry,
}

impl ConnectionManager {
    pub fn store_credentials(&self, host: &str, username: &str, password: &str) -> Result<(), Error> {
        // Store in OS keychain
    }
    
    pub fn get_credentials(&self, host: &str) -> Result<(String, String), Error> {
        // Retrieve from OS keychain
    }
}
```

#### Protocol Support
- **SFTP**: Username/password and SSH key authentication
- **FTP**: Active and passive modes, SSL/TLS encryption (FTPS)
- **Connection pooling**: Reuse connections for multiple operations
- **Timeout handling**: Graceful handling of network issues

#### Success Criteria
- Establish connections reliably across different server configurations
- File transfers complete with accurate progress reporting
- Handle network interruptions gracefully with retry mechanisms
- Credentials stored securely using OS-provided keychain services

---

### Phase 6: Built-in Viewers (2-3 weeks)
**Objective**: Integrated file content viewing without external applications

#### Deliverables
- [x] **Text Viewer**: UTF-8 text files with optional syntax highlighting
- [x] **Image Viewer**: Common formats (JPG, PNG, GIF, BMP, WebP)
- [x] **Hex Viewer**: Binary file inspection with ASCII sidebar
- [x] **Viewer Window Management**: Modal dialogs or detached windows
- [x] **Quick Preview**: Space bar for instant preview (like macOS Finder)

#### Text Viewer Features
```typescript
interface TextViewerProps {
    filePath: string;
    language?: string;
    readOnly: boolean;
    maxFileSize: number; // 10MB limit
}

// Optional syntax highlighting for common file types
const SupportedLanguages = [
    'javascript', 'typescript', 'rust', 'python', 'json', 'xml', 'css', 'html'
];
```

#### Image Viewer Features  
- Basic controls: zoom, rotate, fit to window
- Image metadata display (dimensions, file size, format)
- Support for animated GIFs
- Memory-efficient loading for large images

#### Hex Viewer Features
- Configurable bytes per row (8, 16, 32)
- ASCII representation alongside hex values
- Search functionality within binary data
- Offset display and navigation

#### Success Criteria
- Viewers handle large files efficiently (text files up to 100MB)
- Image viewer supports all common formats without external dependencies
- Hex viewer responsive for files up to 50MB
- Quick preview provides instant feedback

---

### Phase 7: Plugin System Foundation (4-5 weeks)
**Objective**: Establish extensible plugin architecture

#### Deliverables
- [x] **Plugin API Design**: Stable interfaces for content, protocol, and viewer plugins
- [x] **Dynamic Loading**: Runtime plugin discovery and loading mechanism
- [x] **Example Plugins**: Sample implementations demonstrating each plugin type
- [x] **Plugin Manager UI**: Enable/disable plugins, view plugin information
- [x] **SDK Documentation**: Comprehensive guide for plugin developers
- [x] **Version Compatibility**: API versioning and compatibility checking

#### Plugin Types Implementation
```rust
// Content plugin for custom file metadata
pub trait ContentPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> Version;
    fn supported_extensions(&self) -> Vec<String>;
    fn get_columns(&self, file_info: &FileInfo) -> Result<HashMap<String, String>, Error>;
}

// Protocol plugin for custom remote filesystems  
pub trait ProtocolPlugin: Send + Sync {
    fn scheme(&self) -> &str;
    fn version(&self) -> Version;
    fn create_client(&self, config: &ConnectionConfig) -> Result<Box<dyn RemoteClient>, Error>;
}

// Viewer plugin for custom file formats
pub trait ViewerPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> Version;
    fn supported_extensions(&self) -> Vec<String>;
    fn render(&self, file_path: &Path) -> Result<ViewerContent, Error>;
}
```

#### Safety & Security
- Plugin sandboxing options (separate processes, WebAssembly)
- Code signing verification for trusted plugins
- API capability restrictions
- Memory and resource limits

#### Success Criteria
- Load and unload plugins without application restart
- Plugin crashes don't affect main application stability
- Clear separation between plugin and core application APIs
- Comprehensive examples enable third-party development

---

### Phase 8: Polish & UX Improvements (3-4 weeks)
**Objective**: Refine user experience and visual design

#### Deliverables
- [x] **Visual Design**: Professional icon set, consistent styling, themes
- [x] **Drag & Drop**: Full support for file operations between panes and external applications
- [x] **Keyboard Navigation**: Complete keyboard shortcuts matching Total Commander
- [x] **Context Sensitivity**: Dynamic menus and toolbars based on selection
- [x] **Error Handling**: User-friendly error messages with recovery suggestions
- [x] **Accessibility**: Screen reader support, keyboard-only navigation

#### UI/UX Improvements
```typescript
// Keyboard shortcut system
const KeyboardShortcuts = {
    'F5': 'copy',
    'F6': 'move', 
    'F8': 'delete',
    'F3': 'view',
    'Tab': 'switch-pane',
    'Ctrl+T': 'new-tab',
    'Ctrl+W': 'close-tab',
    'Ctrl+F': 'search',
    // ... comprehensive shortcut system
};
```

#### Visual Polish
- **Themes**: Light and dark theme support with system preference detection
- **Icons**: Consistent iconography with file type recognition
- **Animations**: Subtle transitions for better user feedback
- **Layout**: Responsive design for different screen sizes and DPI settings

#### Success Criteria
- Application feels native on each platform
- All common operations accessible via keyboard
- Visual design consistent with platform conventions
- Accessibility compliance (WCAG 2.1 AA)

---

### Phase 9: Packaging & Distribution (2-3 weeks)
**Objective**: Production-ready installers with security

#### Deliverables
- [x] **Code Signing**: Valid certificates for Windows and macOS
- [x] **Installers**: MSI/NSIS for Windows, DMG for macOS, AppImage/deb/rpm for Linux
- [x] **Auto-updater**: Secure update mechanism with signature verification
- [x] **CI/CD Pipeline**: Automated builds and releases
- [x] **Documentation**: Installation guides and release notes

#### Security Implementation
```json
// Tauri configuration for code signing
{
  "tauri": {
    "bundle": {
      "windows": {
        "certificateThumbprint": "CERT_THUMBPRINT",
        "timestampUrl": "http://timestamp.digicert.com"
      },
      "macOS": {
        "signingIdentity": "Developer ID Application: Company Name",
        "providerShortName": "PROVIDER_ID"
      }
    }
  }
}
```

#### Distribution Channels
- **GitHub Releases**: Primary distribution channel with automated builds
- **Package Managers**: Homebrew (macOS), winget (Windows), APT/RPM repos (Linux)
- **Update Server**: JSON manifest with signed update packages

#### Success Criteria
- Installers work without administrator privileges where possible
- Automatic updates function reliably across all platforms
- No false positive antivirus warnings due to proper code signing
- Uninstall process removes all application data cleanly

---

### Phase 10: Beta Release & Feedback (4-6 weeks)
**Objective**: Community validation and bug fixes

#### Deliverables
- [x] **Public Beta**: Stable beta release for community testing
- [x] **Documentation**: Complete user manual and developer guides
- [x] **Bug Tracking**: Issue tracker with triage and prioritization
- [x] **Performance Testing**: Comprehensive benchmarks and stress tests
- [x] **Security Audit**: External security review of critical components

#### Beta Testing Focus Areas
1. **Cross-platform compatibility**: Test on various OS versions and hardware
2. **Large file handling**: Operations with multi-GB files and archives
3. **Network reliability**: Remote file operations under various conditions
4. **Plugin ecosystem**: Third-party plugin compatibility and performance
5. **Accessibility**: Screen reader and keyboard-only navigation testing

#### Success Criteria
- Stable operation for 99% of beta users
- Performance meets or exceeds targets on supported hardware
- Security vulnerabilities identified and addressed
- Community feedback incorporated into final release planning

---

### Phase 11: Version 1.0 Launch (2-3 weeks)
**Objective**: Production release with full feature set

#### Deliverables
- [x] **Stable Release**: Production-ready version 1.0
- [x] **Marketing Materials**: Screenshots, feature comparisons, website
- [x] **Release Documentation**: Changelog, migration guides, known issues
- [x] **Community Resources**: Forums, Discord server, contribution guidelines

#### Post-Launch Support
- **Hotfix Releases**: Critical bug fixes within 24-48 hours
- **Regular Updates**: Monthly feature releases with community feedback
- **Long-term Support**: Security updates for at least 2 years

---

## Post-1.0 Roadmap

### Future Enhancements (Months 6-12)

#### Advanced Features
- **Cloud Integration**: Google Drive, Dropbox, OneDrive, AWS S3 protocols
- **Git Integration**: Repository status, diff viewing, basic Git operations
- **Duplicate Finder**: Intelligent duplicate file detection and removal
- **Batch Operations**: Advanced file renaming, metadata editing
- **Scripting Support**: Lua or JavaScript automation scripting

#### Performance Optimization  
- **Index-based Search**: Optional file indexing for instant search results
- **Lazy Loading**: Virtual scrolling and on-demand file metadata loading
- **Memory Optimization**: Reduce memory footprint for large directories
- **GPU Acceleration**: Hardware-accelerated image thumbnails and operations

#### Enterprise Features
- **Audit Logging**: Comprehensive operation logging for compliance
- **Policy Enforcement**: Configurable restrictions on file operations
- **LDAP Integration**: Enterprise authentication and authorization
- **Remote Management**: Centralized configuration deployment

### Long-term Vision (Years 2-3)

#### Mobile Support
- **Tauri Mobile**: iOS and Android applications with core functionality
- **Remote Access**: Mobile client for accessing desktop file manager
- **Cloud Sync**: Synchronization between desktop and mobile versions

#### AI Integration
- **Smart Organization**: AI-powered file categorization and tagging
- **Intelligent Search**: Natural language search queries
- **Duplicate Detection**: Advanced similarity detection beyond exact matches
- **Workflow Automation**: Learn user patterns and suggest optimizations

---

## Risk Mitigation

### Technical Risks
- **Plugin Stability**: Extensive testing and sandboxing prevent plugin crashes
- **Performance Degradation**: Regular benchmarking and performance regression testing
- **Security Vulnerabilities**: Security-first development and external audits

### Market Risks  
- **Competition**: Focus on unique features (performance, extensibility, cross-platform)
- **User Adoption**: Early beta program and community engagement
- **Platform Changes**: Close monitoring of Tauri and Rust ecosystem updates

### Resource Risks
- **Development Time**: Conservative estimates with buffer time for each phase
- **Code Signing Costs**: Budget allocated for certificates and HSM requirements
- **Community Support**: Plan for scaling documentation and support resources

---

This roadmap provides a structured path from initial prototype to mature, production-ready file manager while maintaining flexibility for community feedback and emerging requirements.