# Nimbus Architecture Documentation

## Overview

Nimbus follows a clean architecture pattern with clear separation of concerns between UI, business logic, and infrastructure layers.

## Current Architecture

### High-Level Flow
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   UI Components │───▶│ CommandExecutor │───▶│   FileService   │───▶│  Tauri Backend  │
│  (Pure UI Only) │    │ (Business Logic)│    │ (Infrastructure)│    │  (File System)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   Redux Store   │
                       │ (State Management)│
                       └─────────────────┘
```

### Layer Responsibilities

#### 1. UI Components (Presentation Layer)
- **Location**: `src/components/`
- **Responsibility**: Pure presentation logic only
- **Examples**: FilePanel, MultiPanelLayout, CommandPalette
- **Rules**:
  - No direct Redux dispatch calls
  - No business logic
  - Delegate all actions to CommandExecutor
  - Use Redux hooks only for reading state

#### 2. CommandExecutor (Business Logic Layer)
- **Location**: `src/services/commandExecutor.ts`
- **Responsibility**: ALL business logic operations
- **Features**:
  - Centralized command execution
  - Unified error handling
  - Loading state management
  - Progress tracking
  - Notification management
  - Context-aware operations

#### 3. FileService (Infrastructure Layer)
- **Location**: `src/services/fileService.ts`
- **Responsibility**: IPC communication with Tauri backend
- **Rules**:
  - Pure infrastructure - no business logic
  - Direct mapping to Tauri commands
  - Path normalization for cross-platform compatibility

#### 4. Redux Store (State Management)
- **Location**: `src/store/`
- **Responsibility**: Application state management
- **Features**:
  - Panel configurations
  - Clipboard state
  - Drag and drop state
  - Notifications
  - Progress indicators

## Component Architecture

### FilePanel Component
```typescript
// BEFORE: Mixed UI and business logic
const FilePanel = () => {
  const handleDeleteFiles = async (files: FileInfo[]) => {
    // Business logic mixed with UI
    dispatch(setLoading({ panelId, isLoading: true }));
    await FileService.deleteItem(file.path);
    // ... more business logic
  };
};

// AFTER: Pure UI component
const FilePanel = () => {
  const handleDeleteFiles = (files: FileInfo[]) => {
    // Pure UI - delegates to CommandExecutor
    CommandExecutor.deleteFiles(panelId, files);
  };
};
```

### CommandExecutor Architecture
```typescript
export class CommandExecutor {
  private static context: CommandContext;
  
  // Initialize with application context
  static initialize(context: CommandContext) { /* ... */ }
  
  // File Operations
  static async createFile(panelId: string, input: string) { /* ... */ }
  static async deleteFiles(panelId: string, files: FileInfo[]) { /* ... */ }
  static async copyFiles(panelId: string, files: FileInfo[]) { /* ... */ }
  
  // Navigation Operations
  static async goToHome(panelId: string) { /* ... */ }
  static async navigateToPath(panelId: string, path: string) { /* ... */ }
  
  // Drag and Drop Operations
  static startDrag(panelId: string, file: FileInfo, isCtrlKey: boolean) { /* ... */ }
  static async handleDrop(targetPanelId: string, dragState: any) { /* ... */ }
  
  // Private utilities
  private static setLoadingState(panelId: string, isLoading: boolean) { /* ... */ }
  private static showNotification(message: string, type: string) { /* ... */ }
}
```

## Command System Architecture

### Command Registration Flow
```typescript
// 1. Command Registry registers commands
CommandRegistry.register({
  id: 'file.create',
  label: 'Create File',
  action: async (context) => {
    await CommandExecutor.createFile(context.activePanelId, fileName);
  }
});

// 2. Command Palette executes commands
const executeCommand = (command: Command) => {
  command.action(executionContext);
};

// 3. CommandExecutor handles business logic
static async createFile(panelId: string, input: string) {
  try {
    this.setLoadingState(panelId, true);
    const { targetDir, fileName } = this.parseFileInput(input, currentPath);
    await FileService.createFile(targetDir, fileName);
    this.showNotification('File created successfully', 'success');
  } catch (error) {
    this.showNotification(`Failed to create file: ${error.message}`, 'error');
  } finally {
    this.setLoadingState(panelId, false);
  }
}
```

## State Management Architecture

### Redux Store Structure
```typescript
interface RootState {
  panels: {
    panels: Record<string, Panel>;           // Panel configurations
    activePanelId: string | null;            // Currently active panel
    gridLayout: { rows: number; cols: number }; // Grid arrangement
    clipboardState: {                       // Copy/cut operations
      hasFiles: boolean;
      files: FileInfo[];
      operation: 'copy' | 'cut';
      sourcePanelId: string | null;
    };
    dragState: {                            // Drag and drop state
      isDragging: boolean;
      draggedFiles: string[];
      sourcePanelId: string | null;
      dragOperation: 'copy' | 'move';
    };
    notifications: Notification[];           // User notifications
    progressIndicators: ProgressIndicator[]; // File operation progress
  }
}
```

### State Update Flow
```
User Action → CommandExecutor → FileService → Backend
     ↓             ↓               ↓
UI Component → Business Logic → Infrastructure
     ↓
Redux Dispatch (via CommandExecutor)
     ↓
State Update
     ↓
UI Re-render
```

## Error Handling Strategy

### Centralized Error Handling
```typescript
export class CommandExecutor {
  static async performOperation(panelId: string) {
    try {
      this.setLoadingState(panelId, true);
      // Business logic here
      this.showNotification('Operation completed', 'success');
    } catch (error) {
      console.error('Operation failed:', error);
      this.showNotification(
        `Operation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'error'
      );
    } finally {
      this.setLoadingState(panelId, false);
    }
  }
}
```

## Progress Tracking Architecture

### Progress Indicator System
```typescript
// CommandExecutor manages progress for long operations
static async copyLargeFiles(panelId: string, files: FileInfo[]) {
  const progressId = `copy-${Date.now()}`;
  
  // Create progress indicator
  this.dispatch(addProgressIndicator({
    id: progressId,
    operation: 'copy',
    fileName: files.length > 1 ? `${files.length} items` : files[0].name,
    progress: 0,
    totalFiles: files.length,
    currentFile: 0,
    isComplete: false
  }));
  
  for (let i = 0; i < files.length; i++) {
    // Update progress
    this.dispatch(updateProgressIndicator({
      id: progressId,
      updates: {
        currentFile: i + 1,
        progress: ((i + 1) / files.length) * 100
      }
    }));
    
    await FileService.copyItem(files[i].path, destinationPath);
  }
  
  // Complete progress
  this.dispatch(updateProgressIndicator({
    id: progressId,
    updates: { isComplete: true, progress: 100 }
  }));
}
```

## Benefits of Current Architecture

### 1. Separation of Concerns
- UI components focus solely on presentation
- Business logic is centralized and testable
- Infrastructure layer is pure and reusable

### 2. Maintainability
- Single location for all business logic changes
- Consistent error handling and user feedback
- Easy to add new commands or modify existing ones

### 3. Testability
- CommandExecutor can be unit tested independently
- UI components can be tested without business logic
- Clear interfaces between layers

### 4. Type Safety
- Strong TypeScript interfaces across all layers
- Compile-time error detection
- IntelliSense support for better developer experience

### 5. Scalability
- Easy to add new commands through CommandExecutor
- Consistent patterns for new developers
- Clear extension points for future features

## Development Patterns

### Adding New Features
1. **Add method to CommandExecutor** with business logic
2. **Register in CommandRegistry** for command palette
3. **Update UI components** to call CommandExecutor method
4. **Add types** in `src/types/` if needed

### Testing Strategy
1. **Unit test CommandExecutor methods** with mocked dependencies
2. **Integration test** command flows from UI to backend
3. **E2E test** complete user workflows

This architecture ensures clean separation of concerns, maintainable code, and consistent user experience across the entire application.