# Search System Documentation

## Overview

The Nimbus search system provides fast, comprehensive file and content search capabilities with a modern, accessible interface. Built with React 19 and TypeScript, it features real-time search, highlighting, keyboard navigation, and virtualized results for handling thousands of files efficiently.

## Architecture

### Component Hierarchy
```
SearchInterface
├── SearchForm (input, filters, options)
├── SearchResults (standard pagination view)
├── VirtualizedSearchResults (high-performance virtualized view)
├── SearchPagination (navigation controls)
└── SearchHistory (recent searches)
```

### Core Hooks
- `useSearch` - Search state management and API integration
- `useKeyboardNavigation` - Comprehensive keyboard controls
- `useSearchResultContext` - Context menu functionality
- `useActiveSearchResults` - Current search results with pagination

### Backend Integration
- **Tauri Commands**: `start_search`, `cancel_search`
- **Event System**: Real-time result streaming via Tauri events
- **Type Safety**: Full TypeScript interfaces for all data structures

## Features

### 🔍 Search Capabilities
- **Name Search**: Exact and fuzzy matching for file names
- **Content Search**: Full-text search within files
- **Advanced Filters**: Size, date, file type, and location filters
- **Real-time Results**: Streaming search results as they're found
- **Search History**: Recent searches with quick re-execution

### ⚡ Performance Features
- **Virtualization**: Handle 10,000+ results efficiently
- **Parallel Backend**: Multi-threaded Rust search engine
- **Progressive Loading**: Results appear as they're found
- **Memory Optimization**: <200MB RAM for large result sets

### 🎨 User Experience
- **Visual Highlighting**: Search term highlighting with match type colors
- **Keyboard Navigation**: Full keyboard control with shortcuts
- **Context Menus**: Right-click actions for file operations
- **Responsive Design**: Works on all screen sizes
- **Accessibility**: WCAG 2.1 AA compliant

### 🚀 Interactive Features
- **Typeahead Navigation**: Type to jump to matching results
- **Multi-select Support**: Select multiple files for batch operations
- **Quick Actions**: Single-key shortcuts for common operations
- **Drag & Drop**: Drag files from results to other applications

## Usage Guide

### Basic Search

1. **Open Search Interface**: Click search icon or press `Ctrl+F`
2. **Enter Query**: Type filename or content to search for
3. **Navigate Results**: Use arrow keys or click to select
4. **Open Files**: Double-click or press `Enter`

### Advanced Search Options

**Search Types:**
- **Name Only**: Search file and folder names
- **Content**: Search within file contents
- **Both**: Search names and contents simultaneously

**Filters:**
- **File Types**: Documents, images, code, archives
- **Size Range**: Min/max file size filters
- **Date Range**: Modified, created, or accessed dates
- **Location**: Specific folders or drives

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Open search interface |
| `↑/↓` | Navigate results |
| `Page Up/Down` | Jump 10 results |
| `Home/End` | First/last result |
| `Enter` | Open selected file |
| `Space` | Select/deselect file |
| `Ctrl+C` | Copy file path |
| `Ctrl+R` | Reveal in folder |
| `Delete` | Delete selected files |
| `Escape` | Clear selection/close |
| `Type` | Typeahead navigation |

### Context Menu Actions

**Right-click any search result for:**
- **Open** - Open file with default application
- **Copy Path** - Copy full file path to clipboard
- **Copy Name** - Copy filename to clipboard
- **Reveal in Folder** - Show file in file manager
- **Delete** - Move file to trash
- **Properties** - View file details and metadata

## Developer Guide

### Component API

#### SearchResults Component
```typescript
interface SearchResultsProps {
  onResultClick?: (result: SearchResult) => void;
  onResultDoubleClick?: (result: SearchResult) => void;
  className?: string;
}
```

#### VirtualizedSearchResults Component
```typescript
interface VirtualizedSearchResultsProps {
  onResultClick?: (result: SearchResult) => void;
  onResultDoubleClick?: (result: SearchResult) => void;
  className?: string;
  itemHeight?: number;      // Default: 120px
  maxHeight?: number;       // Default: 600px
}
```

### Hook Usage

#### useSearch Hook
```typescript
const {
  // State
  searchQuery,
  results,
  isSearching,
  error,
  
  // Actions
  startSearch,
  cancelSearch,
  clearResults
} = useSearch();
```

#### useKeyboardNavigation Hook
```typescript
const navigation = useKeyboardNavigation(results, {
  onResultSelect: handleSelect,
  onResultActivate: handleActivate,
  enableQuickActions: true,
  enableTypeahead: true,
  wrapNavigation: true
});
```

### Search Result Interface
```typescript
interface SearchResult {
  path: string;              // Full file path
  name: string;              // File name
  size: number;              // File size in bytes
  modified: string;          // ISO timestamp
  matchType: MatchType;      // Type of match found
  relevanceScore: number;    // 0-100 relevance score
  matches?: ContentMatch[];  // Content match details
}

interface ContentMatch {
  lineNumber: number;        // Line where match was found
  lineContent: string;       // Full line content
  matchStart: number;        // Match start position
  matchEnd: number;          // Match end position
}

type MatchType = 
  | 'exact_name'    // Exact filename match
  | 'fuzzy_name'    // Fuzzy filename match
  | 'content'       // Content match
  | 'extension'     // File extension match
  | 'directory';    // Directory name match
```

### Backend Integration

#### Starting a Search
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const searchId = await invoke<string>('start_search', {
  query: {
    rootPath: '/home/user',
    namePattern: 'document',
    contentPattern: 'important',
    options: {
      caseSensitive: false,
      useRegex: false,
      includeHidden: false,
      maxResults: 1000
    }
  }
});
```

#### Listening for Results
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<SearchResult>('search-result', (event) => {
  const result = event.payload;
  addSearchResult(result);
});
```

## Styling and Theming

### CSS Classes

**Search Results:**
- `.search-results` - Main container
- `.result-item` - Individual result item
- `.result-item.selected` - Selected result (keyboard navigation)
- `.result-header` - Result name and badges
- `.result-details` - Path and metadata
- `.content-matches` - Content match preview

**Visual States:**
- `.card-enhanced` - Modern card styling
- `.hover-lift` - Hover elevation effect
- `.search-result-enter-staggered` - Staggered entrance animation

**Highlighting:**
- `.search-match-highlight` - General match highlighting
- `.search-match-highlight.fuzzy` - Fuzzy match styling
- `.search-match-highlight.content` - Content match styling

### Animation Classes
- `.fadeIn` - Fade in animation
- `.slideIn` - Slide in animation
- `.bounceIn` - Bounce entrance animation
- `.skeleton-loading` - Loading skeleton animation

### Dark Mode Support
All components automatically adapt to system dark mode preferences using CSS media queries and CSS custom properties.

## Performance Optimization

### Virtualization
For large result sets (>100 items), use `VirtualizedSearchResults`:
- Only renders visible items (5-10 at a time)
- Smooth scrolling with overscan buffering
- Memory usage independent of result count
- 60fps scrolling performance

### Search Optimization
- **Incremental Results**: Results stream in real-time
- **Parallel Processing**: Multi-threaded backend search
- **Smart Caching**: Recent searches cached for instant re-execution
- **Debounced Input**: 300ms debounce on search input

### Memory Management
- **Result Limits**: Configurable maximum results (default: 10,000)
- **Garbage Collection**: Automatic cleanup of old search sessions
- **Event Cleanup**: Proper event listener cleanup on unmount

## Accessibility Features

### Keyboard Navigation
- **Tab Order**: Logical tab sequence through interface
- **Focus Indicators**: Clear visual focus indicators
- **Screen Reader**: ARIA labels and live regions
- **Keyboard Shortcuts**: All mouse actions have keyboard equivalents

### Screen Reader Support
- **Live Regions**: Search status and result count announcements
- **ARIA Labels**: Descriptive labels for all interactive elements
- **Role Attributes**: Proper semantic roles for custom components
- **Alt Text**: Alternative text for icons and visual indicators

### Visual Accessibility
- **High Contrast**: Sufficient color contrast ratios
- **Reduced Motion**: Respects `prefers-reduced-motion` setting
- **Scalable Text**: Responsive to browser zoom levels
- **Focus Indicators**: Clear focus outlines for keyboard navigation

## Troubleshooting

### Common Issues

**Search Not Working:**
1. Check backend connection
2. Verify file permissions
3. Check search query syntax
4. Review browser console for errors

**Slow Performance:**
1. Enable virtualization for large results
2. Reduce search scope with filters
3. Limit maximum results
4. Check available system memory

**Keyboard Navigation Not Working:**
1. Ensure search results have focus
2. Check for JavaScript errors
3. Verify event listeners are attached
4. Test with different browsers

### Debug Mode
Enable debug logging in development:
```typescript
localStorage.setItem('search-debug', 'true');
```

This enables detailed logging for search operations, keyboard events, and performance metrics.

## Future Enhancements

### Planned Features
- **Saved Searches**: Save and organize frequent search queries
- **Search Filters UI**: Visual filter builder interface  
- **File Previews**: Quick preview panel for selected files
- **Search Analytics**: Usage statistics and optimization suggestions
- **Plugin System**: Extensible search providers and result processors

### Performance Improvements
- **Web Workers**: Move search processing to background threads
- **IndexedDB Caching**: Persistent result caching
- **Streaming Compression**: Compressed result streaming
- **Priority Queue**: Intelligent result prioritization

---

This documentation covers the complete search system implementation. For additional technical details, see the individual component documentation files and inline code comments.