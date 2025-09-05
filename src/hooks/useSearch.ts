/**
 * Advanced Search Hook
 * 
 * Integrates the SearchService with Redux state management for seamless
 * search functionality in React components.
 */

import { useCallback, useEffect, useRef } from 'react';
import { useAppDispatch, useAppSelector } from '@/store';
import { searchService } from '@/services/searchService';
import { 
  startSearch as startSearchAction,
  addSearchResults,
  updateSearchStatus,
  cancelSearch as cancelSearchAction,
  setActiveSearch,
  clearSearch,
  clearAllSearches,
  sortSearchResults,
  filterSearchResults,
  clearSearchFilters,
  clearHistory,
  removeFromHistory,
  rerunSearchFromHistory,
  saveSavedSearch,
  updateSavedSearch,
  deleteSavedSearch,
  useSavedSearch,
  clearSavedSearches,
} from '@/store/slices/searchSlice';
import { 
  SearchQuery, 
  SearchOptions, 
  SearchState, 
  SearchResult,
  SavedSearch
} from '@/types';

export interface UseSearchReturn {
  // Current search state
  activeSearch: SearchState | undefined;
  allSearches: Record<string, SearchState>;
  isSearching: boolean;
  
  // Search operations
  startSearch: (query: SearchQuery) => Promise<string>;
  cancelSearch: (searchId: string) => Promise<void>;
  clearSearch: (searchId: string) => void;
  clearAllSearches: () => void;
  
  // Search management
  setActiveSearch: (searchId: string | undefined) => void;
  getSearchById: (searchId: string) => SearchState | undefined;
  
  // Result manipulation
  sortResults: (searchId: string, sortBy: string, order: 'asc' | 'desc') => void;
  filterResults: (searchId: string, filters: any) => void;
  clearFilters: (searchId: string) => void;
  
  // Convenience methods
  quickFileSearch: (path: string, pattern: string, options?: Partial<SearchOptions>) => Promise<string>;
  quickContentSearch: (path: string, content: string, options?: Partial<SearchOptions>) => Promise<string>;
  
  // Search history
  searchHistory: Array<any>;
  clearHistory: () => void;
  removeFromHistory: (searchId: string) => void;
  rerunFromHistory: (historyId: string) => Promise<string>;

  // Saved searches
  savedSearches: SavedSearch[];
  saveSavedSearch: (savedSearch: SavedSearch) => void;
  updateSavedSearch: (savedSearch: SavedSearch) => void;
  deleteSavedSearch: (searchId: string) => void;
  useSavedSearch: (searchId: string) => void;
  clearSavedSearches: () => void;
  
  // Default options
  createDefaultOptions: () => SearchOptions;
}

export function useSearch(): UseSearchReturn {
  const dispatch = useAppDispatch();
  const searchState = useAppSelector(state => state.search);
  const listenerSetup = useRef(false);

  // Set up global event listener for search updates
  useEffect(() => {
    if (listenerSetup.current) return;
    
    const handleSearchUpdate = (event: CustomEvent) => {
      const { searchId, searchState: updatedState } = event.detail;
      
      // Update Redux state based on search service events
      if (updatedState.status === 'running') {
        // Add new results if any
        if (updatedState.results.length > (searchState.searches[searchId]?.results.length || 0)) {
          const newResults = updatedState.results.slice(searchState.searches[searchId]?.results.length || 0);
          dispatch(addSearchResults({ searchId, results: newResults }));
        }
      } else {
        // Update final status
        dispatch(updateSearchStatus({
          searchId,
          status: updatedState.status,
          error: updatedState.error
        }));
      }
    };

    window.addEventListener('search-update', handleSearchUpdate as EventListener);
    listenerSetup.current = true;

    return () => {
      window.removeEventListener('search-update', handleSearchUpdate as EventListener);
    };
  }, [dispatch, searchState.searches]);

  // Start a new search
  const startSearch = useCallback(async (query: SearchQuery): Promise<string> => {
    try {
      const searchId = await searchService.startSearch(query);
      
      // Update Redux state
      dispatch(startSearchAction({ id: searchId, query }));
      
      return searchId;
    } catch (error) {
      console.error('Failed to start search:', error);
      throw error;
    }
  }, [dispatch]);

  // Cancel an active search
  const cancelSearch = useCallback(async (searchId: string): Promise<void> => {
    try {
      await searchService.cancelSearch(searchId);
      dispatch(cancelSearchAction(searchId));
    } catch (error) {
      console.error('Failed to cancel search:', error);
      throw error;
    }
  }, [dispatch]);

  // Quick file name search
  const quickFileSearch = useCallback(async (
    path: string, 
    pattern: string, 
    options: Partial<SearchOptions> = {}
  ): Promise<string> => {
    const searchOptions: SearchOptions = {
      ...createDefaultOptions(),
      ...options,
    };

    const query: SearchQuery = {
      rootPath: path,
      namePattern: pattern,
      options: searchOptions,
    };

    return startSearch(query);
  }, [startSearch]);

  // Quick content search
  const quickContentSearch = useCallback(async (
    path: string, 
    content: string, 
    options: Partial<SearchOptions> = {}
  ): Promise<string> => {
    const searchOptions: SearchOptions = {
      ...createDefaultOptions(),
      ...options,
    };

    const query: SearchQuery = {
      rootPath: path,
      contentPattern: content,
      options: searchOptions,
    };

    return startSearch(query);
  }, [startSearch]);

  // Sort search results
  const sortResults = useCallback((
    searchId: string, 
    sortBy: string, 
    order: 'asc' | 'desc'
  ) => {
    dispatch(sortSearchResults({ 
      searchId, 
      sortBy: sortBy as any, 
      order 
    }));
  }, [dispatch]);

  // Filter search results
  const filterResults = useCallback((searchId: string, filters: any) => {
    dispatch(filterSearchResults({ searchId, filter: filters }));
  }, [dispatch]);

  // Clear filters
  const clearFilters = useCallback((searchId: string) => {
    dispatch(clearSearchFilters(searchId));
  }, [dispatch]);

  // Get search by ID
  const getSearchById = useCallback((searchId: string): SearchState | undefined => {
    return searchState.searches[searchId];
  }, [searchState.searches]);

  // Set active search
  const setActiveSearchHandler = useCallback((searchId: string | undefined) => {
    dispatch(setActiveSearch(searchId));
  }, [dispatch]);

  // Clear single search
  const clearSearchHandler = useCallback((searchId: string) => {
    dispatch(clearSearch(searchId));
  }, [dispatch]);

  // Clear all searches
  const clearAllSearchesHandler = useCallback(() => {
    dispatch(clearAllSearches());
  }, [dispatch]);

  // History management
  const clearHistoryHandler = useCallback(() => {
    dispatch(clearHistory());
  }, [dispatch]);

  const removeFromHistoryHandler = useCallback((searchId: string) => {
    dispatch(removeFromHistory(searchId));
  }, [dispatch]);

  const rerunFromHistoryHandler = useCallback(async (historyId: string): Promise<string> => {
    const historyEntry = searchState.history.find(entry => entry.id === historyId);
    if (!historyEntry) {
      throw new Error(`History entry ${historyId} not found`);
    }

    // Start a new search with the same query
    const newSearchId = await startSearch(historyEntry.query);
    return newSearchId;
  }, [searchState.history, startSearch]);

  // Saved search handlers
  const saveSavedSearchHandler = useCallback((savedSearch: SavedSearch) => {
    dispatch(saveSavedSearch(savedSearch));
  }, [dispatch]);

  const updateSavedSearchHandler = useCallback((savedSearch: SavedSearch) => {
    dispatch(updateSavedSearch(savedSearch));
  }, [dispatch]);

  const deleteSavedSearchHandler = useCallback((searchId: string) => {
    dispatch(deleteSavedSearch(searchId));
  }, [dispatch]);

  const useSavedSearchHandler = useCallback((searchId: string) => {
    dispatch(useSavedSearch(searchId));
  }, [dispatch]);

  const clearSavedSearchesHandler = useCallback(() => {
    dispatch(clearSavedSearches());
  }, [dispatch]);

  // Create default options
  const createDefaultOptions = useCallback((): SearchOptions => {
    return {
      useFuzzy: false,
      fuzzyThreshold: 80,
      sortByRelevance: true,
      caseSensitive: false,
      useRegex: false,
      includeHidden: false,
      followSymlinks: false,
      maxResults: 1000,
      maxDepth: undefined
    };
  }, []);

  // Determine if any search is currently running
  const isSearching = Object.values(searchState.searches).some(
    search => search.status === 'running'
  );

  // Get active search
  const activeSearch = searchState.activeSearchId 
    ? searchState.searches[searchState.activeSearchId]
    : undefined;

  return {
    // State
    activeSearch,
    allSearches: searchState.searches,
    isSearching,
    
    // Operations
    startSearch,
    cancelSearch,
    clearSearch: clearSearchHandler,
    clearAllSearches: clearAllSearchesHandler,
    
    // Management
    setActiveSearch: setActiveSearchHandler,
    getSearchById,
    
    // Result manipulation
    sortResults,
    filterResults,
    clearFilters,
    
    // Convenience methods
    quickFileSearch,
    quickContentSearch,
    
    // History
    searchHistory: searchState.history,
    clearHistory: clearHistoryHandler,
    removeFromHistory: removeFromHistoryHandler,
    rerunFromHistory: rerunFromHistoryHandler,

    // Saved searches
    savedSearches: searchState.savedSearches,
    saveSavedSearch: saveSavedSearchHandler,
    updateSavedSearch: updateSavedSearchHandler,
    deleteSavedSearch: deleteSavedSearchHandler,
    useSavedSearch: useSavedSearchHandler,
    clearSavedSearches: clearSavedSearchesHandler,
    
    // Utilities
    createDefaultOptions,
  };
}

// Type-safe search result selector hook
export function useSearchResults(searchId: string): SearchResult[] {
  return useAppSelector(state => 
    state.search.searches[searchId]?.results || []
  );
}

// Hook for search status
export function useSearchStatus(searchId: string) {
  return useAppSelector(state => ({
    status: state.search.searches[searchId]?.status || 'completed',
    error: state.search.searches[searchId]?.error,
    totalResults: state.search.searches[searchId]?.totalResults || 0,
    startTime: state.search.searches[searchId]?.startTime,
    endTime: state.search.searches[searchId]?.endTime,
  }));
}

// Hook for active search results with real-time updates
export function useActiveSearchResults(): {
  results: SearchResult[];
  isSearching: boolean;
  searchId?: string;
  totalResults: number;
  error?: string;
  pagination?: { page: number; pageSize: number; totalPages: number };
  query?: SearchQuery;
} {
  const activeSearchId = useAppSelector(state => state.search.activeSearchId);
  const activeSearch = useAppSelector(state => 
    activeSearchId ? state.search.searches[activeSearchId] : undefined
  );
  
  return {
    results: activeSearch?.results || [],
    isSearching: activeSearch?.status === 'running' || false,
    searchId: activeSearchId,
    totalResults: activeSearch?.totalResults || 0,
    error: activeSearch?.error,
    pagination: activeSearch?.pagination,
    query: activeSearch?.query,
  };
}