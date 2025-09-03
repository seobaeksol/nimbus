import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { 
  SearchResultsState, 
  SearchQuery, 
  SearchResult,
  SearchHistoryEntry 
} from "@/types";

// Initial state for search functionality
const initialState: SearchResultsState = {
  activeSearchId: undefined,
  searches: {},
  history: []
};

export const searchSlice = createSlice({
  name: 'search',
  initialState,
  reducers: {
    // Start a new search
    startSearch: (state, action: PayloadAction<{ id: string; query: SearchQuery }>) => {
      const { id, query } = action.payload;
      
      state.searches[id] = {
        id,
        query,
        status: 'running',
        results: [],
        totalResults: 0,
        startTime: new Date(),
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
    },

    // Clear search history
    clearHistory: (state) => {
      state.history = [];
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
} = searchSlice.actions;

export default searchSlice.reducer;