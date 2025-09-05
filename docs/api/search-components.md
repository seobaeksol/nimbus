# Search Components API Reference

## Components

### SearchResults

Standard search results component with pagination support.

```tsx
import { SearchResults } from '@/components/search/SearchResults';
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `onResultClick` | `(result: SearchResult) => void` | - | Callback for single click |
| `onResultDoubleClick` | `(result: SearchResult) => void` | - | Callback for double click |
| `className` | `string` | `''` | Additional CSS classes |

#### Features
- ✅ Pagination support
- ✅ Keyboard navigation
- ✅ Context menus
- ✅ Search highlighting
- ✅ Loading states
- ✅ Error handling

#### Performance
- **Recommended**: <100 results
- **Memory**: ~1MB per 1000 results
- **Render time**: ~50ms for 50 results

---

### VirtualizedSearchResults

High-performance virtualized search results for large datasets.

```tsx
import { VirtualizedSearchResults } from '@/components/search/VirtualizedSearchResults';
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `onResultClick` | `(result: SearchResult) => void` | - | Callback for single click |
| `onResultDoubleClick` | `(result: SearchResult) => void` | - | Callback for double click |
| `className` | `string` | `''` | Additional CSS classes |
| `itemHeight` | `number` | `120` | Height of each result item (px) |
| `maxHeight` | `number` | `600` | Maximum container height (px) |

#### Features
- ✅ Virtual scrolling
- ✅ Keyboard navigation
- ✅ Context menus
- ✅ Search highlighting
- ✅ Performance stats
- ✅ Memory optimization

#### Performance
- **Recommended**: >100 results
- **Memory**: Constant (~50MB regardless of result count)
- **Render time**: ~10ms regardless of total results
- **Scroll**: 60fps smooth scrolling

---

### SearchForm

Search input form with filters and options.

```tsx
import { SearchForm } from '@/components/search/SearchForm';
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `onSearch` | `(query: SearchQuery) => void` | - | Search submission callback |
| `initialQuery` | `SearchQuery` | - | Initial form values |
| `disabled` | `boolean` | `false` | Disable form inputs |
| `placeholder` | `string` | - | Input placeholder text |

#### Features
- ✅ Advanced filters UI
- ✅ Search history
- ✅ Keyboard shortcuts
- ✅ Form validation
- ✅ Real-time preview

---

### SearchPagination

Pagination controls for search results.

```tsx
import { SearchPagination } from '@/components/search/SearchPagination';
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `searchId` | `string` | - | Current search ID |
| `currentPage` | `number` | - | Current page (0-indexed) |
| `pageSize` | `number` | - | Results per page |
| `totalPages` | `number` | - | Total page count |
| `totalResults` | `number` | - | Total result count |
| `className` | `string` | `''` | Additional CSS classes |

#### Features
- ✅ Page navigation
- ✅ Jump to page
- ✅ Results summary
- ✅ Keyboard shortcuts

---

## Hooks

### useSearch

Main search state management hook.

```tsx
import { useSearch } from '@/hooks/useSearch';

const {
  // State
  searchQuery,
  results,
  isSearching,
  searchId,
  totalResults,
  error,
  pagination,
  
  // Actions
  startSearch,
  cancelSearch,
  clearResults
} = useSearch();
```

#### Return Value

| Property | Type | Description |
|----------|------|-------------|
| `searchQuery` | `SearchQuery \| null` | Current search query |
| `results` | `SearchResult[]` | Search results array |
| `isSearching` | `boolean` | Search in progress |
| `searchId` | `string \| null` | Current search ID |
| `totalResults` | `number` | Total result count |
| `error` | `string \| null` | Error message if any |
| `pagination` | `PaginationInfo \| null` | Pagination state |

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `startSearch` | `(query: SearchQuery) => Promise<string>` | Start new search |
| `cancelSearch` | `() => Promise<void>` | Cancel current search |
| `clearResults` | `() => void` | Clear all results |

---

### useKeyboardNavigation

Keyboard navigation hook for search results.

```tsx
import { useKeyboardNavigation } from '@/hooks/useKeyboardNavigation';

const navigation = useKeyboardNavigation(results, {
  onResultSelect: handleSelect,
  onResultActivate: handleActivate,
  enableQuickActions: true,
  enableTypeahead: true,
  wrapNavigation: true
});
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `results` | `SearchResult[]` | Search results to navigate |
| `options` | `KeyboardNavigationOptions` | Navigation configuration |

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `onResultSelect` | `(result: SearchResult, index: number) => void` | - | Selection callback |
| `onResultActivate` | `(result: SearchResult, index: number) => void` | - | Activation callback |
| `onEscape` | `() => void` | - | Escape key callback |
| `onSearch` | `() => void` | - | Search focus callback |
| `enableQuickActions` | `boolean` | `true` | Enable quick key actions |
| `enableTypeahead` | `boolean` | `true` | Enable typeahead search |
| `wrapNavigation` | `boolean` | `true` | Wrap at start/end |

#### Return Value

| Property | Type | Description |
|----------|------|-------------|
| `selectedIndex` | `number` | Currently selected index |
| `selectedResult` | `SearchResult \| null` | Selected result object |
| `isNavigating` | `boolean` | Navigation active |
| `typeaheadBuffer` | `string` | Current typeahead text |
| `containerRef` | `RefObject<HTMLElement>` | Container ref |
| `setSelectedIndex` | `(index: number) => void` | Set selection |
| `clearSelection` | `() => void` | Clear selection |
| `shortcuts` | `ShortcutInfo` | Keyboard shortcut info |

---

### useSearchResultContext

Context menu hook for search results.

```tsx
import { useSearchResultContext } from '@/hooks/useSearchResultContext';

const context = useSearchResultContext({
  onOpenFile: openFile,
  onRevealInFolder: revealFile,
  onCopyPath: copyPath,
  onCopyName: copyName,
  onDeleteFile: deleteFiles,
  onViewProperties: showProperties
});
```

#### Options

| Option | Type | Description |
|--------|------|-------------|
| `onOpenFile` | `(result: SearchResult) => void` | Open file handler |
| `onRevealInFolder` | `(result: SearchResult) => void` | Reveal in folder |
| `onCopyPath` | `(result: SearchResult) => void` | Copy path handler |
| `onCopyName` | `(result: SearchResult) => void` | Copy name handler |
| `onDeleteFile` | `(results: SearchResult[]) => void` | Delete handler |
| `onViewProperties` | `(result: SearchResult) => void` | Properties handler |

#### Return Value

| Property | Type | Description |
|----------|------|-------------|
| `contextMenu` | `SearchResultContextMenuState` | Menu state |
| `showContextMenu` | `(event: MouseEvent, results: SearchResult[]) => void` | Show menu |
| `hideContextMenu` | `() => void` | Hide menu |
| `getContextMenuItems` | `() => ContextMenuItem[]` | Get menu items |
| `actions` | `object` | Individual action methods |

---

### useActiveSearchResults

Hook for accessing current search results with pagination.

```tsx
import { useActiveSearchResults } from '@/hooks/useSearch';

const {
  results,
  isSearching,
  searchId,
  totalResults,
  error,
  pagination,
  query
} = useActiveSearchResults();
```

#### Return Value

| Property | Type | Description |
|----------|------|-------------|
| `results` | `SearchResult[]` | Current page results |
| `isSearching` | `boolean` | Search in progress |
| `searchId` | `string \| null` | Current search ID |
| `totalResults` | `number` | Total result count |
| `error` | `string \| null` | Error message |
| `pagination` | `PaginationInfo \| null` | Pagination info |
| `query` | `SearchQuery \| null` | Current query |

---

## Utilities

### highlightText

Highlight search terms in text.

```tsx
import { highlightText } from '@/utils/searchHighlight';

const highlighted = highlightText(
  'filename.txt',
  'file',
  'exact_name',
  { className: 'custom-highlight' }
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | `string` | Text to highlight |
| `searchTerm` | `string` | Term to highlight |
| `matchType` | `MatchType` | Type of match |
| `options` | `HighlightOptions` | Highlight options |

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `className` | `string` | `'search-match-highlight'` | CSS class |
| `caseSensitive` | `boolean` | `false` | Case sensitive |

---

### highlightContentMatches

Highlight content matches with precise positioning.

```tsx
import { highlightContentMatches } from '@/utils/searchHighlight';

const highlighted = highlightContentMatches(
  'This is sample content',
  matches
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | `string` | Text content |
| `matches` | `ContentMatch[]` | Match positions |

---

### highlightLineContent  

Render highlighted line content with line numbers.

```tsx
import { highlightLineContent } from '@/utils/searchHighlight';

const lineElement = highlightLineContent(match, {
  showLineNumbers: true,
  maxLength: 150
});
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `match` | `ContentMatch` | Content match |
| `options` | `LineHighlightOptions` | Display options |

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `showLineNumbers` | `boolean` | `true` | Show line numbers |
| `maxLength` | `number` | `200` | Max line length |

---

## Type Definitions

### SearchQuery

```typescript
interface SearchQuery {
  rootPath: string;
  namePattern?: string;
  contentPattern?: string;
  sizeFilter?: SizeFilter;
  dateFilter?: DateFilter;
  fileTypeFilter?: FileTypeFilter;
  options: SearchOptions;
}
```

### SearchResult

```typescript
interface SearchResult {
  path: string;
  name: string;
  size: number;
  modified: string;
  matchType: MatchType;
  relevanceScore: number;
  matches?: ContentMatch[];
}
```

### ContentMatch

```typescript
interface ContentMatch {
  lineNumber: number;
  lineContent: string;
  matchStart: number;
  matchEnd: number;
}
```

### MatchType

```typescript
type MatchType = 
  | 'exact_name'
  | 'fuzzy_name'
  | 'content'
  | 'extension'
  | 'directory';
```

### PaginationInfo

```typescript
interface PaginationInfo {
  page: number;        // Current page (0-indexed)
  pageSize: number;    // Results per page
  totalPages: number;  // Total page count
}
```

---

## CSS Classes

### Result Items

| Class | Description |
|-------|-------------|
| `.search-results` | Main container |
| `.result-item` | Individual result |
| `.result-item.selected` | Selected state |
| `.result-header` | Name and badges |
| `.result-details` | Path and metadata |
| `.content-matches` | Match previews |

### Visual Effects

| Class | Description |
|-------|-------------|
| `.card-enhanced` | Modern card style |
| `.hover-lift` | Hover elevation |
| `.search-match-highlight` | Match highlighting |
| `.skeleton-loading` | Loading skeleton |

### Animations

| Class | Description |
|-------|-------------|
| `.fadeIn` | Fade entrance |
| `.slideIn` | Slide entrance |
| `.bounceIn` | Bounce entrance |
| `.search-result-enter-staggered` | Staggered list animation |

---

*For implementation examples and usage patterns, see `/docs/development/search-quick-reference.md`*