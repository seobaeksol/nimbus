# Troubleshooting Guide

## Common Development Issues

### Build and Compilation Errors

#### TypeScript Compilation Errors
**Symptoms**: `tsc` build fails, type errors in IDE
```bash
# Check for type errors
npm run build

# Common solutions
npm install --save-dev @types/node    # Missing type definitions
rm -rf node_modules && npm install    # Corrupted dependencies
```

#### Rust Compilation Errors
**Symptoms**: `cargo build` fails, Rust errors
```bash
# Check compilation
cd src-tauri && cargo check

# Common solutions
cargo clean && cargo build            # Clear build cache
rustup update                         # Update Rust toolchain
```

#### Missing Dependencies
**Symptoms**: Module not found errors
```bash
# Frontend dependencies
npm install --save missing-package

# Rust dependencies  
cd src-tauri && cargo add missing-crate

# Tauri system dependencies (Linux)
sudo apt install webkit2gtk-4.0-dev build-essential
```

### Testing Issues

#### Frontend Test Failures
**Symptoms**: Vitest tests fail, component errors
```bash
# Run with verbose output
npm test -- --reporter=verbose

# Common issues:
# 1. Mock setup - check src/test/setup.ts
# 2. Component imports - verify paths
# 3. Provider wrapping - check testUtils.tsx
```

#### Backend Test Failures  
**Symptoms**: `cargo test` failures
```bash
# Run with output
cd src-tauri && cargo test -- --nocapture

# Common issues:
# 1. File permissions - tests create temp files
# 2. Async timing - use proper await patterns  
# 3. Test isolation - clean up test data
```

#### Test Environment Issues
```bash
# Reset test environment
rm -rf coverage/ .nyc_output/
npm test -- --run

# Mock issues - check these files:
# - src/test/setup.ts (global mocks)
# - src/test/mocks/ (service mocks)
# - vitest.config.ts (configuration)
```

### Development Server Issues

#### Port Already in Use
**Symptoms**: `Port 1420 is already in use`
```bash
# Kill existing processes
lsof -ti:1420 | xargs kill -9
pkill -f "vite"
pkill -f "tauri"

# Use different port
npm run dev -- --port 3001
```

#### Hot Reload Not Working
**Symptoms**: Changes don't reflect, manual refresh needed
```bash
# Check file watching
npm run tauri dev --verbose

# Solutions:
# 1. Restart dev server
# 2. Clear browser cache
# 3. Check file permissions
# 4. Disable browser extensions
```

#### WebView Issues
**Symptoms**: Application window blank, console errors
```bash
# Check WebView installation
npm run tauri info

# macOS - install Xcode Command Line Tools
xcode-select --install

# Windows - install WebView2 Runtime
# Linux - install webkit2gtk development package
```

### Font and Asset Issues

#### Fonts Not Loading
**Symptoms**: Default fonts used, no Nerd Font icons
```bash
# Check font files exist
ls -la src/assets/fonts/

# Verify CSS declarations
grep -n "font-face" src/index.css

# Clear browser cache and rebuild
rm -rf dist/
npm run build
```

#### Asset Import Errors
**Symptoms**: Cannot resolve asset files
```bash
# Check Vite configuration
cat vite.config.ts

# Verify asset paths
find src/assets -type f | head -10

# Solution: Use proper import syntax
import iconPath from '@/assets/icons/file.svg'
```

### Performance Issues

#### Slow Development Server
**Symptoms**: Slow hot reload, high CPU usage
```bash
# Check dependency optimization
rm -rf node_modules/.vite
npm run dev

# Exclude large directories from watching
# Edit vite.config.ts to ignore node_modules
```

#### Memory Usage High
**Symptoms**: IDE sluggish, system slow
```bash
# Check TypeScript memory usage
tsc --listFiles | wc -l

# Solutions:
# 1. Exclude test files from TypeScript project
# 2. Use TypeScript project references
# 3. Increase Node.js memory limit
export NODE_OPTIONS="--max-old-space-size=4096"
```

### Git and Version Control

#### Merge Conflicts in Lock Files
```bash
# Delete lock files and reinstall
rm package-lock.json
npm install

# For Cargo.lock
cd src-tauri && rm Cargo.lock && cargo build
```

#### Gitignore Issues
**Symptoms**: Unwanted files committed
```bash
# Update .gitignore and clean
echo "dist/" >> .gitignore
git rm -r --cached dist/
git commit -m "fix: update gitignore"
```

### IDE and Editor Issues

#### VSCode TypeScript Issues
```bash
# Reload TypeScript service
# Cmd+Shift+P → "TypeScript: Reload Project"

# Check TypeScript version
npm list typescript

# Workspace settings in .vscode/settings.json:
{
  "typescript.preferences.includePackageJsonAutoImports": "auto",
  "typescript.suggest.autoImports": true
}
```

#### ESLint Not Working
**Symptoms**: No linting errors shown, warnings missed
```bash
# Check ESLint configuration
npx eslint --print-config src/main.tsx

# Verify ESLint extension installed
# Check output panel for ESLint errors

# Reset ESLint cache
rm -rf .eslintcache
npm run lint
```

### Platform-Specific Issues

#### macOS Development
```bash
# Xcode Command Line Tools required
xcode-select --install

# If build fails with missing SDK
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

#### Windows Development  
```bash
# Visual Studio Build Tools required
# Install via Visual Studio Installer

# WebView2 runtime required for testing
# Download from Microsoft
```

#### Linux Development
```bash
# Install required system packages
sudo apt update
sudo apt install -y webkit2gtk-4.0-dev build-essential curl wget

# If using other distros, adjust package names
# Fedora: webkit2gtk4.0-devel
# Arch: webkit2gtk
```

## Diagnostic Commands

### System Information
```bash
# Check system requirements
npm run tauri info

# Node.js and npm versions  
node --version && npm --version

# Rust toolchain
rustc --version && cargo --version
```

### Project Health Check
```bash
# Run all quality checks
npm run lint
npm test  
cd src-tauri && cargo test && cargo clippy
npm run build

# Check bundle size
npm run build && du -sh dist/
```

### Environment Variables
```bash
# Useful debug variables
export RUST_LOG=debug          # Rust logging
export RUST_BACKTRACE=1        # Rust backtraces
export NODE_OPTIONS="--max-old-space-size=4096"  # Node memory
export TAURI_DEBUG=1           # Tauri debug mode
```

## Getting Help

### Before Asking for Help
1. Check this troubleshooting guide
2. Search existing GitHub issues  
3. Run diagnostic commands above
4. Try the "Common Fixes" section
5. Create minimal reproduction case

### Common Fixes to Try First
```bash
# The nuclear option - reset everything
git clean -fdx
npm install
cd src-tauri && cargo build
npm run tauri dev

# Less destructive - clear caches
rm -rf node_modules/.vite dist/ src-tauri/target/debug/
npm install
npm run tauri dev

# Quick fixes
pkill -f "vite|tauri"           # Kill processes
npm run format                  # Fix formatting
rm package-lock.json && npm i   # Reset dependencies
```

### Information to Include in Bug Reports
1. Operating system and version
2. Node.js and Rust versions (`npm run tauri info`)
3. Complete error message and stack trace
4. Steps to reproduce the issue
5. Expected vs actual behavior
6. Any recent changes made

### Performance Debugging
```bash
# Profile application startup
npm run tauri dev --verbose

# Check bundle analysis
npm run build -- --analyze

# Memory profiling
node --inspect-brk node_modules/.bin/vite dev
```