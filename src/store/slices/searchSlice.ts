import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { 
  SearchResultsState, 
  SearchQuery, 
  SearchResult,
  SearchHistoryEntry,
  SavedSearch
} from "@/types";
import { searchHistoryStorage, savedSearchesStorage, initializeSearchStorage } from "@/utils/searchStorage";

// Load persisted data
const { history, savedSearches } = initializeSearchStorage();

// Initial state for search functionality
const initialState: SearchResultsState = {
  activeSearchId: undefined,
  searches: {},
  history,
  savedSearches
};

export const searchSlice = createSlice({
  name: 'search',
  initialState,
  reducers: {
    // Start a new search
    startSearch: (state, action: PayloadAction<{ id: string; query: SearchQuery }>) => {
      const { id, query } = action.payload;
      const pageSize = 50; // Default page size
      
      state.searches[id] = {
        id,
        query,
        status: 'running',
        results: [],
        totalResults: 0,
        startTime: new Date(),
        pagination: {
          page: 0,
          pageSize,
          totalPages: 0,
        },
      };
      
      state.activeSearchId = id;
    },

    // Add search results incrementally
    addSearchResults: (state, action: PayloadAction<{ searchId: string; results: SearchResult[] }>) => {
      const { searchId, results } = action.payload;
      const search = state.searches[searchId];
      
      if (search) {
        search.results.push(...results);
        search.totalResults = search.results.length;
        
        // Update pagination
        search.pagination.totalPages = Math.ceil(search.totalResults / search.pagination.pageSize);
      }
    },

    // Update search status
    updateSearchStatus: (state, action: PayloadAction<{ 
      searchId: string; 
      status: 'running' | 'completed' | 'cancelled' | 'error';
      error?: string;
    }>) => {
      const { searchId, status, error } = action.payload;
      const search = state.searches[searchId];
      
      if (search) {
        search.status = status;
        search.endTime = new Date();
        
        if (error) {
          search.error = error;
        }
        
        // Add to history when completed
        if (status === 'completed') {
          const historyEntry: SearchHistoryEntry = {
            id: searchId,
            query: search.query,
            timestamp: search.startTime,
            resultCount: search.totalResults
          };
          
          // Add to beginning of history, keep only last 50
          state.history.unshift(historyEntry);
          if (state.history.length > 50) {
            state.history = state.history.slice(0, 50);
          }
          
          // Persist to localStorage
          searchHistoryStorage.save(state.history);
        }
      }
    },

    // Cancel search
    cancelSearch: (state, action: PayloadAction<string>) => {
      const searchId = action.payload;
      const search = state.searches[searchId];
      
      if (search && search.status === 'running') {
        search.status = 'cancelled';
        search.endTime = new Date();
      }
      
      // Clear active search if it's the cancelled one
      if (state.activeSearchId === searchId) {
        state.activeSearchId = undefined;
      }
    },

    // Set active search
    setActiveSearch: (state, action: PayloadAction<string | undefined>) => {
      state.activeSearchId = action.payload;
    },

    // Clear search results
    clearSearch: (state, action: PayloadAction<string>) => {
      const searchId = action.payload;
      delete state.searches[searchId];
      
      if (state.activeSearchId === searchId) {
        state.activeSearchId = undefined;
      }
    },

    // Clear all searches
    clearAllSearches: (state) => {
      state.searches = {};
      state.activeSearchId = undefined;
    },

    // Update single search result (for real-time updates)
    updateSearchResult: (state, action: PayloadAction<{ searchId: string; result: SearchResult }>) => {
      const { searchId, result } = action.payload;
      const search = state.searches[searchId];
      
      if (search) {
        // Find existing result or add new one
        const existingIndex = search.results.findIndex(r => r.path === result.path);
        if (existingIndex >= 0) {
          search.results[existingIndex] = result;
        } else {
          search.results.push(result);
          search.totalResults = search.results.length;
        }
      }
    },

    // Remove search from history
    removeFromHistory: (state, action: PayloadAction<string>) => {
      const searchId = action.payload;
      state.history = state.history.filter(entry => entry.id !== searchId);
      
      // Persist to localStorage
      searchHistoryStorage.save(state.history);
    },

    // Clear search history
    clearHistory: (state) => {
      state.history = [];
      
      // Clear localStorage
      searchHistoryStorage.clear();
    },

    // Re-run search from history
    rerunSearchFromHistory: (state, action: PayloadAction<{ historyId: string; newSearchId: string }>) => {
      const { historyId, newSearchId } = action.payload;
      const historyEntry = state.history.find(entry => entry.id === historyId);
      
      if (historyEntry) {
        state.searches[newSearchId] = {
          id: newSearchId,
          query: historyEntry.query,
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
        
        state.activeSearchId = newSearchId;
      }
    },

    // Sort search results
    sortSearchResults: (state, action: PayloadAction<{ 
      searchId: string; 
      sortBy: 'relevance' | 'name' | 'path' | 'size' | 'modified' | 'match_type';
      order: 'asc' | 'desc';
    }>) => {
      const { searchId, sortBy, order } = action.payload;
      const search = state.searches[searchId];
      
      if (search && search.results.length > 0) {
        search.results.sort((a, b) => {
          let comparison = 0;
          
          switch (sortBy) {
            case 'relevance':
              comparison = b.relevanceScore - a.relevanceScore;
              break;
            case 'name':
              comparison = a.name.localeCompare(b.name);
              break;
            case 'path':
              comparison = a.path.localeCompare(b.path);
              break;
            case 'size':
              comparison = a.size - b.size;
              break;
            case 'modified':
              const aDate = a.modified ? new Date(a.modified).getTime() : 0;
              const bDate = b.modified ? new Date(b.modified).getTime() : 0;
              comparison = bDate - aDate;
              break;
            case 'match_type':
              comparison = a.matchType.localeCompare(b.matchType);
              break;
          }
          
          return order === 'desc' ? -comparison : comparison;
        });
      }
    },

    // Filter search results
    filterSearchResults: (state, action: PayloadAction<{ 
      searchId: string; 
      filter: {
        matchTypes?: string[];
        minRelevance?: number;
        extensions?: string[];
        dateRange?: { start: string; end: string };
      }
    }>) => {
      const { searchId, filter } = action.payload;
      const search = state.searches[searchId];
      
      if (search) {
        // Store original results if not already stored
        if (!(search as any).originalResults) {
          (search as any).originalResults = [...search.results];
        }
        
        // Apply filters to original results
        const originalResults = (search as any).originalResults;
        let filteredResults = [...originalResults];
        
        if (filter.matchTypes && filter.matchTypes.length > 0) {
          filteredResults = filteredResults.filter(result => 
            filter.matchTypes!.includes(result.matchType)
          );
        }
        
        if (filter.minRelevance !== undefined) {
          filteredResults = filteredResults.filter(result => 
            result.relevanceScore >= filter.minRelevance!
          );
        }
        
        if (filter.extensions && filter.extensions.length > 0) {
          filteredResults = filteredResults.filter(result => {
            const ext = result.name.split('.').pop()?.toLowerCase();
            return ext && filter.extensions!.includes(ext);
          });
        }
        
        if (filter.dateRange) {
          const startDate = new Date(filter.dateRange.start);
          const endDate = new Date(filter.dateRange.end);
          
          filteredResults = filteredResults.filter(result => {
            if (!result.modified) return false;
            const resultDate = new Date(result.modified);
            return resultDate >= startDate && resultDate <= endDate;
          });
        }
        
        search.results = filteredResults;
        search.totalResults = filteredResults.length;
      }
    },

    // Clear filters
    clearSearchFilters: (state, action: PayloadAction<string>) => {
      const searchId = action.payload;
      const search = state.searches[searchId];
      
      if (search && (search as any).originalResults) {
        search.results = [...(search as any).originalResults];
        search.totalResults = search.results.length;
        delete (search as any).originalResults;
      }
    },

    // Pagination actions
    setSearchPage: (state, action: PayloadAction<{ searchId: string; page: number }>) => {
      const { searchId, page } = action.payload;
      const search = state.searches[searchId];
      
      if (search && page >= 0 && page < search.pagination.totalPages) {
        search.pagination.page = page;
      }
    },

    setPageSize: (state, action: PayloadAction<{ searchId: string; pageSize: number }>) => {
      const { searchId, pageSize } = action.payload;
      const search = state.searches[searchId];
      
      if (search && pageSize > 0) {
        search.pagination.pageSize = pageSize;
        search.pagination.totalPages = Math.ceil(search.totalResults / pageSize);
        
        // Adjust current page if it's now out of bounds
        if (search.pagination.page >= search.pagination.totalPages) {
          search.pagination.page = Math.max(0, search.pagination.totalPages - 1);
        }
      }
    },

    // Saved search CRUD operations
    saveSavedSearch: (state, action: PayloadAction<SavedSearch>) => {
      const savedSearch = action.payload;
      const existingIndex = state.savedSearches.findIndex(s => s.id === savedSearch.id);
      
      if (existingIndex >= 0) {
        state.savedSearches[existingIndex] = savedSearch;
      } else {
        state.savedSearches.unshift(savedSearch);
      }
      
      // Persist to localStorage
      savedSearchesStorage.save(state.savedSearches);
    },

    updateSavedSearch: (state, action: PayloadAction<SavedSearch>) => {
      const updatedSearch = action.payload;
      const index = state.savedSearches.findIndex(s => s.id === updatedSearch.id);
      
      if (index >= 0) {
        state.savedSearches[index] = updatedSearch;
        savedSearchesStorage.save(state.savedSearches);
      }
    },

    deleteSavedSearch: (state, action: PayloadAction<string>) => {
      const searchId = action.payload;
      state.savedSearches = state.savedSearches.filter(s => s.id !== searchId);
      savedSearchesStorage.save(state.savedSearches);
    },

    useSavedSearch: (state, action: PayloadAction<string>) => {
      const searchId = action.payload;
      const savedSearch = state.savedSearches.find(s => s.id === searchId);
      
      if (savedSearch) {
        savedSearch.lastUsed = new Date();
        savedSearch.useCount += 1;
        savedSearchesStorage.save(state.savedSearches);
      }
    },

    clearSavedSearches: (state) => {
      state.savedSearches = [];
      savedSearchesStorage.clear();
    },
  }
});

export const {
  startSearch,
  addSearchResults,
  updateSearchStatus,
  cancelSearch,
  setActiveSearch,
  clearSearch,
  clearAllSearches,
  updateSearchResult,
  removeFromHistory,
  clearHistory,
  rerunSearchFromHistory,
  sortSearchResults,
  filterSearchResults,
  clearSearchFilters,
  setSearchPage,
  setPageSize,
  saveSavedSearch,
  updateSavedSearch,
  deleteSavedSearch,
  useSavedSearch,
  clearSavedSearches,
} = searchSlice.actions;

export default searchSlice.reducer;