# Core Features

## Overview

Nimbus provides a comprehensive set of core file management features designed for power users who demand efficiency, reliability, and cross-platform consistency. All features are built with keyboard navigation as a first-class citizen while maintaining full mouse support.

## Dual-Pane File Management

### Side-by-Side Interface
- **Independent Panes**: Two file browser panes that operate independently
- **Resizable Split**: Adjustable splitter for customizing pane widths
- **Active Pane Indicator**: Clear visual indication of which pane has focus
- **Synchronized Operations**: Easy file operations between panes (copy, move, compare)

### Multi-Tab Support
- **Tab Management**: Multiple tabs per pane for efficient workflow
- **Tab Persistence**: Restore open tabs between application sessions
- **Tab Navigation**: Keyboard shortcuts and mouse controls for tab switching
- **Tab Context**: Each tab maintains its own navigation history and selection state

**Keyboard Shortcuts**:
- `Tab`: Switch active pane
- `Ctrl+T`: Open new tab in active pane
- `Ctrl+W`: Close current tab
- `Ctrl+Shift+T`: Reopen recently closed tab
- `Ctrl+1-9`: Switch to specific tab number

## File Operations

### Basic Operations
All file operations support both single files and multiple selections with progress feedback for large operations.

#### Copy & Move
```rust
// Example API structure
#[tauri::command]
pub async fn copy_files(
    sources: Vec<PathBuf>,
    destination: PathBuf,
    options: CopyOptions
) -> Result<OperationId, FileError>

pub struct CopyOptions {
    pub overwrite_policy: OverwritePolicy,
    pub preserve_timestamps: bool,
    pub verify_integrity: bool,
    pub show_progress: bool,
}
```

**Features**:
- **Conflict Resolution**: Interactive prompts for existing files (overwrite, skip, rename)
- **Progress Tracking**: Real-time progress for large operations with ETA
- **Integrity Verification**: Optional checksum verification for critical operations
- **Cancellation**: Ability to cancel long-running operations
- **Undo Support**: Undo recent file operations when possible

#### Delete Operations
- **Safe Deletion**: Move to trash/recycle bin by default
- **Permanent Deletion**: Direct deletion with confirmation for sensitive files
- **Secure Deletion**: Overwrite file data before deletion (configurable)
- **Batch Operations**: Delete multiple files with single confirmation

#### Create & Rename
- **New Folders**: Create folder hierarchies with intelligent naming
- **File Renaming**: In-place renaming with validation
- **Batch Rename**: Pattern-based renaming for multiple files
- **Template System**: Predefined naming templates for common patterns

### Advanced Operations

#### File Comparison
- **Binary Comparison**: Byte-by-byte file comparison for exact matching
- **Timestamp Sync**: Compare and synchronize file modification times
- **Size Analysis**: Quick size-based filtering and comparison
- **Hash Verification**: MD5, SHA-1, SHA-256 hash calculation and comparison

#### Symbolic Links
- **Link Creation**: Create symbolic and hard links (platform-dependent)
- **Link Resolution**: Navigate to link targets
- **Link Visualization**: Clear indication of linked files and folders
- **Broken Link Detection**: Identify and manage broken symbolic links

## Navigation & Browsing

### Path Navigation
- **Breadcrumb Navigation**: Clickable path segments for quick navigation
- **Address Bar**: Editable address bar for direct path entry
- **Bookmarks**: Save frequently accessed locations
- **History**: Forward/back navigation with full history stack

### File Listing

#### View Modes
- **List View**: Compact file listing with essential information
- **Details View**: Detailed information including permissions, owner, dates
- **Grid View**: Icon-based grid layout for visual browsing
- **Tree View**: Hierarchical folder structure (optional sidebar)

#### Sorting & Filtering
```typescript
interface SortOptions {
    field: 'name' | 'size' | 'modified' | 'created' | 'type' | 'extension';
    order: 'ascending' | 'descending';
    foldersFirst: boolean;
    caseSensitive: boolean;
}

interface FilterOptions {
    namePattern: string;        // Wildcards or regex
    sizeRange: [number, number];
    dateRange: [Date, Date];
    fileTypes: string[];        // Extensions
    attributes: FileAttributes; // Hidden, system, etc.
}
```

**Features**:
- **Multi-Level Sorting**: Primary and secondary sort criteria
- **Quick Filters**: Instant filtering by file type, size, date
- **Custom Columns**: Add/remove columns based on user preferences
- **Column Resizing**: Adjust column widths and remember preferences

### File Selection

#### Selection Methods
- **Single Selection**: Click to select individual files
- **Range Selection**: Shift+click for contiguous selection
- **Multi-Selection**: Ctrl+click for non-contiguous selection
- **Pattern Selection**: Select files matching patterns (Ctrl+A for all)
- **Invert Selection**: Reverse current selection

#### Selection Indicators
- **Visual Feedback**: Clear highlighting of selected items
- **Selection Counter**: Display count and total size of selected files
- **Selection Persistence**: Maintain selection across navigation (optional)

## Search Functionality

### Quick Search
- **Incremental Search**: Type to filter current directory contents
- **Live Filtering**: Real-time results as you type
- **Highlight Matches**: Visual highlighting of matching text
- **Navigation**: Jump between matches with arrow keys

### Advanced Search
Covered in detail in [File Search](./search.md) documentation.

**Summary Features**:
- Recursive directory search with parallel processing
- Multiple search criteria (name, content, size, date, attributes)
- Regular expression support
- Streaming results with cancellation
- Search result navigation

## File Information & Metadata

### File Properties
- **Basic Properties**: Size, dates (created, modified, accessed), attributes
- **Extended Attributes**: Platform-specific metadata (NTFS streams, xattrs)
- **Media Information**: Duration, dimensions, codec info for media files
- **Archive Information**: Compression ratio, contents for archive files

### Thumbnails & Previews
- **Image Thumbnails**: Generate and cache thumbnails for image files
- **Document Previews**: Preview first page of PDF and document files
- **Video Previews**: Extract frame thumbnails from video files
- **Custom Previews**: Plugin system for additional file type previews

## Keyboard Navigation

### Core Shortcuts
Following Total Commander conventions where applicable:

| Shortcut | Action | Description |
|----------|---------|-------------|
| `Tab` | Switch Pane | Toggle between left and right panes |
| `Enter` | Open/Execute | Open folders, execute files |
| `Backspace` | Parent Directory | Navigate to parent directory |
| `F2` | Rename | Rename selected file/folder |
| `F3` | View | Open file in built-in viewer |
| `F4` | Edit | Open file in default editor |
| `F5` | Copy | Copy selected files to other pane |
| `F6` | Move | Move selected files to other pane |
| `F7` | Create Folder | Create new folder |
| `F8` | Delete | Delete selected files |
| `F9` | Properties | Show file properties |
| `F10` | Menu | Open context menu |

### Navigation Shortcuts
| Shortcut | Action |
|----------|---------|
| `Ctrl+Left/Right` | Navigate between tabs |
| `Ctrl+Up` | Go to parent directory |
| `Ctrl+Down` | Enter selected directory |
| `Alt+Left/Right` | History navigation (back/forward) |
| `Ctrl+D` | Add current location to bookmarks |
| `Ctrl+L` | Focus address bar |

### Selection Shortcuts
| Shortcut | Action |
|----------|---------|
| `Space` | Select/deselect current item |
| `Ctrl+A` | Select all |
| `Ctrl+Shift+A` | Deselect all |
| `Num+` | Select by pattern |
| `Num-` | Deselect by pattern |
| `Num*` | Invert selection |

## Error Handling & Recovery

### Error Management
- **Graceful Degradation**: Continue operations when individual items fail
- **Detailed Error Messages**: Clear explanations of what went wrong
- **Retry Mechanisms**: Automatic retry for network-related failures
- **Error Logging**: Maintain logs for troubleshooting complex issues

### Recovery Options
- **Operation Rollback**: Undo changes when operations fail partway
- **Partial Success Handling**: Report which files succeeded/failed
- **Manual Resolution**: Tools to resolve conflicts and issues manually
- **Backup Integration**: Optional backup before destructive operations

## Performance Features

### Optimization
- **Lazy Loading**: Load file information on-demand for large directories
- **Virtual Scrolling**: Handle directories with 100k+ files smoothly
- **Background Operations**: Non-blocking file operations with progress feedback
- **Caching**: Intelligent caching of file metadata and thumbnails

### Memory Management
- **Bounded Memory**: Configurable limits on cache sizes
- **Cleanup**: Automatic cleanup of temporary files and caches
- **Resource Monitoring**: Built-in tools to monitor resource usage
- **Garbage Collection**: Proactive cleanup of unused resources

## Accessibility Features

### Screen Reader Support
- **ARIA Labels**: Proper labeling for all interface elements
- **Screen Reader Announcements**: Status updates and operation feedback
- **Keyboard-Only Navigation**: Complete functionality without mouse
- **High Contrast**: Support for system high-contrast themes

### Visual Accessibility
- **Scalable UI**: Respect system DPI and font size settings
- **Color Blind Support**: Use shape and pattern in addition to color
- **Customizable Themes**: Light, dark, and custom color schemes
- **Font Options**: Configurable fonts and sizes for better readability

## Configuration & Customization

### Settings Management
- **Persistent Settings**: All preferences saved automatically
- **Export/Import**: Share configurations between installations
- **Profile Support**: Multiple configuration profiles for different workflows
- **Reset Options**: Easy reset to default settings

### Customization Options
- **Toolbar Customization**: Add/remove/rearrange toolbar buttons
- **Column Configuration**: Choose which columns to display and in what order
- **Keyboard Shortcuts**: Customize keyboard mappings
- **Color Themes**: Built-in and custom theme support

---

These core features provide a solid foundation for efficient file management while maintaining the flexibility and extensibility that makes Nimbus suitable for a wide range of users and workflows.