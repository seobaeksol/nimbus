# Phase 3 Implementation Plan: Advanced Features & Polish

## Overview

Phase 3 builds upon the solid foundation of Phase 2 to implement advanced file manager features that differentiate Nimbus from basic file managers. This phase focuses on remote file system support, additional archive formats, built-in file viewers, and initial plugin system foundations.

## Phase 2 Completion Analysis ✅

### Successfully Implemented in Phase 2
- **✅ Task 1**: Complete file operations with copy_dir_recursive fix
- **✅ Task 2**: Parallel search engine with jwalk and rayon, streaming results
- **✅ Task 3**: ZIP archive support with async operations and progress tracking  
- **✅ Task 4**: Enhanced UI components (file icons, progress indicators, archive browser)
- **✅ Task 5**: Core engine improvements (file watching, performance optimization)

### Technical Assets Available for Phase 3
- **Core Engine**: Robust with performance optimization and file watching
- **Archive Support**: ZIP format working, foundation for additional formats
- **Search Engine**: High-performance parallel search with streaming
- **UI Components**: Professional progress indicators and file browsing
- **Performance Optimization**: Intelligent caching and concurrency control

## Phase 3 Strategic Objectives

### Primary Goals
1. **Remote File System Support**: SFTP, FTP, WebDAV protocols
2. **Extended Archive Formats**: 7z, RAR, TAR support beyond ZIP
3. **Built-in File Viewers**: Text, image, hex viewers without external apps
4. **Plugin System Foundation**: Basic plugin architecture and SDK
5. **Advanced UI Polish**: Themes, drag-drop, keyboard shortcuts

### Success Metrics
- **Performance**: Remote operations responsive (<2s for directory listings)
- **Usability**: All common archive formats supported with seamless browsing
- **Extensibility**: Plugin system enables third-party development
- **Polish**: Professional user experience matching commercial file managers

---

## Phase 3 Tasks

### Task 1: Remote File System Support (Priority: High)
**Estimated**: 4-5 days
**Dependencies**: Core engine, performance optimization

#### Objectives
- Implement SFTP protocol support with SSH key authentication
- Add FTP protocol support with passive/active modes
- Create WebDAV client for cloud storage integration
- Design unified remote file system interface
- Add secure credential storage using OS keychain

#### Technical Implementation
```rust
// Remote file system trait
#[async_trait]
pub trait RemoteFileSystem: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), RemoteError>;
    async fn disconnect(&mut self) -> Result<(), RemoteError>;
    async fn list_dir(&self, path: &RemotePath) -> Result<Vec<RemoteFileInfo>, RemoteError>;
    async fn download(&self, remote_path: &RemotePath, local_path: &Path, progress: ProgressCallback) -> Result<(), RemoteError>;
    async fn upload(&self, local_path: &Path, remote_path: &RemotePath, progress: ProgressCallback) -> Result<(), RemoteError>;
}

// SFTP implementation
pub struct SftpFileSystem {
    session: Option<Session>,
    sftp: Option<Sftp>,
    config: SftpConfig,
}

// Connection management
pub struct ConnectionManager {
    connections: HashMap<String, Box<dyn RemoteFileSystem>>,
    credentials: CredentialStore,
}
```

#### Deliverables
1. **SFTP Client**: Full SFTP support with SSH key and password authentication
2. **FTP Client**: Basic FTP with passive/active mode support
3. **WebDAV Client**: HTTP/HTTPS-based WebDAV for cloud storage
4. **Connection UI**: Dialog for managing remote connections
5. **Credential Storage**: Secure storage using OS keychain services
6. **Progress Tracking**: Real-time progress for upload/download operations

#### Success Criteria
- Establish connections to common SFTP/FTP servers
- Browse remote directories with same UI as local files
- Transfer files with progress indication and cancellation support
- Credential storage works securely on all target platforms
- Network interruptions handled gracefully with retry mechanisms

---

### Task 2: Extended Archive Format Support (Priority: High)
**Estimated**: 3-4 days
**Dependencies**: Archive foundation from Phase 2

#### Objectives
- Extend archive system beyond ZIP to support 7z, RAR, TAR formats
- Implement unified archive interface for seamless format switching
- Add archive creation capabilities for multiple formats
- Enhance extraction with advanced options (selective extraction, path mapping)
- Support password-protected archives across formats

#### Technical Implementation
```rust
// Extended archive support
pub enum ArchiveFormat {
    Zip,
    SevenZ,
    Tar,
    TarGz,
    TarBz2,
    Rar,
}

pub trait ArchiveHandler: Send + Sync {
    fn format(&self) -> ArchiveFormat;
    fn can_create(&self) -> bool;
    fn can_extract(&self) -> bool;
    fn supported_extensions(&self) -> Vec<&'static str>;
    
    async fn list_entries(&self, path: &Path) -> Result<Vec<ArchiveEntry>, ArchiveError>;
    async fn extract_entry(&self, archive: &Path, entry: &str, destination: &Path, options: &ExtractionOptions) -> Result<(), ArchiveError>;
    async fn create_archive(&self, files: &[PathBuf], archive_path: &Path, options: &CompressionOptions) -> Result<(), ArchiveError>;
}

// Format-specific implementations
pub struct SevenZHandler {
    // Using sevenz-rust or p7zip integration
}

pub struct RarHandler {
    // Using unrar crate for read-only support
}

pub struct TarHandler {
    // Using tar crate with compression support
}
```

#### Archive Format Support Matrix
| Format | Read | Write | Password | Compression Levels |
|--------|------|-------|----------|-------------------|
| ZIP    | ✅   | ✅    | ✅       | 0-9 (deflate)     |
| 7z     | ✅   | ✅    | ✅       | 0-9 (LZMA2)       |
| RAR    | ✅   | ❌    | ✅       | N/A (read-only)   |
| TAR    | ✅   | ✅    | ❌       | None/gzip/bzip2   |

#### Deliverables
1. **7z Support**: Read/write 7z archives with LZMA compression
2. **RAR Support**: Read-only RAR archive support
3. **TAR Support**: TAR with gzip/bzip2 compression variants
4. **Format Detection**: Automatic format detection from file headers
5. **Archive Creation UI**: Dialog for creating archives with format options
6. **Extraction Options**: Selective extraction, overwrite policies, path mapping

#### Success Criteria
- Browse large archives (>10K files) across all formats without performance issues
- Create archives with appropriate compression settings
- Extract with progress indication and cancellation
- Handle nested archives and password-protected files
- Maintain consistent UI experience across all formats

---

### Task 3: Built-in File Viewers (Priority: Medium)
**Estimated**: 3-4 days
**Dependencies**: UI components from Phase 2

#### Objectives
- Implement text viewer with syntax highlighting for code files
- Create image viewer supporting common formats (JPG, PNG, GIF, WebP, BMP)
- Add hex viewer for binary file inspection
- Design viewer window management system
- Add quick preview functionality (Space bar preview)

#### Technical Implementation
```typescript
// Viewer system architecture
interface FileViewer {
  name: string;
  supportedExtensions: string[];
  canHandle(fileInfo: FileInfo): boolean;
  render(filePath: string, container: HTMLElement): Promise<void>;
  dispose(): void;
}

// Text viewer with syntax highlighting
class TextViewer implements FileViewer {
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;
  
  async render(filePath: string, container: HTMLElement): Promise<void> {
    const content = await invoke<string>('read_text_file', { path: filePath });
    const language = this.detectLanguage(filePath);
    
    this.editor = monaco.editor.create(container, {
      value: content,
      language,
      readOnly: true,
      theme: 'vs-dark',
      automaticLayout: true,
    });
  }
}

// Image viewer with zoom and rotation
class ImageViewer implements FileViewer {
  private imageElement: HTMLImageElement | null = null;
  private zoomLevel = 1;
  
  async render(filePath: string, container: HTMLElement): Promise<void> {
    const imageUrl = convertFileSrc(filePath);
    // Implement zoom, pan, rotate controls
  }
}

// Hex viewer for binary files
class HexViewer implements FileViewer {
  private bytesPerRow = 16;
  
  async render(filePath: string, container: HTMLElement): Promise<void> {
    const data = await invoke<number[]>('read_hex_data', { 
      path: filePath, 
      offset: 0, 
      length: 8192 
    });
    // Render hex dump with ASCII sidebar
  }
}
```

#### Viewer Features
**Text Viewer**:
- Monaco Editor integration for syntax highlighting
- Support for 20+ programming languages
- Line numbers, code folding, search functionality
- Encoding detection (UTF-8, UTF-16, Latin-1)
- File size limits (10MB for performance)

**Image Viewer**:
- Zoom in/out with mouse wheel and buttons
- Fit to window, actual size, custom zoom levels
- Basic rotation (90°, 180°, 270°)
- Image metadata display (dimensions, format, file size)
- Navigation between images in directory

**Hex Viewer**:
- Configurable bytes per row (8, 16, 32)
- ASCII representation alongside hex values
- Search functionality within binary data
- Offset display and navigation
- Export selected bytes functionality

#### Deliverables
1. **Text Viewer**: Monaco-based editor with syntax highlighting
2. **Image Viewer**: Full-featured image display with zoom and metadata
3. **Hex Viewer**: Professional binary file inspection tool
4. **Viewer Manager**: System for opening appropriate viewer based on file type
5. **Quick Preview**: Space bar for instant file preview
6. **Viewer Windows**: Modal dialogs and detached window support

#### Success Criteria
- Text viewer handles large files efficiently (up to 100MB)
- Image viewer supports all common formats without external dependencies
- Hex viewer responsive for files up to 50MB
- Quick preview provides instant feedback (<200ms)
- Viewer windows integrate seamlessly with main application

---

### Task 4: Plugin System Foundation (Priority: Medium)
**Estimated**: 4-5 days
**Dependencies**: Core architecture, viewer system

#### Objectives
- Design stable plugin API interfaces for content, protocol, and viewer plugins
- Implement dynamic plugin loading and lifecycle management
- Create example plugins demonstrating each plugin type
- Add plugin manager UI for enable/disable functionality
- Establish plugin SDK with documentation and tooling

#### Technical Implementation
```rust
// Plugin system architecture
use libloading::{Library, Symbol};
use semver::Version;

#[repr(C)]
pub struct PluginMetadata {
    name: *const c_char,
    version: Version,
    description: *const c_char,
    author: *const c_char,
}

// Content plugin for custom file metadata
#[repr(C)]
pub struct ContentPlugin {
    metadata: PluginMetadata,
    supported_extensions: extern "C" fn() -> *const *const c_char,
    get_columns: extern "C" fn(&FileInfo) -> *const ColumnData,
    cleanup: extern "C" fn(),
}

// Protocol plugin for custom remote filesystems
#[repr(C)]
pub struct ProtocolPlugin {
    metadata: PluginMetadata,
    scheme: extern "C" fn() -> *const c_char,
    create_client: extern "C" fn(&ConnectionConfig) -> *mut c_void,
    cleanup: extern "C" fn(),
}

// Viewer plugin for custom file formats
#[repr(C)]
pub struct ViewerPlugin {
    metadata: PluginMetadata,
    supported_extensions: extern "C" fn() -> *const *const c_char,
    can_handle: extern "C" fn(&FileInfo) -> bool,
    render: extern "C" fn(&Path, *mut c_void) -> i32,
    cleanup: extern "C" fn(),
}

// Plugin manager
pub struct PluginManager {
    loaded_plugins: HashMap<String, LoadedPlugin>,
    plugin_directories: Vec<PathBuf>,
}

struct LoadedPlugin {
    library: Library,
    plugin_type: PluginType,
    metadata: PluginMetadata,
    enabled: bool,
}
```

#### Plugin Types and Examples

**Content Plugins**:
- **ExifPlugin**: Display EXIF metadata for photos
- **AudioMetadataPlugin**: Show ID3 tags for music files
- **VideoInfoPlugin**: Display video codec and resolution info

**Protocol Plugins**:
- **GitPlugin**: Browse Git repositories with status info
- **DatabasePlugin**: Connect to SQL databases as file-like interface
- **CloudStoragePlugin**: Generic cloud storage adapter

**Viewer Plugins**:
- **MarkdownViewer**: Render Markdown with live preview
- **PdfViewer**: PDF document viewing capabilities
- **VideoPreview**: Video thumbnail and metadata preview

#### Plugin SDK Structure
```
nimbus-plugin-sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Core plugin traits and utilities
│   ├── content.rs      # Content plugin helpers
│   ├── protocol.rs     # Protocol plugin helpers
│   ├── viewer.rs       # Viewer plugin helpers
│   └── examples/
│       ├── example_content.rs
│       ├── example_protocol.rs
│       └── example_viewer.rs
├── docs/
│   ├── plugin_guide.md
│   ├── api_reference.md
│   └── examples/
└── templates/
    ├── content_plugin_template/
    ├── protocol_plugin_template/
    └── viewer_plugin_template/
```

#### Deliverables
1. **Plugin API**: Stable C ABI for all plugin types
2. **Plugin Loader**: Dynamic loading with lifecycle management
3. **Example Plugins**: Working examples for each plugin type
4. **Plugin Manager UI**: Enable/disable plugins, view information
5. **Plugin SDK**: Complete SDK with documentation and templates
6. **Safety System**: Plugin sandboxing and resource limits

#### Success Criteria
- Load and unload plugins without application restart
- Plugin crashes don't affect main application stability
- Clear API separation between plugin and core functionality
- Comprehensive SDK enables third-party development
- Performance impact of plugin system <10% overhead

---

### Task 5: Advanced UI Polish & Features (Priority: Medium)
**Estimated**: 3-4 days
**Dependencies**: All previous tasks

#### Objectives
- Implement comprehensive drag and drop between panes and external applications
- Add complete keyboard shortcut system matching Total Commander
- Create theme system with light/dark modes
- Enhance context menus with dynamic content
- Improve accessibility for screen readers and keyboard navigation
- Add advanced file selection and multi-operation capabilities

#### Technical Implementation
```typescript
// Drag and drop system
interface DragDropManager {
  registerDropZone(element: HTMLElement, handlers: DropHandlers): void;
  startDrag(files: FileInfo[], source: PanelInfo): DragOperation;
  handleExternalDrop(files: File[], target: string): Promise<void>;
}

// Keyboard shortcut system
interface KeyboardShortcuts {
  'F5': () => void;     // Copy
  'F6': () => void;     // Move
  'F8': () => void;     // Delete
  'F3': () => void;     // View
  'Tab': () => void;    // Switch pane
  'Ctrl+T': () => void; // New tab
  'Ctrl+W': () => void; // Close tab
  'Ctrl+F': () => void; // Search
  'Space': () => void;  // Quick preview
  'Enter': () => void;  // Open/Navigate
  'Backspace': () => void; // Parent directory
  'Ctrl+A': () => void; // Select all
  'Ctrl+D': () => void; // Deselect all
  'Insert': () => void; // Toggle selection
  'Ctrl+Plus': () => void; // Select by pattern
  'Ctrl+Minus': () => void; // Deselect by pattern
}

// Theme system
interface ThemeSystem {
  currentTheme: 'light' | 'dark' | 'system';
  applyTheme(theme: string): void;
  registerCustomTheme(theme: ThemeDefinition): void;
  getSystemPreference(): 'light' | 'dark';
}

interface ThemeDefinition {
  name: string;
  colors: {
    background: string;
    foreground: string;
    accent: string;
    border: string;
    selection: string;
    hover: string;
  };
  fonts: {
    ui: string;
    code: string;
  };
}
```

#### UI/UX Enhancements

**Drag and Drop**:
- File operations between panes (copy, move)
- External application integration (drag to other apps)
- Multiple file selection with visual feedback
- Drop zones with highlight indication
- Progress indication for large operations

**Keyboard Navigation**:
- Complete Total Commander shortcut compatibility
- Context-sensitive shortcuts based on selection
- Customizable shortcut mapping
- Accessibility compliance (ARIA labels, focus management)
- Screen reader support with meaningful announcements

**Theme System**:
- Light and dark themes with system preference detection
- High contrast mode support
- Customizable accent colors
- Font size and family preferences
- Icon theme selection

**Context Menus**:
- Dynamic menu items based on file selection
- Plugin-contributed menu items
- Separator groups for logical organization
- Keyboard shortcuts displayed in menus
- Recent actions and favorites

#### Accessibility Features
- **Screen Reader Support**: Proper ARIA labels and live regions
- **High Contrast**: Theme variant for visual accessibility
- **Keyboard Only**: Complete functionality without mouse
- **Focus Management**: Logical focus order and visible indicators
- **Customization**: Font sizes, color adjustments, motion reduction

#### Deliverables
1. **Drag & Drop System**: Full support between panes and external apps
2. **Keyboard Shortcuts**: Complete Total Commander-compatible shortcuts
3. **Theme System**: Light/dark themes with customization options
4. **Context Menus**: Dynamic, plugin-extensible context menus
5. **Accessibility**: WCAG 2.1 AA compliance
6. **Preferences UI**: Comprehensive settings dialog

#### Success Criteria
- Drag and drop works reliably across all platforms
- All common operations accessible via keyboard shortcuts
- Theme system provides consistent, professional appearance
- Accessibility tools work correctly with the application
- User preferences persist across application restarts

---

## Implementation Strategy

### Week 1: Remote & Archive Foundations
**Days 1-2**: Task 1 (Remote File System Support)
- Implement SFTP client with SSH authentication
- Add FTP support with passive/active modes
- Create connection management system
- Add secure credential storage

**Days 3**: Task 2 (Extended Archive Formats) - Part 1
- Implement 7z format support
- Add TAR format with compression variants
- Create unified archive interface

### Week 2: Viewers & Advanced Features
**Days 1**: Task 2 (Extended Archive Formats) - Part 2
- Add RAR read-only support
- Implement format auto-detection
- Complete archive creation UI

**Days 2-3**: Task 3 (Built-in File Viewers)
- Implement text viewer with Monaco editor
- Create image viewer with zoom capabilities
- Add hex viewer for binary files
- Implement quick preview system

**Day 4**: Task 4 (Plugin System Foundation)
- Design plugin API interfaces
- Implement plugin loader system
- Create example plugins

### Week 3: Polish & Integration
**Days 1-2**: Task 4 (Plugin System Foundation) - Complete
- Plugin manager UI
- SDK documentation and templates
- Plugin safety and sandboxing

**Days 2-3**: Task 5 (Advanced UI Polish)
- Drag and drop implementation
- Keyboard shortcut system
- Theme system with light/dark modes

**Day 4**: Integration and Testing
- End-to-end testing of all features
- Performance testing and optimization
- Documentation updates
- Bug fixes and polish

---

## Technical Architecture

### Backend Rust Enhancements
```rust
// Enhanced crate structure
src-tauri/crates/
├── core-engine/          # ✅ Complete with performance & watching
├── archive/              # ✅ ZIP, extend to 7z, RAR, TAR
├── search-engine/        # ✅ Complete parallel search
├── remote-fs/            # 🔧 Implement SFTP, FTP, WebDAV
├── plugin-sdk/           # 🔧 Complete plugin system
├── file-viewers/         # 🆕 New: Text, image, hex viewers
└── theme-system/         # 🆕 New: Theme management
```

### Frontend React Enhancements
```typescript
// Enhanced component structure
src/
├── components/
│   ├── viewers/          # 🆕 New: Text, image, hex viewers
│   ├── remote/           # 🆕 New: Remote connection UI
│   ├── plugins/          # 🆕 New: Plugin manager
│   ├── themes/           # 🆕 New: Theme system
│   └── common/           # ✅ Enhanced from Phase 2
├── services/
│   ├── remoteService.ts  # 🆕 New: Remote file operations
│   ├── pluginService.ts  # 🆕 New: Plugin management
│   ├── viewerService.ts  # 🆕 New: File viewer coordination
│   └── themeService.ts   # 🆕 New: Theme management
└── hooks/
    ├── useRemoteFS.ts    # 🆕 New: Remote file system hook
    ├── usePlugins.ts     # 🆕 New: Plugin management hook
    ├── useViewer.ts      # 🆕 New: File viewer hook
    └── useTheme.ts       # 🆕 New: Theme management hook
```

---

## Quality Gates & Testing

### Code Quality Requirements
- All new code must have corresponding tests (>85% coverage)
- ESLint v9 compliance with zero warnings
- TypeScript strict mode compliance
- Rust clippy compliance with zero warnings
- Plugin API stability verified with example implementations

### Performance Requirements
- **Remote Operations**: Directory listing <2s over reasonable network connections
- **Archive Operations**: Browse 10K+ files without UI lag
- **File Viewers**: Open files <500ms, scroll performance >30fps
- **Plugin System**: <10% performance overhead when plugins loaded
- **Theme Switching**: <100ms theme transition time

### Security Requirements
- **Credential Storage**: OS keychain integration, no plaintext passwords
- **Plugin Sandboxing**: Isolated execution with resource limits
- **Remote Connections**: Proper certificate validation, encrypted connections
- **Input Validation**: All user inputs sanitized and validated

### User Experience Requirements
- **Error Handling**: Clear, actionable error messages with recovery suggestions
- **Progress Feedback**: All operations >1s show progress indication
- **Accessibility**: WCAG 2.1 AA compliance verified
- **Platform Integration**: Native look and feel on each target platform

---

## Risk Mitigation

### Technical Risks
- **Plugin Stability**: Extensive testing, sandboxing, crash isolation
- **Remote Protocol Complexity**: Start with simple implementations, iterate
- **Performance with Large Archives**: Implement streaming and lazy loading
- **Cross-platform Consistency**: Regular testing on all target platforms

### Implementation Risks
- **API Stability**: Lock plugin API early, maintain backward compatibility
- **Scope Creep**: Focus on core functionality, defer advanced features
- **Integration Complexity**: Regular integration testing throughout development
- **Third-party Dependencies**: Careful evaluation, fallback strategies

---

## Success Metrics

### Functional Metrics
- All remote protocols connect to common servers successfully
- All archive formats handle real-world files correctly
- All file viewers handle common file types without issues
- Plugin system enables basic third-party development
- UI polish meets professional application standards

### Performance Metrics
- Remote operations responsive (<2s directory listings)
- Archive browsing smooth (>30fps scroll performance)
- File viewers efficient (<500ms open time, <100MB memory)
- Plugin system minimal overhead (<10% performance impact)
- Theme switching fast (<100ms transition time)

### Quality Metrics
- Test coverage >85% for all new functionality
- Zero critical security vulnerabilities
- Zero accessibility violations (WCAG 2.1 AA)
- User feedback >4.5/5 for new features in beta testing

---

## Deliverables

### Code Deliverables
1. **Remote File System**: Complete SFTP, FTP, WebDAV support with UI
2. **Extended Archives**: 7z, RAR, TAR support with creation capabilities
3. **File Viewers**: Professional text, image, and hex viewers
4. **Plugin System**: Working plugin API with SDK and examples
5. **UI Polish**: Themes, drag-drop, shortcuts, accessibility improvements

### Documentation Deliverables
1. **User Guide**: Remote connections, archive handling, file viewing
2. **Plugin Developer Guide**: Complete SDK documentation with examples
3. **API Documentation**: Updated Tauri commands and frontend APIs
4. **Theme Guide**: Theme creation and customization instructions

### Testing Deliverables
1. **Automated Tests**: Unit and integration tests for all new features
2. **Performance Benchmarks**: Baseline measurements for future optimization
3. **Security Assessment**: Review of credential handling and plugin safety
4. **Accessibility Audit**: WCAG compliance verification

---

## Phase 3 Success Criteria

Upon completion of Phase 3, Nimbus will have:

### Advanced Functionality ✨
- **Remote file system access** with secure authentication and reliable transfers
- **Comprehensive archive support** for all common formats with creation capabilities
- **Built-in file viewers** eliminating need for external applications
- **Plugin system foundation** enabling third-party extensibility
- **Professional UI polish** matching commercial file managers

### Technical Excellence 🔧
- **Robust architecture** with proper error handling and performance optimization
- **Security-first design** with encrypted connections and secure credential storage
- **Extensible plugin system** with safe loading and resource management
- **Accessibility compliance** supporting users with diverse needs
- **Cross-platform consistency** with native look and feel

### User Experience 🚀  
- **Seamless remote file management** feeling as natural as local operations
- **Unified archive experience** with consistent browsing and extraction
- **Instant file preview** without launching external applications
- **Professional polish** with themes, shortcuts, and intuitive interactions
- **Accessibility support** for keyboard-only and screen reader users

This positions Nimbus for Phase 4 development focusing on advanced features like cloud integration, Git support, and enterprise capabilities.

---

**Estimated Total Time**: 3 weeks (15 working days)
**Team Size**: 1 developer + Claude Code assistance  
**Success Probability**: High (building on proven Phase 2 architecture)
**Target Completion**: Ready for public beta testing

`★ Insight ─────────────────────────────────────`
**Strategic Phase 3 Vision**: This phase transforms Nimbus from a solid local file manager into a comprehensive file management solution with remote access, extensive archive support, and plugin extensibility. The focus on polish and user experience ensures the application will compete effectively with commercial solutions while maintaining the flexibility for future enhancement.
`─────────────────────────────────────────────────`