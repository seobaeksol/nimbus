# Multi-Panel Interface

## Overview

The multi-panel interface is the cornerstone of Nimbus, providing flexible side-by-side file management that enables efficient operations between different locations. This design, inspired by orthodox file managers like Total Commander, maximizes productivity through reduced navigation overhead and streamlined file operations. Unlike traditional dual-panel designs, Nimbus supports multiple panels that can be arranged and configured according to user needs.

## Interface Layout

### Panel Structure

#### Linear Layout (Horizontal)
```
┌─────────────────────────────────────────────────────────────┐
│                    Toolbar & Menu                           │
├─────────────────┬─┬─────────────────┬─┬───────────────────────┤
│  Panel 1        │ │  Panel 2        │ │  Panel 3              │
│  ┌─ Tabs ─────┐ │S│  ┌─ Tabs ─────┐ │S│  ┌─ Tabs ─────┐       │
│  │Tab1│Tab2│+│ │P│  │Tab1│Tab2│+│ │P│  │Tab1│Tab2│+│       │
│  └───────────┘ │L│  └───────────┘ │L│  └───────────┘       │
│  ┌─ Path ────┐ │I│  ┌─ Path ────┐ │I│  ┌─ Path ────┐       │
│  │/home/user │ │T│  │/var/log   │ │T│  │/tmp       │       │
│  └───────────┘ │T│  └───────────┘ │T│  └───────────┘       │
│  ┌─File List─┐ │E│  ┌─File List─┐ │E│  ┌─File List─┐       │
│  │📁 docs    │ │R│  │📄 sys.log │ │R│  │📄 temp.txt│       │
│  │📄 file.tx │ │ │  │📄 app.log │ │ │  │📁 cache   │       │
│  │📦 arch.zip│ │ │  │📄 err.log │ │ │  │📄 data.tmp│       │
│  └───────────┘ │ │  └───────────┘ │ │  └───────────┘       │
│  ┌─Status───┐  │ │  ┌─Status───┐  │ │  ┌─Status───┐        │
│  │142 items │  │ │  │89 items  │  │ │  │15 items  │        │
│  └─────────┘   │ │  └─────────┘   │ │  └─────────┘         │
├─────────────────┴─┴─────────────────┴─┴───────────────────────┤
│                 Global Status Bar                           │
└─────────────────────────────────────────────────────────────┘
```

#### Grid Layout (2x2)
```
┌─────────────────────────────────────────────────────────────┐
│                    Toolbar & Menu                           │
├─────────────────────────┬─┬─────────────────────────────────┤
│  Panel 1 (Top-Left)     │ │  Panel 2 (Top-Right)           │
│  ┌─ Tabs ─────────────┐ │V│  ┌─ Tabs ─────────────┐         │
│  │Tab1│Tab2│Tab3│+   │ │E│  │Tab1│Tab2│+         │         │
│  └───────────────────┘ │R│  └───────────────────┘         │
│  ┌─ Path ────────────┐ │T│  ┌─ Path ────────────┐         │
│  │/home/user/src     │ │I│  │/home/user/build   │         │
│  └───────────────────┘ │C│  └───────────────────┘         │
│  ┌─ File List ───────┐ │A│  ┌─ File List ───────┐         │
│  │📁 components      │ │L│  │📄 app.exe         │         │
│  │📄 main.ts         │ │ │  │📄 styles.css      │         │
│  │📄 utils.ts        │ │S│  │📁 assets          │         │
│  │📦 lib.zip         │ │P│  │📄 manifest.json   │         │
│  └───────────────────┘ │L│  └───────────────────┘         │
│  ┌─ Status ─────────┐  │I│  ┌─ Status ─────────┐          │
│  │89 items, 45MB    │  │T│  │12 items, 8.2MB   │          │
│  └─────────────────┘   │T│  └─────────────────┘           │
├─────────────────────────┼─┼─────────────────────────────────┤
│  Panel 3 (Bottom-Left) │E│  Panel 4 (Bottom-Right)        │
│  ┌─ Tabs ─────────────┐ │R│  ┌─ Tabs ─────────────┐        │
│  │Tab1│+             │ │ │  │Tab1│Tab2│+         │        │
│  └───────────────────┘ │ │  └───────────────────┘        │
│  ┌─ Path ────────────┐ │ │  ┌─ Path ────────────┐        │
│  │/home/user/tests   │ │ │  │/home/user/docs    │        │
│  └───────────────────┘ │ │  └───────────────────┘        │
│  ┌─ File List ───────┐ │ │  ┌─ File List ───────┐        │
│  │📄 unit.test.ts    │ │ │  │📄 README.md       │        │
│  │📄 e2e.test.ts     │ │ │  │📄 CHANGELOG.md    │        │
│  │📁 fixtures        │ │ │  │📄 LICENSE         │        │
│  │📄 jest.config.js  │ │ │  │📁 images          │        │
│  └───────────────────┘ │ │  └───────────────────┘        │
│  ┌─ Status ─────────┐  │ │  ┌─ Status ─────────┐         │
│  │34 items, 2.1MB   │  │ │  │8 items, 1.3MB    │         │
│  └─────────────────┘   │ │  └─────────────────┘          │
├─────────────────────────┴─┴─────────────────────────────────┤
│                 Global Status Bar                           │
└─────────────────────────────────────────────────────────────┘
```

#### Grid Layout (2x3)
```
┌───────────────────────────────────────────────────────────────────────┐
│                         Toolbar & Menu                               │
├─────────────────┬─┬─────────────────┬─┬─────────────────────────────┤
│  Panel 1        │ │  Panel 2        │ │  Panel 3                    │
│  /src           │ │  /build         │ │  /dist                      │
│  [File List]    │ │  [File List]    │ │  [File List]                │
├─────────────────┼─┼─────────────────┼─┼─────────────────────────────┤
│  Panel 4        │ │  Panel 5        │ │  Panel 6                    │
│  /tests         │ │  /docs          │ │  /backup                    │
│  [File List]    │ │  [File List]    │ │  [File List]                │
├─────────────────┴─┴─────────────────┴─┴─────────────────────────────┤
│                      Global Status Bar                              │
└───────────────────────────────────────────────────────────────────────┘
```

### Panel Components

Each panel consists of several key components working together:

#### Tab Bar
- **Multiple Tabs**: Each panel supports unlimited tabs (memory permitting)
- **Tab Management**: Close, reorder, and duplicate tabs
- **Tab Indicators**: Visual cues for loading, errors, or special locations
- **Tab Context Menu**: Right-click for tab-specific operations
- **Add Tab Button**: Quick access to create new tabs

#### Path Navigation
- **Breadcrumb Path**: Clickable path segments for quick navigation
- **Address Bar Mode**: Switch to editable text input for direct path entry
- **Path History**: Dropdown showing recent locations in this panel
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

## Panel Management

### Active Panel System

Only one panel is "active" at any time, determining where keyboard input and commands apply.

```typescript
interface PanelState {
    id: string;
    isActive: boolean;
    tabs: TabState[];
    activeTabIndex: number;
    selection: Set<string>;
    history: NavigationHistory;
    position: PanelPosition;
    width: number;
}

interface AppState {
    panels: PanelState[];
    activePanelId: string;
    layout: PanelLayout;
}

interface PanelLayout {
    type: 'horizontal' | 'vertical' | 'grid';
    splitterPositions: number[];
    panelCount: number;
    minPanelWidth: number;
}
```

#### Visual Indicators
- **Border Highlighting**: Active panel has distinct border color/style
- **Title Bar**: Active panel title bar uses accent color
- **Tab Styling**: Active panel tabs have different appearance
- **Cursor Behavior**: File selection cursor only visible in active panel

#### Switching Panels
- **Tab Key**: Primary method for cycling through panels
- **Ctrl+1-9**: Jump directly to specific panel number
- **Mouse Click**: Click in panel area to make it active
- **Keyboard Navigation**: Arrow keys move within active panel only
- **Command Target**: File operations apply to active panel by default

## Grid Layout System

### Grid Layout Overview

Nimbus supports advanced grid layouts that arrange panels in rows and columns, providing a powerful workspace for complex file management tasks. Grid layouts are particularly useful for developers, system administrators, and power users who need to work with multiple directory structures simultaneously.

### Grid Layout Types

#### 2x2 Grid Layout
Perfect for comparing four related directories or workflows:
```typescript
interface Grid2x2Layout {
    type: 'grid2x2';
    panels: {
        topLeft: PanelState;
        topRight: PanelState;
        bottomLeft: PanelState;
        bottomRight: PanelState;
    };
    horizontalSplit: number;    // 0.0-1.0 position of horizontal divider
    verticalSplit: number;      // 0.0-1.0 position of vertical divider
}
```

**Common Use Cases:**
- **Development Workflow**: Source (`/src`) → Build (`/build`) → Test (`/test`) → Deploy (`/dist`)
- **Content Management**: Draft → Review → Published → Archive
- **System Administration**: Live → Staging → Backup → Temp
- **Media Production**: Raw → Processed → Finals → Archive

#### 2x3 Grid Layout (6 Panels)
For complex multi-location workflows:
```typescript
interface Grid2x3Layout {
    type: 'grid2x3';
    panels: {
        topRow: [PanelState, PanelState, PanelState];
        bottomRow: [PanelState, PanelState, PanelState];
    };
    horizontalSplit: number;
    verticalSplits: [number, number];  // Two vertical divider positions
}
```

**Common Use Cases:**
- **Multi-Site Management**: Site1 → Site2 → Site3 → Backup1 → Backup2 → Backup3  
- **Version Control**: Main → Dev → Feature → Staging → Test → Release
- **Content Localization**: EN → ES → FR → DE → JA → ZH

#### 3x2 Grid Layout (6 Panels)
Alternative arrangement for different workflow patterns:
```typescript
interface Grid3x2Layout {
    type: 'grid3x2';
    panels: {
        leftCol: [PanelState, PanelState];
        centerCol: [PanelState, PanelState];
        rightCol: [PanelState, PanelState];
    };
    verticalSplits: [number, number];
    horizontalSplit: number;
}
```

### Grid Layout Controls

#### Keyboard Shortcuts
- **Ctrl+G, 2**: Switch to 2x2 grid layout
- **Ctrl+G, 3**: Switch to 2x3 grid layout (6 panels)
- **Ctrl+G, H**: Switch to horizontal 3-panel layout
- **Ctrl+G, V**: Switch to vertical 3-panel layout
- **Ctrl+G, 1**: Return to single panel
- **Ctrl+G, D**: Return to classic dual panel

#### Grid Navigation
- **Ctrl+Arrow Keys**: Move between panels in grid
- **Ctrl+Shift+Arrow**: Move panel content to adjacent grid position
- **Ctrl+Alt+Arrow**: Swap panels in grid
- **F11**: Toggle grid cell maximization (zoom current panel)

#### Grid Customization
```typescript
interface GridCustomization {
    uniformSizing: boolean;        // All panels same size
    proportionalSizing: boolean;   // Size based on content
    minPanelSize: { width: number; height: number };
    snapToGrid: boolean;           // Snap splitters to grid positions
    gridSpacing: number;           // Gap between panels
    showGridLines: boolean;        // Visual grid guides
}
```

### Smart Grid Management

#### Auto-Layout Suggestions
```typescript
interface GridLayoutSuggestion {
    confidence: number;           // 0-1 confidence score
    layout: GridLayout;
    reasoning: string;
    suggestedPanels: {
        position: GridPosition;
        path: string;
        purpose: string;
    }[];
}

// Example suggestions based on current directories
const developmentSuggestion: GridLayoutSuggestion = {
    confidence: 0.9,
    layout: 'grid2x2',
    reasoning: 'Detected development project structure',
    suggestedPanels: [
        { position: 'topLeft', path: '/project/src', purpose: 'Source Code' },
        { position: 'topRight', path: '/project/build', purpose: 'Build Output' },
        { position: 'bottomLeft', path: '/project/tests', purpose: 'Test Files' },
        { position: 'bottomRight', path: '/project/docs', purpose: 'Documentation' }
    ]
};
```

#### Context-Aware Grid Layouts
The system can suggest optimal grid layouts based on:
- **Directory Structure Analysis**: Detects project types (React, Node.js, Python, etc.)
- **File Type Patterns**: Groups related file types in logical panels
- **Access Patterns**: Learns from user behavior to suggest frequently used combinations
- **Workflow Templates**: Pre-configured layouts for common tasks

### Grid Layout Presets

#### Developer Presets
```typescript
const developerPresets = {
    webDevelopment: {
        layout: 'grid2x2',
        panels: {
            topLeft: { path: '/project/src', name: 'Source' },
            topRight: { path: '/project/build', name: 'Build' },
            bottomLeft: { path: '/project/tests', name: 'Tests' },
            bottomRight: { path: '/project/docs', name: 'Docs' }
        }
    },
    fullStack: {
        layout: 'grid2x3',
        panels: {
            topRow: [
                { path: '/frontend/src', name: 'Frontend' },
                { path: '/backend/src', name: 'Backend' },
                { path: '/database/migrations', name: 'Database' }
            ],
            bottomRow: [
                { path: '/tests', name: 'Tests' },
                { path: '/docs', name: 'Docs' },
                { path: '/deploy', name: 'Deploy' }
            ]
        }
    }
};
```

#### Content Management Presets
```typescript
const contentPresets = {
    publishing: {
        layout: 'grid2x2',
        panels: {
            topLeft: { path: '/content/drafts', name: 'Drafts' },
            topRight: { path: '/content/review', name: 'Review' },
            bottomLeft: { path: '/content/published', name: 'Published' },
            bottomRight: { path: '/content/assets', name: 'Assets' }
        }
    },
    multiSite: {
        layout: 'grid2x3',
        panels: {
            topRow: [
                { path: '/site1/content', name: 'Site 1' },
                { path: '/site2/content', name: 'Site 2' },
                { path: '/site3/content', name: 'Site 3' }
            ],
            bottomRow: [
                { path: '/shared/assets', name: 'Shared' },
                { path: '/backup', name: 'Backup' },
                { path: '/staging', name: 'Staging' }
            ]
        }
    }
};
```

### Grid Layout Synchronization

#### Cross-Panel Sync in Grids
```typescript
interface GridSyncOptions {
    syncGroups: {
        groupId: string;
        panels: GridPosition[];
        syncType: 'navigation' | 'selection' | 'viewMode' | 'filters';
    }[];
    
    // Example: Sync top row navigation, bottom row independent
    rowSync: boolean;
    columnSync: boolean;
    
    // Advanced sync patterns
    masterPanel: GridPosition;      // One panel drives others
    syncChain: GridPosition[];      // Sequential synchronization
}
```

#### Grid-Specific Operations
- **Broadcast Copy**: Copy files from one panel to all others in row/column
- **Cascade Move**: Move files through workflow stages (panel 1 → 2 → 3)
- **Grid Compare**: Compare files across all grid positions
- **Synchronized Navigation**: Navigate related directory structures together

### Splitter Control

The adjustable splitters between panels provide flexible layout control:

#### Splitter Features
- **Drag Resize**: Mouse drag to adjust panel widths
- **Double-Click Reset**: Double-click splitter to evenly distribute space
- **Keyboard Resize**: Keyboard shortcuts for common arrangements
- **Memory**: Remember splitter positions between sessions
- **Minimum Sizes**: Enforce minimum panel widths for usability

#### Panel Arrangements
```typescript
type PanelArrangement = 
    | { type: 'single', panelId: string }
    | { type: 'dual', ratio: number }
    | { type: 'triple', ratios: [number, number, number] }
    | { type: 'quad', layout: 'grid' | 'row' }
    | { type: 'custom', panels: PanelConfig[] };
```

#### Dynamic Panel Management
- **Add Panel**: Create new panels on demand (up to 6 panels)
- **Remove Panel**: Close panels when not needed
- **Rearrange**: Drag and drop to reorder panels
- **Preset Layouts**: Quick access to common arrangements

## Tab Management

### Tab Lifecycle

Each tab represents an independent navigation session within a panel:

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
    isPinned: boolean;
}
```

#### Tab Creation
- **Ctrl+T**: Create new tab in active panel at current location
- **Middle-Click**: Create new tab from folder (middle-click folder in file list)
- **Drag & Drop**: Drop folder onto tab bar to create new tab
- **Bookmark**: Create tab from bookmark (inherits bookmark's settings)

#### Tab Navigation
- **Ctrl+Tab**: Cycle through tabs in active panel (most recently used order)
- **Ctrl+Shift+Tab**: Cycle through tabs in reverse order
- **Ctrl+1-9**: Jump to specific tab number in active panel
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
    syncNavigation: boolean;    // Navigate multiple panels together
    syncSelection: boolean;     // Mirror selection between panels
    syncViewMode: boolean;      // Match view modes
    syncSort: boolean;          // Match sorting preferences
    syncGroup: string[];        // Which panels to synchronize
}
```

#### Tab Context Menu
Right-click on tab provides context-specific options:

- **Duplicate Tab**: Create copy of current tab in new tab or different panel
- **Move to Panel**: Move tab to another panel
- **Close Others**: Close all other tabs in this panel
- **Close to Right**: Close all tabs to the right of current tab
- **Reopen**: Reopen recently closed tabs
- **Pin Tab**: Pin important tabs to prevent accidental closing
- **Rename Tab**: Custom tab titles for better organization
- **Open in New Panel**: Open tab location in a new panel

## File Operations Between Panels

### Multi-Panel Operations

The multi-panel design enables flexible file operations between any combination of panels:

```rust
// Example: Copy operation between panels
#[tauri::command]
pub async fn copy_between_panels(
    files: Vec<PathBuf>,
    source_panel_id: String,
    target_panel_id: String,
    options: CopyOptions
) -> Result<OperationId, FileError> {
    let destination = get_panel_current_path(&target_panel_id)?;
    copy_files(files, destination, options).await
}
```

#### Keyboard Operations
- **F5 (Copy)**: Copy selected files to target panel (prompts for panel selection)
- **F6 (Move)**: Move selected files to target panel
- **Ctrl+F5**: Copy to specific panel (Ctrl+1-9 to select target)
- **Shift+F5**: Copy with options dialog
- **Alt+F5**: Copy to all other panels

#### Drag & Drop Operations
- **Panel-to-Panel Drag**: Drag files between any panels
- **Multi-Panel Drop**: Hold Ctrl while dropping to copy to multiple panels
- **Smart Targeting**: Visual indicators show valid drop targets
- **Operation Preview**: Preview the operation before confirming

### Operation Feedback

#### Progress Indication
```typescript
interface FileOperation {
    id: string;
    type: 'copy' | 'move' | 'delete' | 'compress';
    source: { panelId: string; paths: string[] };
    destinations: { panelId: string; path: string }[];
    progress: {
        current: number;
        total: number;
        currentFile: string;
        bytesProcessed: number;
        bytesTotal: number;
        speed: number;
        eta: number;
    };
    status: 'running' | 'paused' | 'completed' | 'error' | 'cancelled';
}
```

## Navigation Synchronization

### Multi-Panel Synchronization

Advanced synchronization features for working with multiple panels:

#### Sync Modes
```typescript
type SyncMode = 
    | 'none'        // Independent navigation
    | 'all'         // All panels navigate together
    | 'group'       // Sync specific panel groups
    | 'pair'        // Sync panel pairs (1-2, 3-4, etc.)
    | 'follow'      // One panel follows another
    | 'mirror';     // Bidirectional synchronization
```

#### Use Cases
- **Multi-Location Comparison**: Compare multiple folder structures simultaneously
- **Complex File Organization**: Organize files across multiple hierarchies
- **Development Workflows**: Navigate source, build, and deployment directories together
- **Backup Management**: Compare source with multiple backup locations

### Comparison Features

#### Visual Comparison
- **Multi-Panel Diff**: Highlight differences across all visible panels
- **Consensus View**: Show files common to all panels vs. unique files
- **Change Tracking**: Track changes across panel sessions
- **Sync Status**: Visual indicators for synchronization state

## Layout Customization

### Responsive Design

The multi-panel interface adapts to different screen sizes:

#### Breakpoints
- **Ultra-Wide Screens** (>2560px): Up to 6 panels side by side
- **Wide Screens** (1920-2560px): Up to 4 panels or 2x2 grid
- **Standard Screens** (1366-1920px): Up to 3 panels
- **Narrow Screens** (1024-1366px): Maximum 2 panels
- **Mobile/Tablet** (<1024px): Single panel with panel switching

#### Layout Options
```typescript
interface LayoutSettings {
    maxPanels: number;                       // Maximum panels for screen size
    orientation: 'horizontal' | 'vertical' | 'auto';
    splitterSize: number;
    tabBarPosition: 'top' | 'bottom';
    showStatusBars: boolean;
    compactMode: boolean;
    adaptiveLayout: boolean;                 // Auto-adjust based on content
    panelMinWidth: number;
    gridMode: boolean;                       // Enable 2x2, 2x3 grid layouts
}
```

### Panel Presets

#### Linear Layouts
- **Single**: One panel, full width
- **Classic**: Two panels, 50/50 split (traditional dual-pane)
- **Triple**: Three panels, even distribution
- **Commander**: Two main panels + narrow preview panel

#### Grid Layouts
- **Grid 2x2**: Four panels in 2x2 arrangement - perfect for development workflows
- **Grid 2x3**: Six panels in 2x3 arrangement - ideal for complex multi-site management
- **Grid 3x2**: Six panels in 3x2 arrangement - optimized for version control workflows

#### Specialized Grid Presets
- **Developer Grid**: Source, Build, Test, Deploy in 2x2 grid
- **Content Grid**: Draft, Review, Published, Archive workflow
- **System Admin Grid**: Live, Staging, Backup, Logs monitoring
- **Media Production Grid**: Raw, Processing, Final, Archive pipeline
- **Multi-Site Grid**: Multiple site management with synchronized operations
- **Compare Grid**: Side-by-side comparison of multiple directory structures

## Performance Optimization

### Memory Management
- **Panel Virtualization**: Unload inactive panel contents
- **Smart Caching**: Cache frequently accessed panels
- **Resource Balancing**: Distribute resources across active panels
- **Background Loading**: Load content for inactive panels in background

### Rendering Optimization
- **Panel-Aware Rendering**: Update only visible panels
- **Lazy Panel Loading**: Initialize panels only when needed
- **Shared Resources**: Reuse file system data across panels
- **Efficient Synchronization**: Minimize overhead for synced panels

---

The multi-panel interface provides the foundation for advanced file management, enabling users to work with multiple locations simultaneously while maintaining clear visual organization and intuitive operation flows. This flexible design scales from simple dual-panel usage to complex multi-location workflows.