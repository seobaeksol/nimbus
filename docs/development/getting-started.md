# Getting Started

## Prerequisites

### System Requirements

#### Minimum Requirements
- **OS**: Windows 10, macOS 10.15, or Linux (Ubuntu 18.04+ or equivalent)
- **RAM**: 4GB (8GB recommended)
- **Storage**: 500MB for development tools, 1GB for build artifacts
- **CPU**: x86_64 or ARM64 (Apple Silicon supported)

#### Development Environment
- **Node.js**: 18.x or later (for frontend build tools)
- **Rust**: 1.81.0 or later (latest stable recommended)
- **Git**: 2.25.0 or later

### Platform-Specific Setup

#### Windows
```powershell
# Install Rust via rustup
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Install Node.js (use winget or download from nodejs.org)
winget install OpenJS.NodeJS

# Install Git
winget install Git.Git

# Install Visual Studio Build Tools (required for some dependencies)
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

**Additional Windows Requirements**:
- WebView2 Runtime (usually pre-installed on Windows 11)
- Microsoft Visual C++ Build Tools or Visual Studio

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js via Homebrew
brew install node

# Install Git (if not already installed)
brew install git
```

#### Linux (Ubuntu/Debian)
```bash
# Update package list
sudo apt update

# Install system dependencies
sudo apt install -y curl wget git build-essential pkg-config libssl-dev

# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js via NodeSource repository
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs

# Install additional dependencies for Tauri
sudo apt install -y libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf
```

## Project Setup

### Clone Repository

```bash
# Clone the repository
git clone https://github.com/seobaeksol/nimbus.git
cd nimbus

# Verify the directory structure
ls -la
```

Expected directory structure:
```
nimbus/
├── src/                    # React frontend source
├── src-tauri/             # Rust backend source
├── public/                # Static assets
├── docs/                  # Documentation
├── package.json           # Frontend dependencies
├── package-lock.json      # Frontend lock file
├── vite.config.ts         # Vite configuration
├── tailwind.config.js     # Tailwind CSS config
├── tsconfig.json          # TypeScript configuration
└── README.md              # Project overview
```

### Install Dependencies

#### Frontend Dependencies
```bash
# Install Node.js dependencies
npm install

# Verify installation
npm list --depth=0
```

#### Backend Dependencies
```bash
# Navigate to Rust project
cd src-tauri

# Build Rust dependencies (this will take a while on first run)
cargo build

# Verify Rust toolchain
cargo --version
rustc --version

# Return to project root
cd ..
```

### Verify Installation

```bash
# Check Tauri CLI installation
npm run tauri -- --version

# Run basic health check
npm run tauri info
```

Expected output should show:
- Operating System information
- Rust version and targets
- Node.js and npm versions
- Tauri CLI version
- Available build targets

## Development Workflow

### Running in Development Mode

#### Start Development Server
```bash
# Start the development environment
npm run tauri dev

# Alternative: separate frontend and backend
# Terminal 1: Start frontend dev server
npm run dev

# Terminal 2: Start Tauri in dev mode (in another terminal)
npm run tauri dev --no-watch
```

The development server provides:
- **Hot Module Replacement**: Frontend changes reload instantly
- **Rust Hot Reload**: Rust changes trigger automatic recompilation
- **DevTools**: Browser developer tools available in the app window
- **Console Output**: Both frontend and backend logs in the terminal

#### Development Features
- **Debug Console**: Press F12 to open browser developer tools
- **Hot Reload**: Changes to source code automatically refresh the app
- **Error Overlay**: Build errors displayed directly in the app window
- **Network Inspection**: Monitor IPC calls between frontend and backend

### Building for Production

#### Debug Build
```bash
# Build development version (faster, larger, includes debug info)
npm run tauri build --debug
```

#### Release Build
```bash
# Build optimized production version
npm run tauri build

# Build for specific targets
npm run tauri build --target x86_64-pc-windows-msvc
npm run tauri build --target x86_64-apple-darwin
npm run tauri build --target x86_64-unknown-linux-gnu
```

### Code Organization

#### Frontend Structure
```typescript
src/
├── components/           # Reusable React components
│   ├── FileList/        # File listing components
│   ├── Toolbar/         # Toolbar components
│   ├── Dialogs/         # Modal dialogs
│   └── common/          # Shared UI components
├── hooks/               # Custom React hooks
├── services/            # API services and IPC calls
├── stores/              # State management
├── types/               # TypeScript type definitions
├── utils/               # Utility functions
├── styles/              # Global styles and themes
└── main.tsx             # Application entry point
```

#### Backend Structure
```rust
src-tauri/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── commands/        # Tauri command handlers
│   │   ├── mod.rs
│   │   ├── files.rs     # File operation commands
│   │   ├── search.rs    # Search commands
│   │   └── archives.rs  # Archive commands
│   ├── core/            # Core business logic
│   └── utils/           # Utility functions
├── crates/              # Local crates
│   ├── core-engine/     # File system abstraction
│   ├── archive/         # Archive handling
│   ├── search/          # Search engine
│   └── remote-fs/       # Remote file systems
├── icons/               # Application icons
└── Cargo.toml           # Rust dependencies
```

## IDE Setup

### VS Code (Recommended)

#### Required Extensions
```json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "tauri-apps.tauri-vscode",
        "bradlc.vscode-tailwindcss",
        "esbenp.prettier-vscode",
        "ms-vscode.vscode-typescript-next"
    ]
}
```

#### Settings Configuration
Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },
    "tailwindCSS.includeLanguages": {
        "rust": "html"
    }
}
```

#### Debug Configuration
Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Tauri Development Debug",
            "type": "node",
            "request": "launch",
            "cwd": "${workspaceFolder}",
            "program": "${workspaceFolder}/node_modules/.bin/tauri",
            "args": ["dev"]
        },
        {
            "name": "Tauri Rust Debug", 
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/src-tauri/target/debug/nimbus",
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### Alternative IDEs

#### Rust Development
- **RustRover**: JetBrains' Rust IDE with excellent debugging support
- **Vim/Neovim**: With rust-analyzer LSP support
- **Emacs**: With rust-mode and LSP support

#### Frontend Development  
- **WebStorm**: JetBrains IDE with excellent TypeScript support
- **Sublime Text**: With TypeScript and Prettier plugins
- **Atom**: With language-rust and ide-typescript packages

## Common Development Tasks

### Adding New Tauri Commands

1. **Define the command in Rust**:
```rust
// src-tauri/src/commands/files.rs
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    // Implementation
}
```

2. **Register the command**:
```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::files::get_file_info,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. **Call from frontend**:
```typescript
// src/services/fileService.ts
import { invoke } from '@tauri-apps/api/tauri';

export async function getFileInfo(path: string): Promise<FileInfo> {
    return await invoke('get_file_info', { path });
}
```

### Adding New React Components

1. **Create component directory**:
```bash
mkdir src/components/NewComponent
```

2. **Create component files**:
```typescript
// src/components/NewComponent/NewComponent.tsx
import React from 'react';
import { NewComponentProps } from './types';
import './NewComponent.css';

export const NewComponent: React.FC<NewComponentProps> = ({ 
    // props 
}) => {
    return (
        <div className="new-component">
            {/* component content */}
        </div>
    );
};
```

3. **Export component**:
```typescript
// src/components/NewComponent/index.ts
export { NewComponent } from './NewComponent';
export type { NewComponentProps } from './types';
```

### Running Tests

#### Frontend Tests
```bash
# Run Jest tests
npm test

# Run tests with coverage
npm test -- --coverage

# Run tests in watch mode
npm test -- --watch
```

#### Backend Tests
```bash
# Navigate to Rust project
cd src-tauri

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test files::

# Run integration tests
cargo test --test integration
```

### Linting and Formatting

#### Automatic Formatting
```bash
# Format frontend code
npm run format

# Format Rust code
cd src-tauri && cargo fmt

# Lint frontend code
npm run lint

# Lint Rust code  
cd src-tauri && cargo clippy
```

### Performance Profiling

#### Frontend Profiling
```bash
# Build with profiling enabled
npm run build -- --mode=profiling

# Analyze bundle size
npm run analyze
```

#### Backend Profiling
```bash
cd src-tauri

# Build with profiling symbols
cargo build --release --profile=profiling

# Run with profiler (Linux)
perf record --call-graph=dwarf target/release/nimbus
perf report

# Memory profiling with Valgrind
valgrind --tool=massif target/release/nimbus
```

## Troubleshooting

### Common Issues

#### Build Failures

**Issue**: Rust compilation errors
```bash
error: linking with `cc` failed: exit status: 1
```
**Solution**: Install build tools:
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools  
- Linux: build-essential package

**Issue**: Node.js version mismatch
```bash
error: Unsupported engine: wanted: {"node":">=18.0.0"}
```
**Solution**: Update Node.js to version 16 or later

#### Runtime Issues

**Issue**: WebView2 not found (Windows)
**Solution**: Install WebView2 Runtime from Microsoft

**Issue**: Permission denied (Linux)
**Solution**: Ensure user is in appropriate groups:
```bash
sudo usermod -a -G dialout,plugdev $USER
```

#### Development Server Issues

**Issue**: Hot reload not working
**Solution**: Check file watchers limit on Linux:
```bash
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Getting Help

1. **Check documentation**: Browse the docs/ directory
2. **Search issues**: Check GitHub issues for similar problems  
3. **Community support**: Join our Discord server
4. **Create issue**: Submit detailed bug reports with reproduction steps

---

This guide should get you up and running with Nimbus development. The next step is to explore the [Architecture Documentation](../architecture/) to understand the codebase structure in detail.
