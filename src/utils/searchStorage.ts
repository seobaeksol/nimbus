/**
 * Search Storage Utilities
 * 
 * Handles localStorage persistence for search history and saved searches.
 */

import { SearchHistoryEntry, SavedSearch } from '@/types';

const SEARCH_HISTORY_KEY = 'nimbus_search_history';
const SAVED_SEARCHES_KEY = 'nimbus_saved_searches';

// Search History Storage
export const searchHistoryStorage = {
  load: (): SearchHistoryEntry[] => {
    try {
      const stored = localStorage.getItem(SEARCH_HISTORY_KEY);
      if (!stored) return [];
      
      const parsed = JSON.parse(stored);
      
      // Convert timestamp strings back to Date objects
      return parsed.map((entry: any) => ({
        ...entry,
        timestamp: new Date(entry.timestamp)
      }));
    } catch (error) {
      console.warn('Failed to load search history:', error);
      return [];
    }
  },

  save: (history: SearchHistoryEntry[]): void => {
    try {
      // Limit to 50 entries and convert dates to strings
      const limitedHistory = history.slice(0, 50).map(entry => ({
        ...entry,
        timestamp: entry.timestamp.toISOString()
      }));
      
      localStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(limitedHistory));
    } catch (error) {
      console.warn('Failed to save search history:', error);
    }
  },

  clear: (): void => {
    try {
      localStorage.removeItem(SEARCH_HISTORY_KEY);
    } catch (error) {
      console.warn('Failed to clear search history:', error);
    }
  }
};

// Saved Searches Storage  
export const savedSearchesStorage = {
  load: (): SavedSearch[] => {
    try {
      const stored = localStorage.getItem(SAVED_SEARCHES_KEY);
      if (!stored) return [];
      
      const parsed = JSON.parse(stored);
      
      // Convert timestamp strings back to Date objects
      return parsed.map((search: any) => ({
        ...search,
        createdAt: new Date(search.createdAt),
        lastUsed: search.lastUsed ? new Date(search.lastUsed) : undefined
      }));
    } catch (error) {
      console.warn('Failed to load saved searches:', error);
      return [];
    }
  },

  save: (savedSearches: SavedSearch[]): void => {
    try {
      // Convert dates to strings for storage
      const serialized = savedSearches.map(search => ({
        ...search,
        createdAt: search.createdAt.toISOString(),
        lastUsed: search.lastUsed?.toISOString()
      }));
      
      localStorage.setItem(SAVED_SEARCHES_KEY, JSON.stringify(serialized));
    } catch (error) {
      console.warn('Failed to save searches:', error);
    }
  },

  clear: (): void => {
    try {
      localStorage.removeItem(SAVED_SEARCHES_KEY);
    } catch (error) {
      console.warn('Failed to clear saved searches:', error);
    }
  }
};

// Initialize storage on first load
export const initializeSearchStorage = (): {
  history: SearchHistoryEntry[];
  savedSearches: SavedSearch[];
} => {
  return {
    history: searchHistoryStorage.load(),
    savedSearches: savedSearchesStorage.load()
  };
};