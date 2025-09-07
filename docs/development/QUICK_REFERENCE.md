# Developer Quick Reference

**🟢 Current Status**: Core file operations are fully integrated and production-ready.

## Essential Commands

### Setup & Development
```bash
npm install                     # Install dependencies
npm run tauri dev              # Start development server
npm run dev                    # Frontend only
npm test                       # Run frontend tests
cargo test                     # Run backend tests (from src-tauri/)
```

### Code Quality
```bash
npm run lint                   # ESLint v9 check
npm run format                 # Prettier formatting
cargo fmt                      # Rust formatting
cargo clippy                   # Rust linting
```

### Building
```bash
npm run build                  # TypeScript compilation
npm run tauri build            # Production build
npm run tauri build --debug    # Debug build
```

## Project Architecture

### Frontend (React + TypeScript)
```
src/
├── components/         # Pure UI components
├── services/          # Business logic (CommandExecutor)
├── hooks/            # Custom React hooks (useCommands)
├── store/            # Redux Toolkit state
└── types/            # TypeScript definitions
```

### Backend (Rust + Tauri)
```
src-tauri/
├── src/commands/     # IPC command handlers  
├── crates/          # Local Rust libraries
└── Cargo.toml       # Rust dependencies
```

## Command Pattern Flow

```typescript
// UI → Commands → Services → Backend
UI Component → useCommands Hook → Command Class → CommandExecutor → FileService → Tauri Backend
```

## Current Implementation Status

### ✅ FULLY INTEGRATED - Ready for Use
- **File Operations**: Copy, move, delete, rename, create files/folders
- **Command System**: Modern command pattern with dependency injection  
- **IPC Layer**: Type-safe communication between frontend/backend
- **State Management**: Redux with real-time UI updates
- **Plugin System**: Complete foundation with example plugins
- **Search System**: Advanced search with fuzzy matching

### 🟡 IMPLEMENTED - Needs Integration
- **Archive Support**: Backend ready, frontend integration pending
- **File Viewers**: Command structure exists, implementation needed
- **Remote File Systems**: Plugin foundation complete

### 📋 PLANNED
- **Progress Events**: Real-time operation progress
- **Conflict Resolution**: Interactive file conflict dialogs
- **Performance Optimization**: Caching and parallel operations

## Common Development Tasks

### Adding New Command
1. **Backend**: Add Tauri command in `src-tauri/src/commands/`
2. **Service**: Add TypeScript interface in `src/services/`
3. **Command**: Create command class implementing `Command`
4. **Register**: Add to command registry
5. **Use**: Call via `useCommands` hook in UI

### Adding New Component
1. Create in `src/components/[category]/`
2. Add CSS file for styling
3. Write component tests
4. Export from appropriate index file
5. Use in parent components

## Test Structure

### Frontend Tests (13 tests)
- Component rendering and behavior
- Hook functionality  
- Command execution flows
- User interactions

### Backend Tests (14 tests)
- File operations (CRUD)
- System commands
- Error handling
- Path resolution

## Quality Gates

Before committing, ensure:
- [ ] `npm test` passes (13 tests)
- [ ] `cargo test` passes (14 tests)  
- [ ] `npm run lint` clean
- [ ] `npm run build` succeeds
- [ ] Code formatted (`npm run format`)

## Font Optimization (Completed)

- **Before**: 38.4MB (17 font files)
- **After**: 9.4MB (4 font files)
- **Savings**: 70% reduction
- **Files**: Regular, Italic, Medium, Bold only

## Key Technologies

- **Frontend**: React 19, TypeScript 5.9, Redux Toolkit, Vite
- **Backend**: Rust 1.75+, Tauri 2.8, Tokio async runtime
- **Testing**: Vitest, React Testing Library, Cargo test
- **Quality**: ESLint v9, Prettier, Clippy

## Performance Targets

### Memory Usage
- Idle: <50MB RAM
- Large directories: <200MB  
- Active operations: <150MB

### Response Times
- Directory listing: <100ms for 10K files
- File operations: Competitive with native tools
- Search: <2s for 1M+ files

## Debugging Tips

### Frontend Issues
- Use Chrome DevTools with `npm run tauri dev`
- Check console for error messages
- Use React Developer Tools

### Backend Issues  
- Use `RUST_LOG=debug npm run tauri dev`
- Add `println!` debugging statements
- Check Tauri logs for IPC errors

### Common Fixes
```bash
# Clear caches
rm -rf node_modules dist src-tauri/target
npm install && cargo build

# Kill processes
pkill -f "vite|tauri"

# Reset development
git clean -fd && npm install
```

## File Paths

- **Config**: `package.json`, `tsconfig.json`, `eslint.config.js`
- **Main files**: `src/main.tsx`, `src-tauri/src/main.rs`
- **Documentation**: `docs/`, `CLAUDE.md`
- **Tests**: `src/**/__tests__/`, `src-tauri/src/commands/tests.rs`