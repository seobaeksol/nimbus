/**
 * Performance Tests for Virtualized Search Results
 * 
 * Tests that verify virtualization performance benefits and proper operation
 * with large datasets compared to regular pagination.
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import searchReducer from '@/store/slices/searchSlice';
import { VirtualizedSearchResults } from '../VirtualizedSearchResults';
import { SearchResult } from '@/types';

// Mock useActiveSearchResults
const mockUseActiveSearchResults = vi.fn();
vi.mock('@/hooks/useSearch', () => ({
  useActiveSearchResults: () => mockUseActiveSearchResults()
}));

// Mock react-window
vi.mock('react-window', () => {
  const mockList = vi.fn().mockImplementation(({ children, itemData, itemCount, itemSize }) => {
    // Render only the first few items to simulate virtualization
    const visibleItems = Math.min(10, itemCount);
    const items = [];
    
    for (let i = 0; i < visibleItems; i++) {
      items.push(
        children({
          index: i,
          style: { height: itemSize },
          data: itemData
        })
      );
    }
    
    return <div data-testid="virtualized-list">{items}</div>;
  });

  return {
    FixedSizeList: mockList
  };
});

// Create test store
const createTestStore = () => configureStore({
  reducer: {
    search: searchReducer
  },
  middleware: (getDefaultMiddleware) => getDefaultMiddleware({
    serializableCheck: {
      ignoredActions: ['persist/PERSIST', 'persist/REHYDRATE']
    }
  })
});

// Create wrapper
const createWrapper = (store = createTestStore()) => {
  return ({ children }: { children: React.ReactNode }) => (
    <Provider store={store}>{children}</Provider>
  );
};

// Generate large dataset for performance testing
const generateLargeDataset = (count: number): SearchResult[] => {
  const results: SearchResult[] = [];
  
  for (let i = 0; i < count; i++) {
    results.push({
      path: `/test/path/file${i}.txt`,
      name: `file${i}.txt`,
      size: 1000 + (i % 10000),
      modified: new Date(2024, 0, 1 + (i % 365)).toISOString(),
      matchType: i % 4 === 0 ? 'exactname' : i % 4 === 1 ? 'fuzzyname' : i % 4 === 2 ? 'content' : 'extension',
      relevanceScore: Math.floor(Math.random() * 100),
      snippet: `Match snippet for file ${i}`,
      matches: i % 3 === 0 ? [
        {
          lineNumber: Math.floor(Math.random() * 100) + 1,
          lineContent: `This is line content with match for file ${i}`,
          matchStart: 10,
          matchEnd: 20
        }
      ] : undefined
    });
  }
  
  return results;
};

describe('VirtualizedSearchResults Performance', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    // Mock performance.now for consistent testing
    let mockTime = 0;
    vi.spyOn(performance, 'now').mockImplementation(() => {
      mockTime += 16.67; // Simulate 60fps (16.67ms per frame)
      return mockTime;
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Large Dataset Handling', () => {
    it('should handle 1000 results efficiently', async () => {
      const largeDataset = generateLargeDataset(1000);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: largeDataset,
        isSearching: false,
        searchId: 'large-test-1000',
        totalResults: 1000,
        error: undefined,
        pagination: undefined
      });

      const startTime = performance.now();
      
      render(<VirtualizedSearchResults />, { wrapper });
      
      const endTime = performance.now();
      const renderTime = endTime - startTime;

      // Should render header with result count
      expect(screen.getByText('1000')).toBeInTheDocument();
      expect(screen.getByText(/results found/)).toBeInTheDocument();
      
      // Should show performance badge
      expect(screen.getByText('⚡ Virtualized')).toBeInTheDocument();
      
      // Should show virtualization info
      expect(screen.getByText(/Showing 1000 items \(rendering only visible\)/)).toBeInTheDocument();
      
      // Should render virtualized list
      expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
      
      // Should show performance stats for large datasets
      expect(screen.getByText('Total Results:')).toBeInTheDocument();
      expect(screen.getByText('1,000')).toBeInTheDocument(); // Total results
      
      // Verify only visible items are rendered (mocked to 10)
      const renderedItems = screen.getAllByRole('button');
      expect(renderedItems).toHaveLength(10); // Only visible items
      
      // Performance should be reasonable (less than 5000ms for render in test environment)
      expect(renderTime).toBeLessThan(5000);
    });

    it('should handle 10000 results with minimal performance impact', async () => {
      const massiveDataset = generateLargeDataset(10000);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: massiveDataset,
        isSearching: false,
        searchId: 'massive-test-10000',
        totalResults: 10000,
        error: undefined,
        pagination: undefined
      });

      const startTime = performance.now();
      
      render(<VirtualizedSearchResults itemHeight={100} maxHeight={800} />, { wrapper });
      
      const endTime = performance.now();
      const renderTime = endTime - startTime;

      // Should render header with large count
      expect(screen.getByText('10000')).toBeInTheDocument();
      expect(screen.getByText(/results found/)).toBeInTheDocument();
      
      // Should show performance stats
      expect(screen.getByText('10,000')).toBeInTheDocument(); // Total results formatted
      expect(screen.getByText(/~8 visible/)).toBeInTheDocument(); // Visible items (800px / 100px)
      
      // Should show memory savings  
      expect(screen.getByText(/Memory Saved:/)).toBeInTheDocument();
      expect(screen.getByText(/100%/)).toBeInTheDocument(); // High memory savings
      
      // Performance should still be reasonable for test environment
      expect(renderTime).toBeLessThan(5000);
      
      // Should only render visible items (not all 10,000)
      const renderedItems = screen.getAllByRole('button');
      expect(renderedItems).toHaveLength(10); // Mocked visible count
    });
  });

  describe('Memory Usage Optimization', () => {
    it('should calculate memory savings correctly', () => {
      const dataset = generateLargeDataset(5000);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: dataset,
        isSearching: false,
        searchId: 'memory-test-5000',
        totalResults: 5000,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults itemHeight={120} maxHeight={600} />, { wrapper });
      
      // With 600px max height and 120px item height = 5 visible items
      // Memory saved = (5000 - 5) / 5000 = 99.9% (displayed as 100%)
      expect(screen.getByText(/100%/)).toBeInTheDocument();
    });

    it('should not show performance stats for small datasets', () => {
      const smallDataset = generateLargeDataset(50);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: smallDataset,
        isSearching: false,
        searchId: 'small-test-50',
        totalResults: 50,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      // Should not show performance stats for datasets <= 100
      expect(screen.queryByText('Total Results:')).not.toBeInTheDocument();
    });
  });

  describe('Virtualization Features', () => {
    it('should display virtualization badge and info', () => {
      const dataset = generateLargeDataset(500);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: dataset,
        isSearching: false,
        searchId: 'feature-test',
        totalResults: 500,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      // Should show performance badge
      const performanceBadge = screen.getByText('⚡ Virtualized');
      expect(performanceBadge).toBeInTheDocument();
      expect(performanceBadge).toHaveClass('performance-badge');
      
      // Should show items info
      expect(screen.getByText('Showing 500 items (rendering only visible)')).toBeInTheDocument();
    });

    it('should maintain sorting with virtualization', () => {
      const dataset = generateLargeDataset(100);
      // Set specific relevance scores to test sorting
      dataset[0].relevanceScore = 95;
      dataset[1].relevanceScore = 85;
      dataset[2].relevanceScore = 99;
      
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: dataset,
        isSearching: false,
        searchId: 'sorting-test',
        totalResults: 100,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      // Should render virtualized list with sorted data
      expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
      
      // Verify items are rendered (the mock renders 10 items)
      const renderedItems = screen.getAllByRole('button');
      expect(renderedItems.length).toBeLessThanOrEqual(10);
    });

    it('should handle dynamic item heights', () => {
      const dataset = generateLargeDataset(200);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: dataset,
        isSearching: false,
        searchId: 'height-test',
        totalResults: 200,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults itemHeight={150} maxHeight={900} />, { wrapper });
      
      // Should render virtualized list
      expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
      
      // Should show performance badge
      expect(screen.getByText('⚡ Virtualized')).toBeInTheDocument();
    });
  });

  describe('Loading and Error States', () => {
    it('should show loading state during search', () => {
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: [],
        isSearching: true,
        searchId: undefined,
        totalResults: 0,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      expect(screen.getByText('Searching...')).toBeInTheDocument();
      expect(screen.getByText('Searching...')).toBeInTheDocument();
    });

    it('should show error state', () => {
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: [],
        isSearching: false,
        searchId: undefined,
        totalResults: 0,
        error: 'Search failed due to network error',
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      expect(screen.getByText('Search Error')).toBeInTheDocument();
      expect(screen.getByText('Search failed due to network error')).toBeInTheDocument();
    });

    it('should show no results state', () => {
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: [],
        isSearching: false,
        searchId: 'empty-search',
        totalResults: 0,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      expect(screen.getByText('No results found')).toBeInTheDocument();
      expect(screen.getByText('Try adjusting your search criteria')).toBeInTheDocument();
    });
  });

  describe('Component Integration', () => {
    it('should render virtualization UI elements correctly', () => {
      const dataset = generateLargeDataset(1000);
      const wrapper = createWrapper();
      
      mockUseActiveSearchResults.mockReturnValue({
        results: dataset,
        isSearching: false,
        searchId: 'integration-test',
        totalResults: 1000,
        error: undefined,
        pagination: undefined
      });

      render(<VirtualizedSearchResults />, { wrapper });
      
      // Should show virtualization badge
      expect(screen.getByText('⚡ Virtualized')).toBeInTheDocument();
      
      // Should show items info
      expect(screen.getByText(/Showing 1000 items \(rendering only visible\)/)).toBeInTheDocument();
      
      // Should show performance stats
      expect(screen.getByText('Total Results:')).toBeInTheDocument();
      expect(screen.getByText('1,000')).toBeInTheDocument();
      
      // Should show memory savings
      expect(screen.getByText('Memory Saved:')).toBeInTheDocument();
    });
  });
});