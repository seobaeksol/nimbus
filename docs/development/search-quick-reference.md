# Search System Quick Reference

## Quick Start

### Basic Implementation
```tsx
import { SearchResults } from '@/components/search/SearchResults';
import { useSearch } from '@/hooks/useSearch';

function MySearchComponent() {
  const { results, startSearch } = useSearch();
  
  const handleSearch = (query: string) => {
    startSearch({
      rootPath: '/home/user',
      namePattern: query,
      options: { caseSensitive: false }
    });
  };

  return (
    <SearchResults 
      onResultDoubleClick={(result) => openFile(result.path)}
    />
  );
}
```

### Virtualized for Large Results
```tsx
import { VirtualizedSearchResults } from '@/components/search/VirtualizedSearchResults';

// Use for >100 results
<VirtualizedSearchResults 
  itemHeight={120}
  maxHeight={600}
  onResultClick={handleClick}
/>
```

## Essential Components

| Component | Use Case | Performance |
|-----------|----------|-------------|
| `SearchResults` | <100 results | Standard |
| `VirtualizedSearchResults` | >100 results | High |
| `SearchForm` | Input interface | N/A |
| `SearchPagination` | Navigation | Standard |

## Key Hooks

```tsx
// Main search hook
const { results, startSearch, cancelSearch } = useSearch();

// Keyboard navigation
const nav = useKeyboardNavigation(results, {
  onResultActivate: openFile,
  enableTypeahead: true
});

// Context menu
const ctx = useSearchResultContext({
  onOpenFile: openFile,
  onCopyPath: copyToClipboard
});
```

## Search Query Structure

```typescript
interface SearchQuery {
  rootPath: string;           // Required: where to search
  namePattern?: string;       // Filename pattern
  contentPattern?: string;    // Content search
  sizeFilter?: {              // Size constraints
    minSize: number;
    maxSize: number;
    unit: 'bytes' | 'kb' | 'mb' | 'gb';
  };
  options: {
    caseSensitive: boolean;
    useRegex: boolean;
    includeHidden: boolean;
    maxResults?: number;
  };
}
```

## Keyboard Shortcuts

| Key | Action | Implementation |
|-----|--------|----------------|
| `↑/↓` | Navigate | `useKeyboardNavigation` |
| `Enter` | Activate | `onResultActivate` |
| `Ctrl+C` | Copy path | Context menu action |
| `Escape` | Clear | Built-in handler |
| Type | Search | Typeahead buffer |

## Styling Quick Fixes

```css
/* Selection highlight */
.result-item.selected {
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

/* Match highlighting */
.search-match-highlight {
  background: linear-gradient(120deg, #fef3c7, #fed7aa);
  color: #92400e;
  font-weight: 600;
}

/* Loading animation */
.skeleton-loading {
  animation: shimmer 1.5s infinite;
}
```

## Common Patterns

### Handle Search Results
```tsx
const handleResultClick = useCallback((result: SearchResult) => {
  // Single click - select
  setSelectedFile(result);
}, []);

const handleResultDoubleClick = useCallback((result: SearchResult) => {
  // Double click - open
  openFile(result.path);
}, []);
```

### Error Handling
```tsx
const { error, isSearching } = useSearch();

if (error) {
  return <div className="search-error">Error: {error}</div>;
}

if (isSearching && results.length === 0) {
  return <div className="search-loading">Searching...</div>;
}
```

### Performance Optimization
```tsx
// Use virtualization threshold
const ResultComponent = results.length > 100 
  ? VirtualizedSearchResults 
  : SearchResults;

// Memoize expensive operations
const sortedResults = useMemo(() => 
  results.sort((a, b) => b.relevanceScore - a.relevanceScore),
  [results]
);
```

## Backend Integration

### Start Search
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const searchId = await invoke<string>('start_search', { query });
```

### Listen for Results  
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<SearchResult>('search-result', (event) => {
  addResult(event.payload);
});
```

### Cancel Search
```typescript
await invoke('cancel_search', { searchId });
```

## Testing

### Unit Tests
```typescript
import { render, fireEvent } from '@testing-library/react';
import { SearchResults } from '@/components/search/SearchResults';

test('handles result click', () => {
  const handleClick = jest.fn();
  const { getByText } = render(
    <SearchResults onResultClick={handleClick} />
  );
  
  fireEvent.click(getByText('test-file.txt'));
  expect(handleClick).toHaveBeenCalled();
});
```

### Integration Tests
```typescript
import { invoke } from '@tauri-apps/api/tauri';

test('search integration', async () => {
  const searchId = await invoke('start_search', {
    query: { rootPath: '/test', namePattern: '*.txt' }
  });
  
  expect(searchId).toBeDefined();
});
```

## Debugging

### Enable Debug Logging
```typescript
localStorage.setItem('search-debug', 'true');
```

### Common Debug Points
```typescript
// Hook state
console.log('Search state:', { results, isSearching, error });

// Keyboard navigation
console.log('Selected index:', selectedIndex);

// Performance metrics
console.time('search-render');
// ... render code
console.timeEnd('search-render');
```

## Migration Guide

### From Basic to Virtualized
```tsx
// Before
<SearchResults onResultClick={handleClick} />

// After (for better performance)
<VirtualizedSearchResults 
  onResultClick={handleClick}
  itemHeight={120}
  maxHeight={600}
/>
```

### Adding Keyboard Navigation
```tsx
// Add to existing component
const navigation = useKeyboardNavigation(results, {
  onResultSelect: handleSelect,
  onResultActivate: handleActivate
});

// Update container
<div ref={navigation.containerRef}>
  {/* existing results */}
</div>
```

## File Structure

```
src/
├── components/search/
│   ├── SearchResults.tsx          # Standard results view
│   ├── VirtualizedSearchResults.tsx # High-performance view
│   ├── SearchForm.tsx             # Search input
│   ├── SearchPagination.tsx       # Navigation
│   └── SearchAnimations.css       # Visual effects
├── hooks/
│   ├── useSearch.ts               # Main search hook
│   ├── useKeyboardNavigation.ts   # Keyboard controls
│   └── useSearchResultContext.ts  # Context menus
└── utils/
    └── searchHighlight.tsx        # Text highlighting
```

## Performance Targets

| Metric | Target | Notes |
|--------|---------|-------|
| Initial render | <100ms | First paint |
| Result streaming | <50ms | Per result |
| Keyboard response | <16ms | 60fps |
| Memory usage | <200MB | Large datasets |
| Scroll performance | 60fps | Virtualized |

---

*For detailed API documentation, see `/docs/features/search-system.md`*