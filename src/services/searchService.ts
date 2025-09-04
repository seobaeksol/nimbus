/**
 * Advanced Search Service
 * 
 * Provides high-level search functionality with fuzzy matching, relevance ranking,
 * and real-time result streaming.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  SearchQuery, 
  SearchResult, 
  SearchOptions,
  SearchState 
} from '@/types';

export interface SearchServiceOptions {
  caseSensitive: boolean;
  useRegex: boolean;
  useFuzzy: boolean;
  fuzzyThreshold: number;
  includeHidden: boolean;
  followSymlinks: boolean;
  maxResults?: number;
  maxDepth?: number;
  sortByRelevance: boolean;
}

export class SearchService {
  private activeSearches = new Map<string, SearchState>();
  private eventListeners = new Map<string, () => void>();

  /**
   * Start a new search operation
   */
  async startSearch(query: SearchQuery): Promise<string> {
    try {
      // Convert frontend types to backend format
      const backendQuery = this.convertQueryToBackend(query);
      
      // Start the search
      const searchId = await invoke<string>('start_search', { 
        query: backendQuery 
      });

      // Initialize search state
      const searchState: SearchState = {
        id: searchId,
        query,
        status: 'running',
        results: [],
        totalResults: 0,
        startTime: new Date(),
        pagination: {
          page: 0,
          pageSize: 50,
          totalPages: 0,
        },
      };
      
      this.activeSearches.set(searchId, searchState);
      
      // Set up event listeners for this search
      await this.setupSearchListeners(searchId);
      
      return searchId;
    } catch (error) {
      console.error('Failed to start search:', error);
      throw new Error(`Search failed: ${error}`);
    }
  }

  /**
   * Cancel an active search
   */
  async cancelSearch(searchId: string): Promise<void> {
    try {
      await invoke('cancel_search', { searchId });
      
      // Update local state
      const searchState = this.activeSearches.get(searchId);
      if (searchState) {
        searchState.status = 'cancelled';
        searchState.endTime = new Date();
      }
      
      // Clean up listeners
      this.cleanupSearchListeners(searchId);
    } catch (error) {
      console.error('Failed to cancel search:', error);
      throw new Error(`Cancel search failed: ${error}`);
    }
  }

  /**
   * Get the current state of a search
   */
  getSearchState(searchId: string): SearchState | undefined {
    return this.activeSearches.get(searchId);
  }

  /**
   * Get all active searches
   */
  getActiveSearches(): SearchState[] {
    return Array.from(this.activeSearches.values())
      .filter(search => search.status === 'running');
  }

  /**
   * Set up event listeners for search results
   */
  private async setupSearchListeners(searchId: string): Promise<void> {
    // Listen for search results
    const resultListener = await listen<SearchResult>(
      `search-result-${searchId}`, 
      (event) => {
        this.handleSearchResult(searchId, event.payload);
      }
    );

    // Listen for search completion
    const completeListener = await listen(
      `search-complete-${searchId}`, 
      () => {
        this.handleSearchComplete(searchId);
      }
    );

    // Listen for search errors
    const errorListener = await listen<string>(
      `search-error-${searchId}`, 
      (event) => {
        this.handleSearchError(searchId, event.payload);
      }
    );

    // Store cleanup function
    const cleanup = () => {
      resultListener();
      completeListener();
      errorListener();
    };
    
    this.eventListeners.set(searchId, cleanup);
  }

  /**
   * Clean up event listeners for a search
   */
  private cleanupSearchListeners(searchId: string): void {
    const cleanup = this.eventListeners.get(searchId);
    if (cleanup) {
      cleanup();
      this.eventListeners.delete(searchId);
    }
  }

  /**
   * Handle incoming search result
   */
  private handleSearchResult(searchId: string, result: SearchResult): void {
    const searchState = this.activeSearches.get(searchId);
    if (!searchState) return;

    // Add result to state
    searchState.results.push(result);
    searchState.totalResults = searchState.results.length;

    // Emit event for UI updates
    this.emitSearchUpdate(searchId, searchState);
  }

  /**
   * Handle search completion
   */
  private handleSearchComplete(searchId: string): void {
    const searchState = this.activeSearches.get(searchId);
    if (!searchState) return;

    searchState.status = 'completed';
    searchState.endTime = new Date();

    // Clean up listeners
    this.cleanupSearchListeners(searchId);

    // Emit final update
    this.emitSearchUpdate(searchId, searchState);
  }

  /**
   * Handle search error
   */
  private handleSearchError(searchId: string, error: string): void {
    const searchState = this.activeSearches.get(searchId);
    if (!searchState) return;

    searchState.status = 'error';
    searchState.error = error;
    searchState.endTime = new Date();

    // Clean up listeners
    this.cleanupSearchListeners(searchId);

    // Emit error update
    this.emitSearchUpdate(searchId, searchState);
  }

  /**
   * Emit search state update
   */
  private emitSearchUpdate(searchId: string, searchState: SearchState): void {
    // This would typically dispatch to a global state manager (Redux, Zustand, etc.)
    // For now, we'll use a custom event
    const event = new CustomEvent('search-update', {
      detail: { searchId, searchState }
    });
    window.dispatchEvent(event);
  }

  /**
   * Convert frontend query format to backend format
   */
  private convertQueryToBackend(query: SearchQuery) {
    return {
      root_path: query.rootPath,
      name_pattern: query.namePattern || null,
      content_pattern: query.contentPattern || null,
      size_filter: query.sizeFilter ? {
        min_size: query.sizeFilter.minSize || null,
        max_size: query.sizeFilter.maxSize || null,
        unit: query.sizeFilter.unit
      } : null,
      date_filter: query.dateFilter ? {
        date_type: query.dateFilter.dateType,
        start_date: query.dateFilter.startDate || null,
        end_date: query.dateFilter.endDate || null
      } : null,
      file_type_filter: query.fileTypeFilter ? {
        extensions: query.fileTypeFilter.extensions,
        categories: query.fileTypeFilter.categories
      } : null,
      options: {
        case_sensitive: query.options.caseSensitive,
        use_regex: query.options.useRegex,
        use_fuzzy: query.options.useFuzzy,
        fuzzy_threshold: query.options.fuzzyThreshold,
        include_hidden: query.options.includeHidden,
        follow_symlinks: query.options.followSymlinks,
        max_results: query.options.maxResults || null,
        max_depth: query.options.maxDepth || null,
        sort_by_relevance: query.options.sortByRelevance
      }
    };
  }

  /**
   * Create default search options
   */
  static createDefaultOptions(): SearchOptions {
    return {
      caseSensitive: false,
      useRegex: false,
      useFuzzy: true,          // Enable fuzzy search by default
      fuzzyThreshold: 60,      // Reasonable threshold
      includeHidden: false,
      followSymlinks: false,
      sortByRelevance: true,   // Sort by relevance by default
    };
  }

  /**
   * Clean up all active searches
   */
  dispose(): void {
    // Cancel all active searches
    const activeSearchIds = Array.from(this.activeSearches.keys());
    activeSearchIds.forEach(searchId => {
      if (this.activeSearches.get(searchId)?.status === 'running') {
        this.cancelSearch(searchId).catch(console.error);
      }
    });

    // Clear all state
    this.activeSearches.clear();
    this.eventListeners.clear();
  }
}

// Global search service instance
export const searchService = new SearchService();