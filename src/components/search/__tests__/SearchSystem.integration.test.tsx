/**
 * Search System Integration Tests
 * 
 * Tests the complete search system flow including:
 * - Redux state management
 * - Search history persistence
 * - Pagination functionality
 * - Saved searches CRUD
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import searchReducer from '@/store/slices/searchSlice';
import { useSearch } from '@/hooks/useSearch';
import { SearchQuery, SearchOptions } from '@/types';
import { searchService } from '@/services/searchService';

// Mock Tauri IPC
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn()
}));

// Mock localStorage
const mockLocalStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn()
};
Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage
});

// Mock search service
vi.mock('@/services/searchService', () => ({
  searchService: {
    startSearch: vi.fn(),
    cancelSearch: vi.fn(),
    clearSearch: vi.fn()
  }
}));

// Test store setup with optional preloaded state
const createTestStore = (preloadedState?: any) => configureStore({
  reducer: {
    search: searchReducer
  },
  preloadedState,
  middleware: (getDefaultMiddleware) => getDefaultMiddleware({
    serializableCheck: {
      ignoredActions: ['persist/PERSIST', 'persist/REHYDRATE']
    }
  })
});

// Test wrapper
const createWrapper = (store = createTestStore()) => {
  return ({ children }: { children: React.ReactNode }) => (
    <Provider store={store}>{children}</Provider>
  );
};

describe('Search System Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocalStorage.getItem.mockReturnValue(null);
  });

  describe('Basic Search Flow', () => {
    it('should handle complete search lifecycle', async () => {
      const store = createTestStore();
      const wrapper = createWrapper(store);
      
      // Mock successful search
      const mockSearchId = 'test-search-123';
      (searchService.startSearch as any).mockResolvedValue(mockSearchId);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      const searchQuery: SearchQuery = {
        rootPath: '/test/path',
        namePattern: '*.ts',
        options: {
          caseSensitive: false,
          useRegex: false,
          useFuzzy: false,
          fuzzyThreshold: 80,
          includeHidden: false,
          followSymlinks: true,
          sortByRelevance: true
        }
      };

      // Start search
      await act(async () => {
        await result.current.startSearch(searchQuery);
      });

      // Verify search service was called
      expect(searchService.startSearch).toHaveBeenCalledWith(searchQuery);
      
      // Verify Redux state
      const state = store.getState();
      expect(state.search.searches).toHaveProperty(mockSearchId);
      expect(state.search.activeSearchId).toBe(mockSearchId);
    });

    it('should handle search cancellation', async () => {
      const store = createTestStore();
      const wrapper = createWrapper(store);
      
      const mockSearchId = 'test-search-456';
      (searchService.cancelSearch as any).mockResolvedValue(undefined);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      await act(async () => {
        await result.current.cancelSearch(mockSearchId);
      });

      expect(searchService.cancelSearch).toHaveBeenCalledWith(mockSearchId);
    });
  });

  describe('Search History', () => {
    it('should manage search history with localStorage persistence', async () => {
      // Create test history data
      const historyEntry = {
        id: 'history-1',
        query: { rootPath: '/test', namePattern: '*.js', options: {} as SearchOptions },
        timestamp: new Date(),
        resultCount: 10
      };
      
      // Create store with preloaded history state
      const store = createTestStore({
        search: {
          activeSearchId: undefined,
          searches: {},
          history: [historyEntry],
          savedSearches: []
        }
      });
      const wrapper = createWrapper(store);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      // Test history access - should be loaded from preloaded state
      const state = store.getState();
      expect(state.search.history).toHaveLength(1);
      expect(state.search.history[0].id).toBe('history-1');
      
      // Test history clearing
      act(() => {
        result.current.clearHistory();
      });
      
      expect(mockLocalStorage.removeItem).toHaveBeenCalledWith('nimbus_search_history');
      
      const clearedState = store.getState();
      expect(clearedState.search.history).toHaveLength(0);
    });

    it('should rerun searches from history', async () => {
      // Create test history data
      const historyEntry = {
        id: 'history-1',
        query: { rootPath: '/test', namePattern: '*.tsx', options: {} as SearchOptions },
        timestamp: new Date(),
        resultCount: 5
      };
      
      // Create store with preloaded history state
      const store = createTestStore({
        search: {
          activeSearchId: undefined,
          searches: {},
          history: [historyEntry],
          savedSearches: []
        }
      });
      const wrapper = createWrapper(store);
      
      const mockSearchId = 'history-rerun-123';
      (searchService.startSearch as any).mockResolvedValue(mockSearchId);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      // Verify history entry is loaded from preloaded state
      expect(store.getState().search.history).toHaveLength(1);
      expect(store.getState().search.history[0].id).toBe('history-1');
      
      await act(async () => {
        await result.current.rerunFromHistory('history-1');
      });
      
      expect(searchService.startSearch).toHaveBeenCalledWith(historyEntry.query);
    });
  });

  describe('Pagination', () => {
    it('should handle pagination state changes', () => {
      const store = createTestStore();
      const wrapper = createWrapper(store);
      
      // Add search with pagination
      const searchId = 'paginated-search';
      store.dispatch({
        type: 'search/startSearch',
        payload: {
          id: searchId,
          query: { rootPath: '/test', options: {} as SearchOptions }
        }
      });
      
      // Add results to make pagination valid (need enough results for page 2)
      const mockResults = Array.from({ length: 120 }, (_, i) => ({
        path: `/test/file${i}.txt`,
        name: `file${i}.txt`,
        size: 1000 + i,
        modified: new Date().toISOString(),
        matchType: 'name' as const,
        relevanceScore: 0.9,
        snippet: `Match ${i}`
      }));
      
      store.dispatch({
        type: 'search/addSearchResults',
        payload: { searchId, results: mockResults }
      });
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      // Verify initial pagination setup (default page size is 50, so 120 results = 3 total pages)
      let state = store.getState();
      expect(state.search.searches[searchId].pagination.totalPages).toBe(3);
      expect(state.search.searches[searchId].pagination.page).toBe(0);
      
      // Test page change - now page 2 is valid (0, 1, 2 are valid pages)
      act(() => {
        store.dispatch({
          type: 'search/setSearchPage',
          payload: { searchId, page: 2 }
        });
      });
      
      state = store.getState();
      expect(state.search.searches[searchId].pagination.page).toBe(2);
      
      // Test page size change
      act(() => {
        store.dispatch({
          type: 'search/setPageSize',
          payload: { searchId, pageSize: 100 }
        });
      });
      
      const updatedState = store.getState();
      expect(updatedState.search.searches[searchId].pagination.pageSize).toBe(100);
      // With 100 page size and 120 results, should have 2 total pages
      expect(updatedState.search.searches[searchId].pagination.totalPages).toBe(2);
    });
  });

  describe('Saved Searches', () => {
    it('should handle saved search CRUD operations', () => {
      const store = createTestStore();
      const wrapper = createWrapper(store);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      const savedSearch = {
        id: 'saved-1',
        name: 'TypeScript Files',
        description: 'Find all TypeScript files',
        query: { rootPath: '/src', namePattern: '*.ts', options: {} as SearchOptions },
        createdAt: new Date(),
        useCount: 0,
        tags: ['typescript', 'development']
      };
      
      // Test save
      act(() => {
        result.current.saveSavedSearch(savedSearch);
      });
      
      let state = store.getState();
      expect(state.search.savedSearches).toHaveLength(1);
      expect(state.search.savedSearches[0].name).toBe('TypeScript Files');
      expect(mockLocalStorage.setItem).toHaveBeenCalled();
      
      // Test update
      const updatedSearch = { ...savedSearch, name: 'Updated TypeScript Files' };
      act(() => {
        result.current.updateSavedSearch(updatedSearch);
      });
      
      state = store.getState();
      expect(state.search.savedSearches[0].name).toBe('Updated TypeScript Files');
      
      // Test use (increment count)
      act(() => {
        result.current.useSavedSearch(savedSearch.id);
      });
      
      state = store.getState();
      expect(state.search.savedSearches[0].useCount).toBe(1);
      expect(state.search.savedSearches[0].lastUsed).toBeDefined();
      
      // Test delete
      act(() => {
        result.current.deleteSavedSearch(savedSearch.id);
      });
      
      state = store.getState();
      expect(state.search.savedSearches).toHaveLength(0);
    });

    it('should handle saved search localStorage persistence', () => {
      const store = createTestStore();
      const wrapper = createWrapper(store);
      
      // Mock existing saved searches
      const mockSavedSearches = JSON.stringify([
        {
          id: 'saved-1',
          name: 'Test Search',
          query: { rootPath: '/test', options: {} },
          createdAt: new Date().toISOString(),
          useCount: 0
        }
      ]);
      mockLocalStorage.getItem.mockReturnValue(mockSavedSearches);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      // Test clear all
      act(() => {
        result.current.clearSavedSearches();
      });
      
      expect(mockLocalStorage.removeItem).toHaveBeenCalled();
      
      const state = store.getState();
      expect(state.search.savedSearches).toHaveLength(0);
    });
  });

  describe('Error Handling', () => {
    it('should handle search service errors gracefully', async () => {
      const store = createTestStore();
      const wrapper = createWrapper(store);
      
      // Mock search service error
      const mockError = new Error('Search failed');
      (searchService.startSearch as any).mockRejectedValue(mockError);
      
      const { result } = renderHook(() => useSearch(), { wrapper });
      
      const searchQuery: SearchQuery = {
        rootPath: '/invalid/path',
        namePattern: '*.ts',
        options: {
          caseSensitive: false,
          useRegex: false,
          useFuzzy: false,
          fuzzyThreshold: 80,
          includeHidden: false,
          followSymlinks: true,
          sortByRelevance: true
        }
      };

      // Expect search to throw error
      await expect(act(async () => {
        await result.current.startSearch(searchQuery);
      })).rejects.toThrow('Search failed');
    });
  });
});