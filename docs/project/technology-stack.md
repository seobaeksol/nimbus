# Technology Stack

_Last updated: 2025-08-27_


## Core Framework

### Tauri v2.8.4
**Role**: Application framework providing native backend with web frontend
**Justification**: 
- Smaller memory footprint compared to Electron
- Native performance for file operations
- Strong security model with IPC isolation
- Cross-platform binary distribution
- Rust ecosystem integration

**Key Features Used**:
- Multi-window support for detached viewers
- Secure IPC commands and events
- Auto-updater for seamless updates
- Native system integration
- Code signing and notarization support
- Mobile support (iOS/Android) in development

## Backend Technologies

### Rust (Edition 2021)
**Role**: Primary backend language for core logic
**Advantages**:
- Memory safety without garbage collection
- Excellent performance for I/O operations
- Rich ecosystem for file handling
- Cross-platform compilation
- Strong type system prevents common bugs

### Core Dependencies

#### File System Operations
```toml
[dependencies]
# Core async runtime
tokio = { version = "1.42", features = ["full"] }

# File system operations and watching
notify = "7.0.0"           # File system notifications
walkdir = "2.5.0"          # Directory traversal
jwalk = "0.8.1"            # Parallel directory walking

# Path manipulation
path-clean = "1.0"       # Path canonicalization
dunce = "1.0"            # Windows UNC path handling
```

#### Archive Support
```toml
# Archive handling
zip = { version = "2.2", features = ["deflate", "bzip2"] }
tar = "0.4"              # TAR archive support
flate2 = "1.1.2"           # Gzip compression
bzip2 = "0.6.0"            # Bzip2 compression
sevenz-rust = "0.7.0"      # 7zip support
unrar = "0.5.8"            # RAR decompression
libarchive3-sys = "0.2"  # Universal archive support
```

#### Network Protocols
```toml
# Remote file access
ssh2 = "0.9.5"             # SFTP support via libssh2
async-ftp = "6.0"        # Async FTP client
reqwest = { version = "0.12", features = ["stream"] }  # HTTP/WebDAV
```

#### Search & Indexing
```toml
# Search optimization
rayon = "1.10.0"           # Data parallelism
regex = "1.11.1"           # Pattern matching
ignore = "0.4.23"           # Gitignore-style filtering
tantivy = "0.25.0"         # Full-text search (optional)
```

#### Plugin System
```toml
# Dynamic plugin loading
libloading = "0.8"       # Dynamic library loading
abi_stable = "0.11"      # Stable Rust ABI
wasmtime = "26.0"        # WebAssembly runtime (optional)
serde = { version = "1.0", features = ["derive"] }
```

## Frontend Technologies

### React 19
**Role**: UI framework for component-based interface
**Features Used**:
- Concurrent features for responsive UI
- Suspense for loading states
- Context API for state management
- Hooks for component logic
- React 19 features (when stable)

### TypeScript 5.9
**Role**: Type-safe JavaScript for frontend development
**Benefits**:
- Strong typing for IPC communication
- Better IDE support and refactoring
- Compile-time error catching
- Enhanced code documentation

### UI Dependencies

#### Core UI Framework
```json
{
  "dependencies": {
    "react": "^19.1.1",
    "react-dom": "^19.1.1",
    
    // State management
    "@reduxjs/toolkit": "^2.8.2",
    "react-redux": "^9.2.0",
    
    // Routing (if needed for settings/help pages)
    "react-router-dom": "^7.8.2",
    
    // UI components and styling
    "styled-components": "^6.1.16",
    "@emotion/react": "^11.14.0",
    "framer-motion": "^12.23.12"
  }
}
```

#### File Management UI
```json
{
  "dependencies": {
    // Virtual scrolling for large file lists
    "react-window": "^1.8.8",
    "react-window-infinite-loader": "^1.0.9",
    
    // Table/grid components
    "react-table": "^7.8.0",
    "@tanstack/react-table": "^8.21.3",
    
    // Drag and drop
    "react-dnd": "^16.0.1",
    "react-dnd-html5-backend": "^16.0.1",
    
    // Context menus
    "react-contextmenu": "^2.14.0",
    
    // Icons
    "lucide-react": "^0.542.0",
    "@tabler/icons-react": "^3.34.1"
  }
}
```

#### Code Viewing & Editing
```json
{
  "dependencies": {
    // Text viewer with syntax highlighting
    "monaco-editor": "^0.52.2",
    "@monaco-editor/react": "^4.7.0",
    
    // Alternative lighter syntax highlighting
    "prismjs": "^1.29.0",
    "react-syntax-highlighter": "^15.5.0",
    
    // Hex viewer
    "hex-viewer": "^1.0.0"
  }
}
```

### Build Tools

#### Vite
**Role**: Frontend build tool and dev server
**Features**:
- Fast HMR during development
- Optimized production builds
- TypeScript support out of the box
- Plugin ecosystem

#### Configuration
```json
{
  "devDependencies": {
    "@vitejs/plugin-react": "^4.3.0",
    "vite": "^7.1.3",
    "typescript": "^5.9.2",
    
    // Testing
    "@testing-library/react": "^14.0.0",
    "@testing-library/jest-dom": "^5.16.0",
    "vitest": "^3.2.4",
    
    // Linting
    "@typescript-eslint/eslint-plugin": "^5.0.0",
    "@typescript-eslint/parser": "^5.0.0",
    "eslint": "^8.0.0",
    "eslint-plugin-react": "^7.32.0",
    
    // Formatting
    "prettier": "^2.8.0"
  }
}
```

## Development Tools

### Code Quality

#### Rust Tools
```toml
[dev-dependencies]
# Testing
tokio-test = "0.4"       # Async testing utilities
tempfile = "3.0"         # Temporary file creation
mockall = "0.11"         # Mocking framework

# Benchmarking
criterion = "0.4"        # Performance benchmarking

# Code quality
clippy = "0.1"           # Linting
rustfmt = "1.0"          # Code formatting
```

#### Cross-Platform Testing
```yaml
# GitHub Actions matrix
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, beta]
```

### Performance Profiling

#### Rust Profiling
- **CPU**: `cargo flamegraph` for flamegraph generation
- **Memory**: `valgrind` on Linux, `heaptrack` for heap profiling
- **Async**: `tokio-console` for async task monitoring

#### Frontend Profiling
- **React DevTools**: Component performance analysis
- **Chrome DevTools**: Memory and performance profiling
- **Bundle Analysis**: `webpack-bundle-analyzer` for bundle size optimization

## Platform-Specific Dependencies

### Windows
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winuser", "shellapi"] }
windows = "0.58.0"
```
**Features**:
- File association handling
- Shell integration (context menus, thumbnails)
- Windows-specific file attributes

### macOS
```toml
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.24"
objc = "0.2"
```
**Features**:
- macOS file system metadata
- Spotlight integration
- Apple script automation support

### Linux
```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"
nix = "0.29.0"
```
**Features**:
- Extended file attributes
- Desktop integration (D-Bus)
- Package manager integration

## Security Dependencies

### Cryptography & Security
```toml
[dependencies]
# Cryptographic operations
ring = "0.17.14"            # Cryptographic primitives
rustls = "0.23.12"          # TLS implementation
webpki-roots = "0.26.6"    # Root certificates

# Secure storage
keyring = "2.3.1"          # OS keychain access
secrecy = "0.10.3"          # Secret management
```

### Code Signing Tools
- **Windows**: SignTool.exe with code signing certificate
- **macOS**: Xcode command line tools for signing and notarization  
- **Cross-platform**: `tauri-bundler` with signing configuration

## Testing Framework

### Backend Testing
```toml
[dev-dependencies]
# Unit testing
tokio-test = "0.4"
tempfile = "3.0"
assert_fs = "1.0"        # File system testing utilities
predicates = "2.1"       # Assertion helpers

# Integration testing
testcontainers = "0.16.7"  # Container-based testing
wiremock = "0.6.0"         # HTTP mocking
```

### Frontend Testing
```json
{
  "devDependencies": {
    // Unit and integration testing
    "@testing-library/react": "^14.0.0",
    "@testing-library/user-event": "^14.4.0",
    "@testing-library/jest-dom": "^5.16.0",
    
    // End-to-end testing
    "@playwright/test": "^1.55.0",
    
    // Testing utilities
    "vitest": "^3.2.4",
    "@vitest/ui": "^0.28.0"
  }
}
```

## Distribution & Packaging

### Tauri Bundler Configuration
```json
{
  "tauri": {
    "bundle": {
      "targets": ["msi", "nsis", "deb", "rpm", "dmg", "app"]
    }
  }
}
```

### CI/CD Pipeline
```yaml
# Build matrix for all platforms
name: Build and Release
on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        platform: [ubuntu-20.04, windows-latest, macos-latest]
    runs-on: ${{ matrix.platform }}
```

## Documentation Tools

### API Documentation
```toml
[dependencies]
# Documentation generation
cargo-doc = "0.1"
mdbook = "0.4"           # Documentation book generation
```

### Architecture Documentation
- **Mermaid**: Diagram generation for architecture docs
- **PlantUML**: UML diagrams for detailed design
- **Docusaurus**: Documentation website generation

## Performance Characteristics

### Memory Usage Targets
- **Idle**: < 50MB RAM usage
- **Large Directory**: < 200MB for 100K+ files
- **Archive Browsing**: < 100MB for typical archives
- **Search**: < 150MB during active search operations

### Performance Benchmarks
- **Directory Listing**: < 100ms for 10K files
- **Search**: < 2 seconds for 1M+ files on SSD
- **Archive Extraction**: Competitive with 7-Zip
- **Remote Operations**: Efficient progress tracking and cancellation

### Bundle Size Targets
- **Windows**: < 15MB installer
- **macOS**: < 20MB DMG
- **Linux**: < 12MB AppImage

---

This technology stack provides a solid foundation for building a performant, secure, and maintainable cross-platform file manager while allowing for future extensibility through the plugin system.