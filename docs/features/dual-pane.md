# Dual-Pane Interface

## Overview

The dual-pane interface is the cornerstone of Nimbus, providing side-by-side file management that enables efficient operations between different locations. This design, inspired by orthodox file managers like Total Commander, maximizes productivity through reduced navigation overhead and streamlined file operations.

## Interface Layout

### Pane Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    Toolbar & Menu                           │
├─────────────────────┬─┬─────────────────────────────────────┤
│  Left Pane          │ │  Right Pane                         │
│  ┌─ Tabs ─────────┐ │S│  ┌─ Tabs ─────────┐                │
│  │ Tab1 │Tab2│Tab3│ │P│  │ Tab1 │Tab2│Tab3│                │
│  └─────────────────┘ │L│  └─────────────────┘                │
│  ┌─ Path ──────────┐ │I│  ┌─ Path ──────────┐               │
│  │ /home/user/docs │ │T│  │ /home/user/pics │               │
│  └─────────────────┘ │T│  └─────────────────┘               │
│  ┌─ File List ─────┐ │E│  ┌─ File List ─────┐               │
│  │ 📁 folder1      │ │R│  │ 🖼️ image1.jpg   │               │
│  │ 📄 document.txt │ │ │  │ 🖼️ image2.png   │               │
│  │ 📄 readme.md    │ │ │  │ 🖼️ photo.jpeg   │               │
│  │ 📦 archive.zip  │ │ │  │ 📁 thumbnails   │               │
│  └─────────────────┘ │ │  └─────────────────┘               │
│  ┌─ Status ────────┐ │ │  ┌─ Status ────────┐               │
│  │ 142 items, 2.3GB│ │ │  │ 89 items, 45MB  │               │
│  └─────────────────┘ │ │  └─────────────────┘               │
├─────────────────────┴─┴─────────────────────────────────────┤
│                 Global Status Bar                           │
└─────────────────────────────────────────────────────────────┘
```

### Pane Components

Each pane consists of several key components working together:

#### Tab Bar
- **Multiple Tabs**: Each pane supports unlimited tabs (memory permitting)
- **Tab Management**: Close, reorder, and duplicate tabs
- **Tab Indicators**: Visual cues for loading, errors, or special locations
- **Tab Context Menu**: Right-click for tab-specific operations

#### Path Navigation
- **Breadcrumb Path**: Clickable path segments for quick navigation
- **Address Bar Mode**: Switch to editable text input for direct path entry
- **Path History**: Dropdown showing recent locations in this pane
- **Quick Bookmarks**: One-click access to frequently used locations

#### File List Area
- **Multiple View Modes**: List, details, grid, and tree views
- **Sortable Columns**: Click column headers to sort, multi-level sorting
- **Selection Management**: Visual feedback for selected items
- **Context Sensitivity**: Right-click menus adapt to selection and location

#### Status Information
- **Item Counts**: Number of files, folders, and selected items
- **Size Information**: Total size, selected size, available space
- **Filter Status**: Active filters and search terms
- **Operation Progress**: Current file operation status

## Pane Management

### Active Pane System

Only one pane is "active" at any time, determining where keyboard input and commands apply.

```typescript
interface PaneState {
    id: 'left' | 'right';
    isActive: boolean;
    tabs: TabState[];
    activeTabIndex: number;
    selection: Set<string>;
    history: NavigationHistory;
}

interface AppState {
    leftPane: PaneState;
    rightPane: PaneState;
    activePaneId: 'left' | 'right';
}
```

#### Visual Indicators
- **Border Highlighting**: Active pane has distinct border color/style
- **Title Bar**: Active pane title bar uses accent color
- **Tab Styling**: Active pane tabs have different appearance
- **Cursor Behavior**: File selection cursor only visible in active pane

#### Switching Panes
- **Tab Key**: Primary method for switching between panes
- **Mouse Click**: Click in pane area to make it active
- **Keyboard Navigation**: Arrow keys move within active pane only
- **Command Target**: File operations apply to active pane by default

### Splitter Control

The adjustable splitter between panes provides flexible layout control:

#### Splitter Features
- **Drag Resize**: Mouse drag to adjust pane widths
- **Double-Click Reset**: Double-click splitter to return to 50/50 split
- **Keyboard Resize**: Keyboard shortcuts for common split ratios
- **Memory**: Remember last splitter position between sessions

#### Split Ratios
```typescript
type SplitRatio = 
    | { type: 'equal' }           // 50/50
    | { type: 'golden' }          // ~62/38 golden ratio
    | { type: 'custom', ratio: number } // User-defined percentage
    | { type: 'single', activePane: 'left' | 'right' }; // Single pane mode
```

#### Single Pane Mode
- **Toggle Mode**: Switch between dual and single pane layouts
- **Keyboard Shortcut**: F12 to toggle single/dual pane mode
- **Responsive Design**: Automatically switch to single pane on narrow screens
- **Pane Selection**: Choose which pane to display in single mode

## Tab Management

### Tab Lifecycle

Each tab represents an independent navigation session within a pane:

```typescript
interface TabState {
    id: string;
    title: string;
    path: string;
    files: FileInfo[];
    loading: boolean;
    error?: string;
    selection: Set<string>;
    sortBy: SortField;
    sortOrder: 'asc' | 'desc';
    viewMode: ViewMode;
    filter?: string;
    history: string[];
    historyIndex: number;
    scrollPosition: number;
    lastAccessed: Date;
}
```

#### Tab Creation
- **Ctrl+T**: Create new tab in active pane at current location
- **Middle-Click**: Create new tab from folder (middle-click folder in file list)
- **Drag & Drop**: Drop folder onto tab bar to create new tab
- **Bookmark**: Create tab from bookmark (inherits bookmark's settings)

#### Tab Navigation
- **Ctrl+Tab**: Cycle through tabs in active pane (most recently used order)
- **Ctrl+Shift+Tab**: Cycle through tabs in reverse order
- **Ctrl+1-9**: Jump to specific tab number
- **Ctrl+0**: Jump to last tab

#### Tab Closing
- **Ctrl+W**: Close active tab
- **Middle-Click**: Close tab (middle-click on tab)
- **Close Button**: X button on tab (visible on hover)
- **Close Others**: Right-click menu option to close all other tabs

### Advanced Tab Features

#### Tab Synchronization
```typescript
interface TabSyncOptions {
    syncNavigation: boolean;    // Navigate both panes together
    syncSelection: boolean;     // Mirror selection between panes
    syncViewMode: boolean;      // Match view modes
    syncSort: boolean;          // Match sorting preferences
}
```

- **Sync Navigation**: Option to navigate both panes to same location
- **Independent History**: Each tab maintains its own navigation history
- **Tab Templates**: Save tab configurations as templates for reuse

#### Tab Context Menu
Right-click on tab provides context-specific options:

- **Duplicate Tab**: Create copy of current tab in new tab
- **Move to Other Pane**: Move tab from left pane to right pane (or vice versa)
- **Close Others**: Close all other tabs in this pane
- **Close to Right**: Close all tabs to the right of current tab
- **Reopen**: Reopen recently closed tabs
- **Pin Tab**: Pin important tabs to prevent accidental closing
- **Rename Tab**: Custom tab titles for better organization

## File Operations Between Panes

### Operation Flow

The dual-pane design enables intuitive file operations by treating the inactive pane as the default destination:

```rust
// Example: Copy operation between panes
#[tauri::command]
pub async fn copy_to_other_pane(
    files: Vec<PathBuf>,
    source_pane: PaneId,
    options: CopyOptions
) -> Result<OperationId, FileError> {
    let destination = get_other_pane_current_path(source_pane)?;
    copy_files(files, destination, options).await
}
```

#### Keyboard Operations
- **F5 (Copy)**: Copy selected files from active pane to inactive pane
- **F6 (Move)**: Move selected files from active pane to inactive pane
- **Shift+F5**: Copy with options dialog (rename, overwrite policy, etc.)
- **Shift+F6**: Move with options dialog

#### Drag & Drop Operations
- **Simple Drag**: Drag files between panes for default operation (copy or move)
- **Ctrl+Drag**: Force copy operation
- **Shift+Drag**: Force move operation
- **Alt+Drag**: Create shortcuts/symlinks
- **Right-Drag**: Context menu with operation choices

### Operation Feedback

#### Progress Indication
```typescript
interface FileOperation {
    id: string;
    type: 'copy' | 'move' | 'delete' | 'compress';
    source: string[];
    destination?: string;
    progress: {
        current: number;
        total: number;
        currentFile: string;
        bytesProcessed: number;
        bytesTotal: number;
        speed: number; // bytes/second
        eta: number;   // seconds remaining
    };
    status: 'running' | 'paused' | 'completed' | 'error' | 'cancelled';
}
```

#### Visual Feedback
- **Progress Bar**: Global progress bar in status area
- **File Highlighting**: Highlight files being processed
- **Pane Indicators**: Show which pane is source/destination
- **Operation Queue**: List of pending operations

## Navigation Synchronization

### Synchronized Browsing

Optional feature to keep both panes synchronized for comparison workflows:

#### Sync Modes
```typescript
type SyncMode = 
    | 'none'        // Independent navigation
    | 'path'        // Navigate to same path in both panes
    | 'relative'    // Maintain relative path relationship
    | 'structure';  // Navigate similar folder structures
```

#### Use Cases
- **File Comparison**: Compare contents of similar folder structures
- **Backup Verification**: Compare source and backup locations
- **Development Workflow**: Navigate source and build directories together
- **Archive Comparison**: Compare archive contents with extracted files

### Comparison Features

#### Visual Comparison
- **Diff Highlighting**: Highlight files that differ between panes
- **Size Comparison**: Color-code files by size differences
- **Date Comparison**: Highlight newer/older files
- **Missing Files**: Show files present in one pane but not the other

#### Comparison Tools
- **Compare Directories**: Built-in directory comparison tool
- **Sync Wizard**: Guide user through folder synchronization
- **Merge Preview**: Preview changes before applying sync operations
- **Conflict Resolution**: Handle conflicting files during sync

## Layout Customization

### Responsive Design

The dual-pane interface adapts to different screen sizes and orientations:

#### Breakpoints
- **Wide Screens** (>1600px): Full dual-pane with wide splitter area
- **Standard Screens** (1024-1600px): Standard dual-pane layout
- **Narrow Screens** (768-1024px): Compact dual-pane with minimal margins
- **Mobile/Tablet** (<768px): Single pane mode with pane switching

#### Layout Options
```typescript
interface LayoutSettings {
    orientation: 'horizontal' | 'vertical';  // Side-by-side or top/bottom
    splitterSize: number;                    // Splitter width in pixels
    tabBarPosition: 'top' | 'bottom';        // Tab bar placement
    showStatusBars: boolean;                 // Individual pane status bars
    compactMode: boolean;                    // Reduced spacing/padding
}
```

### Accessibility Features

#### Screen Reader Support
- **ARIA Regions**: Each pane marked as distinct region
- **Focus Management**: Clear focus indication and navigation
- **Announcements**: Screen reader announcements for pane switches
- **Keyboard Navigation**: Full keyboard accessibility

#### Visual Accessibility
- **High Contrast**: Support for high contrast themes
- **Large Text**: Scalable interface elements
- **Color Independence**: Use patterns/shapes in addition to colors
- **Focus Indicators**: Clear visual focus indicators

## Performance Optimization

### Memory Management
- **Tab Virtualization**: Unload inactive tab contents to save memory
- **Lazy Loading**: Load file lists only when tabs become active
- **Cache Management**: Smart caching of recently accessed directories
- **Resource Cleanup**: Cleanup resources when tabs are closed

### Rendering Optimization
- **Virtual Scrolling**: Handle large file lists efficiently
- **Incremental Updates**: Update only changed portions of file lists  
- **Background Loading**: Load file metadata in background
- **Debounced Operations**: Batch rapid navigation changes

---

The dual-pane interface provides the foundation for efficient file management, enabling users to work with multiple locations simultaneously while maintaining clear visual organization and intuitive operation flows.