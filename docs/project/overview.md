# Project Overview

## Vision

Nimbus is a cross-platform modern file manager inspired by Total Commander, built using Tauri (Rust backend, WebView UI) with a React frontend. The application emphasizes modularity, performance, and maintainability to support long-term development and community contributions.

## Core Goals

### Performance & Responsiveness
- **Native Speed**: Leverage Rust's performance for heavy operations (file I/O, network, compression)
- **Responsive UI**: Keep the React interface responsive through efficient IPC communication
- **Optimized Search**: Outperform default OS searches using multithreaded Rust implementations

### Cross-Platform Compatibility
- **Universal Design**: Single codebase targeting Windows, macOS, and Linux
- **Native Integration**: Platform-specific installers, code signing, and OS conventions
- **Consistent UX**: Uniform experience across all supported platforms

### Extensibility & Modularity
- **Plugin Architecture**: Support for content, protocol, and viewer plugins
- **Modular Backend**: Clean separation of concerns through Rust crates
- **Community-Driven**: Enable third-party contributions without core modifications

## Key Features

### Dual-Pane Interface
- **Side-by-side Views**: Two independent file panes for efficient file operations
- **Multi-tab Support**: Multiple tabs per pane, similar to modern web browsers
- **Flexible Layout**: Resizable panes with optional single-pane mode
- **Keyboard Navigation**: Full keyboard support mirroring Total Commander conventions

### Archive Handling
- **Virtual Filesystem**: Browse archives (ZIP, 7z, TAR, RAR) like directories
- **Seamless Operations**: Copy files in/out of archives transparently
- **Format Support**: 
  - ZIP: Full read/write support via Rust `zip` crate
  - TAR (gz, bz2): Support via `tar`, `flate2`, `bzip2` crates
  - 7z & RAR: Support via libarchive bindings or external tools

### Remote File Access
- **Protocol Support**: FTP and SFTP with extensible design for additional protocols
- **Secure Authentication**: Support for password, key-based, and anonymous connections
- **Transparent Operations**: Remote files behave like local files in the interface
- **Connection Management**: Save and reuse connection configurations

### Integrated Viewers
- **Text Viewer**: UTF-8 text files with optional syntax highlighting
- **Image Viewer**: Common image formats (JPG, PNG, GIF) with basic controls
- **Hex Viewer**: Binary file inspection with ASCII sidebar
- **Extensible**: Plugin system allows additional viewer types

### Powerful Search
- **Recursive Search**: Fast directory tree traversal using parallel processing
- **Multiple Criteria**: Search by filename, extension, content, size, and date
- **Streaming Results**: Real-time result display as files are found
- **Performance Focus**: Optimized to handle large directory structures

### Plugin System
- **Content Plugins**: Add custom file metadata columns (duration, EXIF data, etc.)
- **Protocol Plugins**: Support new remote protocols (WebDAV, S3, SMB, etc.)
- **Viewer Plugins**: Handle additional file formats
- **Safe Loading**: Dynamic library loading with versioning and sandboxing options

## Target Audience

### Power Users
- Users comfortable with keyboard shortcuts and efficient workflows
- Those who manage large file collections or work with archives regularly
- Users requiring remote file access and advanced search capabilities

### Developers
- Need efficient file management for development workflows
- Require extensibility through plugins for specialized file types
- Value performance and cross-platform consistency

### System Administrators
- Manage files across multiple systems and protocols
- Need reliable tools for file operations and remote access
- Require security features and audit capabilities

## Technical Philosophy

### Rust-First Backend
- **Memory Safety**: Eliminate common file manager vulnerabilities
- **Performance**: Native speed for I/O operations and data processing
- **Concurrency**: Safe parallel processing for search and file operations
- **Cross-platform**: Single codebase with platform-specific optimizations

### Modern Web Frontend
- **React Ecosystem**: Leverage mature UI component libraries
- **Responsive Design**: CSS Grid/Flexbox for flexible layouts
- **Accessibility**: WCAG compliance and keyboard navigation
- **Theming**: Support for light/dark themes and customization

### Clean Architecture
- **Separation of Concerns**: Clear boundaries between UI, business logic, and I/O
- **Testability**: Modular design enables comprehensive testing
- **Maintainability**: Well-documented interfaces and consistent patterns
- **Extensibility**: Plugin system with stable APIs

## Success Metrics

### Performance Benchmarks
- **Search Speed**: Faster than native OS search on large directories
- **File Operations**: Competitive with native file managers
- **Memory Usage**: Efficient memory footprint under various workloads
- **Startup Time**: Quick application launch and directory loading

### User Experience
- **Accessibility**: WCAG 2.1 AA compliance
- **Keyboard Efficiency**: Complete keyboard navigation support
- **Visual Clarity**: Clear information hierarchy and intuitive icons
- **Error Handling**: Graceful error recovery with informative messages

### Developer Experience
- **Plugin Development**: Clear SDK with comprehensive examples
- **API Stability**: Versioned plugin interfaces with migration guides
- **Documentation**: Complete API reference and architectural guides
- **Community**: Active contributor base with issue response times

## Project Inspiration

### Total Commander
- Dual-pane interface with keyboard shortcuts
- Extensive plugin ecosystem
- Power user focus with advanced features

### Modern File Managers
- **muCommander**: Cross-platform Java-based file manager
- **CoDriver**: Rust-based file manager with performance focus
- **FreeCommander**: Windows file manager with rich features

## Long-term Vision

### Community Ecosystem
- **Plugin Marketplace**: Curated collection of community plugins
- **Extension Points**: Well-defined APIs for deep customization
- **Documentation Hub**: Comprehensive guides for users and developers

### Platform Evolution
- **Mobile Support**: Potential Tauri mobile support integration
- **Cloud Integration**: Enhanced cloud storage protocol support
- **AI Features**: Intelligent file organization and search suggestions

### Enterprise Features
- **Security Auditing**: File operation logging and compliance features
- **Centralized Configuration**: Deploy configurations across organizations
- **Integration APIs**: Embed file manager functionality in other applications

---

This overview establishes the foundation for Nimbus as a modern, performant, and extensible file manager that serves both individual power users and development communities seeking a reliable cross-platform solution.