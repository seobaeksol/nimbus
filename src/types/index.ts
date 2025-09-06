import { FileInfo } from "@/services/commands/ipc/file";


// Panel and layout types from architecture
export interface PanelState {
  id: string;
  tabs: TabState[];
  activeTabIndex: number;
  selection: Set<string>;
  viewMode: "list" | "grid" | "details";
  sortBy: string;
  sortOrder: "asc" | "desc";
  gridPosition?: GridPosition;
  isActive: boolean;
}

export interface TabState {
  id: string;
  title: string;
  path: string;
  files: FileInfo[];
  loading: boolean;
  error?: string;
  selection: Set<string>;
  sortBy: string;
  sortOrder: "asc" | "desc";
  viewMode: "list" | "grid" | "details";
  filter?: string;
  history: string[];
  historyIndex: number;
  scrollPosition: number;
  lastAccessed: Date;
  isPinned: boolean;
}

export interface PanelLayoutConfig {
  type: "single" | "dual" | "triple" | "grid2x2" | "grid2x3" | "grid3x2";
  splitterPositions: number[];
  gridDimensions?: {
    rows: number;
    cols: number;
    cellSpacing: number;
    uniformSizing: boolean;
  };
}

export interface GridPosition {
  row: number;
  col: number;
  rowSpan?: number;
  colSpan?: number;
}

export interface AppState {
  panels: PanelState[];
  activePanelId: string;
  layout: PanelLayoutConfig;
  globalSettings: any; // TODO: define settings structure
  connections: any[]; // TODO: define connection structure
  searchResults?: SearchResultsState;
}

// Advanced Search System Types

export interface SearchOptions {
  caseSensitive: boolean;
  useRegex: boolean;
  useFuzzy: boolean;          // Enable fuzzy matching for name patterns
  fuzzyThreshold: number;     // Minimum fuzzy match score (0-100)
  includeHidden: boolean;
  followSymlinks: boolean;
  maxResults?: number;
  maxDepth?: number;
  sortByRelevance: boolean;   // Sort results by relevance score
}

export interface SearchQuery {
  rootPath: string;
  namePattern?: string;
  contentPattern?: string;
  sizeFilter?: SizeFilter;
  dateFilter?: DateFilter;
  fileTypeFilter?: FileTypeFilter;
  options: SearchOptions;
}

export interface SizeFilter {
  minSize?: number;
  maxSize?: number;
  unit: 'bytes' | 'kb' | 'mb' | 'gb';
}

export interface DateFilter {
  dateType: 'modified' | 'created' | 'accessed';
  startDate?: string;  // ISO 8601 string
  endDate?: string;    // ISO 8601 string
}

export interface FileTypeFilter {
  extensions: string[];
  categories: FileCategory[];
}

export type FileCategory = 'documents' | 'images' | 'audio' | 'video' | 'archives' | 'code';

export type MatchType = 'exact_name' | 'fuzzy_name' | 'content' | 'extension' | 'directory';

export interface ContentMatch {
  lineNumber: number;
  lineContent: string;
  matchStart: number;
  matchEnd: number;
}

export interface SearchResult {
  searchId: string;
  path: string;
  name: string;
  size: number;
  modified?: string;        // ISO 8601 string
  created?: string;         // ISO 8601 string
  isDirectory: boolean;
  matches: ContentMatch[];
  relevanceScore: number;   // Higher score = more relevant
  matchType: MatchType;
}

export interface SearchResultsState {
  activeSearchId?: string;
  searches: Record<string, SearchState>;
  history: SearchHistoryEntry[];
  savedSearches: SavedSearch[];
}

export interface SearchState {
  id: string;
  query: SearchQuery;
  status: 'running' | 'completed' | 'cancelled' | 'error';
  results: SearchResult[];
  totalResults: number;
  error?: string;
  startTime: Date;
  endTime?: Date;
  pagination: {
    page: number;        // Current page (0-based)
    pageSize: number;    // Results per page
    totalPages: number;  // Total number of pages
  };
}

export interface SearchHistoryEntry {
  id: string;
  query: SearchQuery;
  timestamp: Date;
  resultCount: number;
}

// UI Component Types

export interface SearchPanelProps {
  onSearch: (query: SearchQuery) => void;
  onCancel: (searchId: string) => void;
  isSearching: boolean;
  defaultOptions?: Partial<SearchOptions>;
}

export interface SearchResultsProps {
  searchState: SearchState;
  onResultClick: (result: SearchResult) => void;
  onResultDoubleClick: (result: SearchResult) => void;
  highlightQuery?: string;
}

export interface FuzzySearchConfig {
  enabled: boolean;
  threshold: number;
  caseSensitive: boolean;
  includeScore: boolean;
}

export interface SavedSearch {
  id: string;
  name: string;
  description?: string;
  query: SearchQuery;
  createdAt: Date;
  lastUsed?: Date;
  useCount: number;
  tags?: string[];
}

// Re-export plugin types for convenience
export * from './plugins';