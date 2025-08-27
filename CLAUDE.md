# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Documentation Reference System

Use the `@` symbol to reference specific documentation sections quickly:

### Quick References
- `@setup` - Project setup and development workflow commands
- `@architecture` - Multi-process architecture and component structure  
- `@grid-layouts` - Grid layout system (2x2, 2x3, 3x2) and multi-panel interface
- `@features` - Core features: file management, archives, remote systems, search, viewers
- `@tech-stack` - Technology stack, dependencies, and build tools
- `@performance` - Performance targets and optimization guidelines
- `@security` - Security considerations and development guidelines
- `@testing` - Testing strategy for unit, integration, E2E, and performance tests
- `@patterns` - Common development patterns for Tauri commands, components, plugins

### Documentation Files
- `@docs/multi-panel.md` - Comprehensive multi-panel and grid layout documentation
- `@docs/architecture.md` - Detailed system architecture with TypeScript interfaces
- `@docs/api/tauri-commands.md` - Complete Tauri commands API reference
- `@docs/features/` - Individual feature documentation (archives, search, core-features)
- `@docs/project/` - Project overview, roadmap, technology stack
- `@docs/development/getting-started.md` - Detailed development setup guide

### Usage Examples
- "Reference @grid-layouts for panel arrangement options"
- "Check @docs/api/tauri-commands.md for file operation commands"
- "Follow @patterns for adding new Tauri commands"
- "See @performance for memory and response time targets"

## Project Overview

Nimbus is a cross-platform modern file manager inspired by Total Commander, built with **Tauri v2.8** (Rust backend) and **React 19.1.1** (TypeScript frontend). The project follows a multi-process architecture with secure IPC communication between frontend and backend processes.

**Current Status**: Implementation phase - full Tauri project with React 19 frontend and modular Rust backend architecture.

## @setup - Development Commands

### Project Setup
```bash
# Install frontend dependencies
npm install

# Install Rust backend dependencies
cd src-tauri && cargo build && cd ..

# Verify installation
npm run tauri info
```

### Development Workflow
```bash
# Start development server with hot reload
npm run tauri dev

# Alternative: separate frontend/backend development
npm run dev                    # Terminal 1: Frontend dev server
npm run tauri dev --no-watch   # Terminal 2: Tauri in dev mode
```

### Build Commands
```bash
# Debug build (faster, includes debug info)
npm run tauri build --debug

# Production build (optimized)
npm run tauri build

# Platform-specific builds
npm run tauri build --target x86_64-pc-windows-msvc
npm run tauri build --target x86_64-apple-darwin
npm run tauri build --target x86_64-unknown-linux-gnu
```

### Testing
```bash
# Frontend tests
npm test                    # Run Jest tests
npm test -- --coverage     # Run with coverage
npm test -- --watch        # Watch mode

# Backend tests (from src-tauri directory)
cd src-tauri
cargo test                  # All tests
cargo test -- --nocapture  # With output
cargo test files::          # Specific module
cargo test --test integration  # Integration tests
```

### Code Quality
```bash
# Frontend formatting and linting
npm run format              # Format with Prettier
npm run lint                # ESLint

# Backend formatting and linting (from src-tauri)
cd src-tauri
cargo fmt                   # Format Rust code
cargo clippy                # Rust linting
```

## @architecture - Architecture Overview

### Multi-Process Design
- **Frontend Process**: React UI in WebView with TypeScript
- **Backend Process**: Native Rust application handling file operations
- **Communication**: Secure Tauri IPC (JSON serialization, async commands)

### Backend Structure (Planned)
```
src-tauri/
├── src/
│   ├── main.rs              # Tauri app initialization
│   ├── commands/            # IPC command handlers
│   │   ├── files.rs         # File operations
│   │   ├── search.rs        # Search commands
│   │   └── archives.rs      # Archive handling
│   └── lib.rs               # Module integration
├── crates/                  # Specialized local crates
│   ├── core-engine/         # FileSystem trait, unified API
│   ├── archive/             # ZIP, TAR, 7z, RAR support
│   ├── remote-fs/           # FTP/SFTP protocol clients
│   ├── search-engine/       # Parallel search with jwalk/rayon
│   └── plugin-sdk/          # Dynamic plugin loading system
```

### Frontend Structure (Planned)
```
src/
├── components/              # Reusable React components
│   ├── FileList/           # File listing views
│   ├── Toolbar/            # Application toolbar
│   ├── Dialogs/            # Modal dialogs
│   └── common/             # Shared UI components
├── hooks/                  # Custom React hooks
├── services/               # IPC service layer
├── stores/                 # State management (Context + useReducer)
├── types/                  # TypeScript definitions
└── utils/                  # Utility functions
```

## Key Technical Patterns

### Tauri IPC Commands
All backend operations use async Tauri commands with type-safe interfaces:

```typescript
// Frontend service call
import { invoke } from '@tauri-apps/api/tauri';

export async function listDirectory(path: string): Promise<FileInfo[]> {
    return await invoke<FileInfo[]>('list_dir', { path });
}
```

```rust
// Backend command handler
#[tauri::command]
pub async fn list_dir(path: String) -> Result<Vec<FileInfo>, FileError> {
    // Implementation using unified FileSystem trait
}
```

### State Management Pattern
- **Context + useReducer** for global application state
- **Multi-panel interface** with grid layout support and independent tab states
- **Grid layouts**: 2x2, 2x3, and 3x2 arrangements for complex workflows
- **Real-time updates** via Tauri event system

### Plugin Architecture
- **Unified interfaces**: ContentPlugin, ProtocolPlugin, ViewerPlugin
- **Safe loading**: libloading with ABI stability
- **Sandboxing options**: Process isolation, WebAssembly runtime

## @features - Core Features (Planned)

### File Management
- Multi-panel tabbed interface with grid layout support (2x2, 2x3, 3x2 grids)
- Basic operations: copy, move, delete, rename with progress tracking
- Advanced operations: batch operations, secure deletion, integrity verification

### Archive Support
- Browse archives as virtual filesystems (ZIP, TAR, 7z, RAR)
- Extract with options: preserve paths, overwrite policies, password support
- Create archives with compression level control

### Remote File Systems
- FTP/SFTP protocol support with secure credential storage
- WebDAV integration for cloud storage
- Connection pooling and retry mechanisms

### Search Engine
- Parallel recursive search using jwalk + rayon
- Streaming results for immediate UI feedback
- Advanced filters: name patterns, content, size, date, file types

### File Viewers
- Text viewer with encoding detection and syntax highlighting
- Image viewer with EXIF data display
- Hex viewer for binary files
- Plugin-extensible viewer system

### @grid-layouts - Grid Layout System
- **2x2 Grid**: Four panels for development workflows (src, build, test, docs)
- **2x3 Grid**: Six panels for complex multi-site management
- **3x2 Grid**: Six panels optimized for version control workflows
- **Smart presets**: Developer, Content, System Admin, Media Production grids
- **Grid operations**: Broadcast copy, cascade move, synchronized navigation
- **Keyboard shortcuts**: Ctrl+G combinations for quick grid switching

## @tech-stack - Technology Stack

### Core Dependencies
**Backend (Rust)**:
- `tauri = "2.8"` - Application framework
- `tokio = "1.47"` - Async runtime
- `serde = "1.0"` - JSON serialization
- `notify = "8.2"` - File system watching
- `jwalk = "0.8.1"` - Parallel directory traversal
- `zip = "4.5"`, `tar = "0.4"` - Archive support
- `ssh2 = "0.9.5"` - SFTP client
- `rayon = "1.10.0"` - Data parallelism

**Frontend (TypeScript/React)**:
- `react = "^19.1.1"` - UI framework
- `@reduxjs/toolkit = "^2.8.2"` - State management
- `react-window = "^1.8.8"` - Virtual scrolling
- `@tanstack/react-table = "^8.21.3"` - Data tables
- `monaco-editor = "^0.52.2"` - Code editor

### Build Tools
- **Vite**: Frontend build tool with HMR
- **TypeScript 5.9.2**: Type safety
- **ESLint + Prettier**: Code quality
- **Vitest**: Frontend testing
- **Cargo**: Rust package management

## @performance - Performance Targets

### Memory Usage
- Idle: <50MB RAM
- Large directories (100K+ files): <200MB
- Archive browsing: <100MB
- Active search: <150MB

### Response Times
- Directory listing: <100ms for 10K files
- Search: <2s for 1M+ files on SSD
- File operations: Competitive with native tools

### Bundle Sizes
- Windows installer: <15MB
- macOS DMG: <20MB  
- Linux AppImage: <12MB

## @security - Development Guidelines

### Code Organization
- Use the **unified FileSystem trait** for all storage backends
- Implement **async operations** for non-blocking I/O
- Follow **error handling patterns** with structured error types
- Maintain **type safety** across IPC boundaries

### Security Considerations
- All file paths must be **canonicalized** to prevent directory traversal
- Use **secure credential storage** (OS keychain)
- Implement **input validation** at IPC boundaries
- Follow **principle of least privilege** for file system access

### @testing - Testing Strategy
- **Unit tests**: Core business logic (Rust) and components (React)
- **Integration tests**: IPC command flows and file operations
- **E2E tests**: Complete user workflows with Playwright
- **Performance tests**: Search operations and large file handling

## Development Environment

### Required Tools
- **Node.js 16+**: Frontend development
- **Rust 1.70+**: Backend development  
- **Git 2.25+**: Version control

### Platform-Specific Requirements
**Windows**: Visual Studio Build Tools, WebView2 Runtime
**macOS**: Xcode Command Line Tools
**Linux**: build-essential, WebKit2GTK, additional Tauri dependencies

### IDE Setup
**VS Code Extensions**:
- `rust-lang.rust-analyzer`
- `tauri-apps.tauri-vscode` 
- `bradlc.vscode-tailwindcss`
- `esbenp.prettier-vscode`

## @patterns - Common Development Patterns

### Adding New Tauri Commands
1. Define command in appropriate `src-tauri/src/commands/*.rs` file
2. Register in `main.rs` with `tauri::generate_handler!`
3. Add TypeScript types and service function in frontend
4. Add error handling and validation

### Adding New Components  
1. Create component directory with TypeScript files
2. Export from index.ts with proper typing
3. Add to appropriate parent component
4. Follow existing patterns for styling and state

### Plugin Development
1. Implement required trait (ContentPlugin, ProtocolPlugin, ViewerPlugin)
2. Use stable ABI with extern "C" interface
3. Register plugin with PluginManager
4. Test with dynamic loading

This documentation provides the essential context for working with Nimbus, a modern file manager that balances performance, security, and extensibility through its Tauri-based architecture.