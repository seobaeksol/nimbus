# Quick Start Guide

## Welcome to Nimbus

Nimbus is a modern, cross-platform file manager that combines the power of dual-pane navigation with advanced features like archive support, remote file access, and powerful search capabilities. This guide will help you get started quickly.

## First Launch

### Initial Setup

When you first launch Nimbus, you'll see the dual-pane interface:

```
┌─ Nimbus File Manager ────────────────────────────────────────────┐
│ File  Edit  View  Tools  Help                               ⚙️📡 │
├─────────────────────────┬─────────────────────────────────────────┤
│ Left Pane               │ Right Pane                              │
│ 📂 /home/user          │ 📂 /home/user                          │
│                         │                                         │
│ 📁 Documents            │ 📁 Documents                            │
│ 📁 Downloads            │ 📁 Downloads                            │
│ 📁 Pictures             │ 📁 Pictures                             │
│ 📁 Music                │ 📁 Music                                │
│ 📄 readme.txt           │ 📄 readme.txt                          │
│                         │                                         │
│ 127 items, 2.4 GB      │ 127 items, 2.4 GB                      │
└─────────────────────────┴─────────────────────────────────────────┘
```

### Key Interface Elements

- **Left & Right Panes**: Two independent file browsers for efficient file management
- **Active Pane**: The pane with the highlighted border is currently active
- **Tab Bar**: Multiple tabs per pane (visible when you have more than one tab)
- **Status Bar**: Shows file counts, sizes, and current operation status
- **Splitter**: Drag the center divider to adjust pane widths

## Essential Navigation

### Basic Movement

| Action | Method |
|--------|---------|
| **Navigate into folder** | Double-click folder or press Enter |
| **Go up one level** | Click "Up" button or press Backspace |
| **Switch between panes** | Press Tab key |
| **Type to find** | Start typing to filter files in current directory |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|---------|
| `Tab` | Switch active pane |
| `Enter` | Open file/folder |
| `Backspace` | Go to parent directory |
| `F3` | View file content |
| `F5` | Copy selected files to other pane |
| `F6` | Move selected files to other pane |
| `F7` | Create new folder |
| `F8` | Delete selected files |

## Essential File Operations

### Selecting Files

- **Single file**: Click once to select
- **Multiple files**: Hold Ctrl and click additional files
- **Range selection**: Click first file, hold Shift, click last file
- **Select all**: Press Ctrl+A
- **Invert selection**: Press Ctrl+Shift+A

### Copying and Moving Files

#### Method 1: Function Keys (Recommended)
1. Select files in one pane
2. Press **F5** to copy or **F6** to move to the other pane
3. Confirm the operation in the dialog

#### Method 2: Drag and Drop
1. Select files in source pane
2. Drag files to destination pane
3. Choose action from context menu (Copy, Move, or Create Link)

#### Method 3: Context Menu
1. Right-click selected files
2. Choose "Copy" or "Cut"
3. Navigate to destination and right-click → "Paste"

### Creating and Deleting

#### Create New Folder
- Press **F7** or right-click empty space → "New Folder"
- Type folder name and press Enter

#### Delete Files
- Select files and press **F8** or Delete key
- Choose between "Move to Trash" (safe) or "Delete Permanently"

## Working with Archives

### Browsing Archive Contents

Nimbus treats archives like regular folders:

1. **Navigate into archive**: Double-click a ZIP, TAR, 7z, or RAR file
2. **Browse contents**: Use normal navigation (Enter, Backspace, arrow keys)
3. **Path shows archive**: Notice the path shows `archive.zip!/internal/path`
4. **Exit archive**: Navigate up to parent directory

### Extracting Files

#### Quick Extraction
1. Navigate into archive
2. Select files to extract
3. Press **F5** to copy to other pane (automatic extraction)

#### Full Archive Extraction
1. Select archive file (don't enter it)
2. Right-click → "Extract All..."
3. Choose destination and options
4. Click "Extract"

### Creating Archives

1. Select files and folders to archive
2. Right-click → "Create Archive..."
3. Choose format (ZIP, TAR.GZ, etc.)
4. Set compression options
5. Click "Create"

## Quick Search

### In-Directory Search (Quick Filter)
1. Navigate to directory you want to search
2. Press **Ctrl+F** or start typing
3. Type search pattern (supports wildcards like `*.txt`)
4. Use arrow keys to navigate matches
5. Press Escape to clear filter

### Advanced Search
1. Press **Ctrl+Shift+F** to open Advanced Search
2. Set search location (current directory or browse)
3. Configure search criteria:
   - **Filename**: Pattern to match (supports wildcards and regex)
   - **Content**: Text to search inside files
   - **Size**: File size range
   - **Date**: Date modified range
   - **File Types**: Filter by file extensions or categories
4. Click "Search" to start
5. Results appear in real-time
6. Double-click result to open file or right-click to show in pane

## Managing Tabs

### Creating Tabs
- **New tab**: Press Ctrl+T or right-click tab bar → "New Tab"
- **Duplicate tab**: Right-click tab → "Duplicate Tab"
- **Open folder in new tab**: Middle-click folder or Ctrl+Enter

### Tab Navigation
- **Switch tabs**: Ctrl+Tab (next) or Ctrl+Shift+Tab (previous)
- **Close tab**: Ctrl+W or middle-click tab
- **Close other tabs**: Right-click tab → "Close Others"

### Useful Tab Features
- Each tab remembers its navigation history (back/forward)
- Tabs persist between application sessions
- Pin important tabs to prevent accidental closing

## Customizing Your Workspace

### View Options
- **Change view mode**: Click view buttons in toolbar (List, Details, Grid)
- **Sort files**: Click column headers or use View menu
- **Show/hide hidden files**: Press Ctrl+H or View menu
- **Adjust columns**: Right-click column headers to add/remove columns

### Layout Adjustments
- **Resize panes**: Drag the center splitter
- **Single pane mode**: Press F12 to toggle between dual and single pane
- **Toolbar customization**: Right-click toolbar → "Customize"

### Themes and Appearance
1. Open Settings (Ctrl+Comma or gear icon)
2. Go to "Appearance" section
3. Choose theme:
   - **System**: Follow system theme
   - **Light**: Light theme
   - **Dark**: Dark theme
4. Adjust font size and other visual preferences

## Common Tasks

### File Comparison
1. Navigate both panes to directories you want to compare
2. Use Tools → "Compare Directories" or press Ctrl+Shift+C
3. Review differences highlighted in both panes
4. Use "Synchronize" to copy differences between directories

### Batch File Renaming
1. Select multiple files
2. Press F2 or right-click → "Batch Rename"
3. Use patterns like:
   - `Photo_###.jpg` (### becomes numbers)
   - `Document_{original}` ({original} keeps original name)
   - `2024_01_{counter}` ({counter} adds sequential numbers)

### Opening Files with Specific Applications
- **Default application**: Double-click or press Enter
- **Choose application**: Right-click → "Open With"
- **Built-in viewer**: Press F3 to view without external application

## Getting More Help

### Built-in Help
- Press **F1** for context-sensitive help
- Check the status bar for hints about current operations
- Hover over buttons for tooltips

### Keyboard Shortcut Reference
- Press **Ctrl+/** to see all keyboard shortcuts
- Most functions also available via right-click context menus

### Online Resources
- **User Manual**: Complete documentation with advanced features
- **Video Tutorials**: Step-by-step guides for complex operations  
- **Community Forum**: Get help from other users
- **Bug Reports**: Report issues for quick resolution

## Next Steps

Now that you know the basics:

1. **Explore Advanced Search**: Try content search and regular expressions
2. **Set Up Remote Connections**: Connect to FTP/SFTP servers
3. **Try Plugin System**: Install plugins for additional file types
4. **Customize Shortcuts**: Adapt keyboard shortcuts to your workflow
5. **Bookmark Frequently Used Locations**: Create bookmarks for quick access

---

**Pro Tip**: Nimbus is designed to be keyboard-efficient. Learn the function key shortcuts (F3-F8) for maximum productivity!

Remember: Both panes work independently, so you can browse different locations and easily transfer files between them. The Tab key is your friend for switching between panes quickly.