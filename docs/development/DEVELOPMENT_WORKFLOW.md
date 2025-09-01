# Development Workflow Guide

This guide provides comprehensive instructions for setting up, developing, and maintaining the Nimbus file manager project.

## Quick Start

### Prerequisites
- **Node.js** 18+ (LTS recommended)
- **Rust** 1.75+ with Cargo
- **Git** 2.25+

### Initial Setup
```bash
# Clone and setup
git clone <repository-url>
cd nimbus
npm install

# Verify Tauri setup
npm run tauri info

# Run development server
npm run tauri dev
```

## Project Structure

```
nimbus/
├── src/                     # React frontend (TypeScript)
│   ├── components/          # UI components
│   ├── services/           # Business logic & IPC
│   ├── store/              # Redux state management
│   ├── hooks/              # Custom React hooks
│   └── types/              # TypeScript definitions
├── src-tauri/              # Rust backend
│   ├── src/                # Main application logic
│   ├── crates/             # Local Rust crates
│   └── Cargo.toml          # Rust dependencies
├── docs/                   # Project documentation
└── tests/                  # Test files
```

## Development Commands

### Frontend Development
```bash
# Start React dev server only
npm run dev

# Run tests
npm test                    # Run all tests
npm test -- --watch        # Watch mode
npm test -- --coverage     # With coverage

# Code quality
npm run lint                # ESLint v9
npm run format              # Prettier
```

### Backend Development
```bash
# From src-tauri directory
cd src-tauri

# Build and test
cargo build                 # Debug build
cargo build --release       # Production build
cargo test                  # Run Rust tests
cargo fmt                   # Format code
cargo clippy                # Linting
```

### Full Stack Development
```bash
# Recommended: Single command for both
npm run tauri dev

# Alternative: Separate terminals
npm run dev                    # Terminal 1: Frontend
npm run tauri dev --no-watch   # Terminal 2: Backend only
```

## Development Workflow

### 1. Feature Development Process

#### Branch Strategy
```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes and commit regularly
git add -A
git commit -m "feat: implement file copy operation"

# Before PR - run quality checks
npm run lint
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

#### Code Quality Gates
Before committing, ensure all checks pass:
1. **Frontend Tests**: `npm test` (13 tests must pass)
2. **Backend Tests**: `cargo test` (14 tests must pass)
3. **Linting**: `npm run lint` (ESLint v9 clean)
4. **Type Safety**: `npm run build` (TypeScript compilation)

### 2. Command Architecture

Nimbus uses a modern command pattern for all operations:

```typescript
// UI Component delegates to Commands
const { executeCommand } = useCommands();
await executeCommand('create-file');

// Commands orchestrate business logic
export class CreateFileCommand extends Command {
  async execute(context: ExecutionContext): Promise<void> {
    await this.executor.createFile(context.panelId, filename);
  }
}
```

**Key Principles**:
- UI components are pure presentation
- All business logic in CommandExecutor
- Type-safe IPC communication
- Centralized error handling

### 3. Adding New Features

#### Frontend Components
```bash
# Create new component
mkdir -p src/components/feature
touch src/components/feature/FeatureComponent.tsx
touch src/components/feature/FeatureComponent.css
```

#### Backend Commands
```bash
# Add Rust command
# 1. Add to src-tauri/src/commands/files.rs
#[tauri::command]
pub async fn new_operation(args: Args) -> Result<Response, Error>

# 2. Register in src-tauri/src/main.rs
tauri::generate_handler![new_operation]

# 3. Add TypeScript types in src/services/commands/ipc/
```

#### Integration Pattern
1. **Backend**: Add Tauri command handler
2. **Service Layer**: Add TypeScript interface
3. **Business Logic**: Implement in CommandExecutor
4. **Commands**: Create Command class
5. **UI**: Use via useCommands hook
6. **Tests**: Add unit and integration tests

## Testing Strategy

### Frontend Testing (Vitest + React Testing Library)
```bash
# Test structure
src/
├── components/__tests__/
├── hooks/__tests__/
└── test/
    ├── setup.ts          # Global test setup
    ├── mocks/            # Mock implementations  
    └── utils/            # Test utilities
```

**Test Categories**:
- **Component Tests**: UI behavior and rendering
- **Hook Tests**: Custom React hooks
- **Integration Tests**: Command execution flows

### Backend Testing (Cargo Test)
```bash
# Test structure
src-tauri/
├── src/commands/tests.rs     # Command integration tests
└── crates/core-engine/src/tests.rs  # Unit tests
```

**Test Categories**:
- **Unit Tests**: Core business logic
- **Integration Tests**: Command handlers
- **Error Handling**: Edge cases and failures

## Code Style & Standards

### TypeScript/React
- **ESLint v9**: Flat config with TypeScript support
- **Strict TypeScript**: No any types in production code
- **React Hooks**: Modern functional components
- **CSS Modules**: Component-scoped styling

### Rust
- **Standard Formatting**: `cargo fmt`
- **Clippy Linting**: `cargo clippy`
- **Error Handling**: `Result<T, E>` pattern
- **Async/Await**: Tokio runtime

### Naming Conventions
```typescript
// TypeScript
interface FileInfo { }      // PascalCase for types
const fileName = "test";     // camelCase for variables
export class FileService    // PascalCase for classes

// Rust
struct FileInfo { }          // PascalCase for structs
fn list_files() { }         // snake_case for functions
const MAX_SIZE: usize       // SCREAMING_SNAKE_CASE for constants
```

## Performance Optimization

### Build Optimization
- **Font Loading**: Optimized from 40MB to 9.4MB (70% reduction)
- **Bundle Splitting**: Automatic code splitting via Vite
- **Tree Shaking**: Dead code elimination

### Runtime Performance
- **Virtual Scrolling**: Large file lists
- **Lazy Loading**: Components and routes
- **Memoization**: React.memo for expensive components

## Debugging

### Frontend Debugging
```bash
# Chrome DevTools integration
npm run tauri dev

# Console debugging
console.log() statements appear in dev tools
```

### Backend Debugging
```bash
# Rust debugging with prints
println!("Debug: {:?}", value);

# Advanced debugging
RUST_LOG=debug npm run tauri dev
```

### Common Issues

#### Build Errors
```bash
# TypeScript errors
npm run build              # Check for type errors

# Rust compilation errors  
cargo check --manifest-path src-tauri/Cargo.toml
```

#### Test Failures
```bash
# Frontend test failures
npm test -- --reporter=verbose

# Backend test failures
cargo test -- --nocapture
```

## Git Workflow

### Commit Message Format
```bash
feat: add file copy functionality
fix: resolve memory leak in file watcher  
docs: update API documentation
test: add integration tests for commands
chore: update dependencies
```

### Pre-commit Checklist
- [ ] Tests pass (`npm test && cargo test`)
- [ ] Linting clean (`npm run lint`)
- [ ] Code formatted (`npm run format && cargo fmt`)
- [ ] No TypeScript errors (`npm run build`)
- [ ] Documentation updated if needed

## Deployment

### Development Build
```bash
npm run tauri build --debug
```

### Production Build
```bash
npm run tauri build
```

### Platform-specific Builds
```bash
# Windows
npm run tauri build --target x86_64-pc-windows-msvc

# macOS  
npm run tauri build --target x86_64-apple-darwin

# Linux
npm run tauri build --target x86_64-unknown-linux-gnu
```

## Troubleshooting

### Common Development Issues

#### Port Conflicts
```bash
# Kill existing processes
pkill -f "vite"
pkill -f "tauri"

# Use different port
npm run dev -- --port 3001
```

#### Dependency Issues
```bash
# Clear caches
rm -rf node_modules package-lock.json
npm install

# Rust dependencies
cargo clean
cargo build
```

#### Font Loading Issues
After font optimization, if fonts don't load:
1. Check `src/index.css` font-face declarations
2. Verify font files exist in `src/assets/fonts/`
3. Clear browser cache and rebuild

### Getting Help

1. **Check Documentation**: `docs/` directory
2. **Search Issues**: Existing GitHub issues  
3. **Run Diagnostics**: `npm run tauri info`
4. **Debug Mode**: `RUST_LOG=debug npm run tauri dev`

## Contributing

### Code Review Process
1. Create feature branch
2. Implement changes with tests
3. Pass all quality gates
4. Create pull request
5. Address review feedback
6. Merge after approval

### Code Standards
- Write self-documenting code
- Add comments for complex business logic
- Include tests for new functionality
- Update documentation as needed

---

For more specific information, see:
- [API Documentation](../api/tauri-commands.md)
- [Architecture Overview](../architecture.md)
- [Multi-panel System](../multi-panel.md)