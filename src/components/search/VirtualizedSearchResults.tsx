/**
 * Virtualized Search Results Component
 * 
 * High-performance search results display using react-window for virtualization.
 * Handles thousands of search results efficiently by only rendering visible items.
 */

import React, { useCallback, useMemo, useRef, useEffect } from 'react';
import { FixedSizeList as List } from 'react-window';
import { useActiveSearchResults } from '@/hooks/useSearch';
import { useKeyboardNavigation } from '@/hooks/useKeyboardNavigation';
import { SearchResult, MatchType } from '@/types';
import { highlightText, highlightContentMatches, highlightLineContent } from '@/utils/searchHighlight';
import './SearchAnimations.css';

interface VirtualizedSearchResultsProps {
  onResultClick?: (result: SearchResult) => void;
  onResultDoubleClick?: (result: SearchResult) => void;
  className?: string;
  itemHeight?: number;
  maxHeight?: number;
}

interface ResultItemProps {
  index: number;
  style: React.CSSProperties;
  data: {
    results: SearchResult[];
    onResultClick?: (result: SearchResult) => void;
    onResultDoubleClick?: (result: SearchResult) => void;
    formatFileSize: (bytes: number) => string;
    getMatchTypeInfo: (matchType: MatchType) => any;
    getScoreColor: (score: number) => string;
    highlightMatches: (text: string, result: SearchResult) => React.ReactNode;
    query?: any;
    selectedIndex: number;
  };
}

const ResultItem: React.FC<ResultItemProps> = ({ index, style, data }) => {
  const result = data.results[index];
  const matchInfo = data.getMatchTypeInfo(result.matchType);
  const scoreColor = data.getScoreColor(result.relevanceScore);
  const isSelected = data.selectedIndex === index;

  const handleClick = () => {
    data.onResultClick?.(result);
  };

  const handleDoubleClick = () => {
    data.onResultDoubleClick?.(result);
  };

  return (
    <div
      style={style}
      className="virtual-result-item"
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      role="button"
      tabIndex={isSelected ? 0 : -1}
      data-result-index={index}
    >
      <div className={`result-item card-enhanced hover-lift ${isSelected ? 'selected' : ''}`}>
        {/* Result Header */}
        <div className="result-header">
          <div className="result-name">
            {data.highlightMatches(result.name, result)}
          </div>
          <div className="result-badges">
            {/* Relevance Score */}
            <div
              className="relevance-score badge-animated hover-glow"
              style={{ backgroundColor: scoreColor }}
              title="Relevance Score"
            >
              {result.relevanceScore}
            </div>
            {/* Match Type */}
            <div
              className={`match-type ${matchInfo.className} badge-animated hover-scale`}
              style={{ backgroundColor: matchInfo.color }}
              title={`Match Type: ${matchInfo.label}`}
            >
              {matchInfo.label}
            </div>
          </div>
        </div>

        {/* Result Details */}
        <div className="result-details">
          <div className="result-path" title={result.path}>
            {result.path}
          </div>
          <div className="result-meta">
            <span className="file-size">
              {data.formatFileSize(result.size)}
            </span>
            {result.modified && (
              <span className="modified-date">
                Modified: {new Date(result.modified).toLocaleString()}
              </span>
            )}
          </div>
        </div>

        {/* Content Matches Preview */}
        {result.matches && result.matches.length > 0 && (
          <div className="content-matches">
            <div className="matches-header">
              <span className="matches-count">
                {result.matches.length} match{result.matches.length !== 1 ? 'es' : ''}
              </span>
            </div>
            <div className="matches-preview">
              {result.matches.slice(0, 2).map((match, matchIndex) => (
                <div key={matchIndex} className="match-snippet">
                  {highlightLineContent(match, { 
                    showLineNumbers: true,
                    maxLength: 120 
                  })}
                </div>
              ))}
              {result.matches.length > 2 && (
                <div className="more-matches">
                  +{result.matches.length - 2} more match{result.matches.length - 2 !== 1 ? 'es' : ''}
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export const VirtualizedSearchResults: React.FC<VirtualizedSearchResultsProps> = ({
  onResultClick,
  onResultDoubleClick,
  className = '',
  itemHeight = 120,
  maxHeight = 600
}) => {
  const { results, isSearching, searchId, totalResults, error, query } = useActiveSearchResults();
  const listRef = useRef<List>(null);
  
  // Keyboard navigation integration
  const keyboardNavigation = useKeyboardNavigation(results, {
    onResultSelect: onResultClick,
    onResultActivate: onResultDoubleClick,
    enableQuickActions: true,
    enableTypeahead: true,
    wrapNavigation: true
  });

  // Sort results by relevance score (descending)
  const sortedResults = useMemo(() => {
    return [...results].sort((a, b) => b.relevanceScore - a.relevanceScore);
  }, [results]);

  // Format file size
  const formatFileSize = useCallback((bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  }, []);

  // Get match type display info
  const getMatchTypeInfo = useCallback((matchType: MatchType) => {
    switch (matchType) {
      case 'exact_name':
        return { label: 'Exact Name', className: 'match-exact', color: '#10b981' };
      case 'fuzzy_name':
        return { label: 'Fuzzy Name', className: 'match-fuzzy', color: '#3b82f6' };
      case 'content':
        return { label: 'Content', className: 'match-content', color: '#8b5cf6' };
      case 'extension':
        return { label: 'Extension', className: 'match-extension', color: '#f59e0b' };
      case 'directory':
        return { label: 'Directory', className: 'match-directory', color: '#6b7280' };
      default:
        return { label: 'Unknown', className: 'match-unknown', color: '#9ca3af' };
    }
  }, []);

  // Get relevance score color
  const getScoreColor = useCallback((score: number): string => {
    if (score >= 90) return '#10b981'; // green
    if (score >= 70) return '#3b82f6'; // blue
    if (score >= 50) return '#f59e0b'; // yellow
    if (score >= 30) return '#ef4444'; // red
    return '#6b7280'; // gray
  }, []);

  // Highlight matched text based on search query and match type
  const highlightMatches = useCallback((text: string, result: SearchResult): React.ReactNode => {
    if (!query || !text) {
      return text;
    }

    // Determine search term based on match type and query
    let searchTerm = '';
    
    if (result.matchType === 'content' && result.matches && result.matches.length > 0) {
      // For content matches, use the precise match highlighting
      return highlightContentMatches(text, result.matches);
    } else if (query.namePattern) {
      searchTerm = query.namePattern;
    } else if (query.contentPattern) {
      searchTerm = query.contentPattern;
    }

    if (!searchTerm) {
      return text;
    }

    // Get appropriate CSS class based on match type
    const getHighlightClass = (matchType: MatchType) => {
      switch (matchType) {
        case 'fuzzy_name':
          return 'search-match-highlight fuzzy';
        case 'extension':
          return 'search-match-highlight extension';
        case 'directory':
          return 'search-match-highlight directory';
        default:
          return 'search-match-highlight';
      }
    };

    return highlightText(
      text, 
      searchTerm, 
      result.matchType, 
      { 
        className: getHighlightClass(result.matchType),
        caseSensitive: query.options.caseSensitive 
      }
    );
  }, [query]);

  // Auto-scroll to top when results change
  useEffect(() => {
    if (listRef.current && results.length > 0) {
      listRef.current.scrollToItem(0, 'start');
    }
  }, [searchId]);

  // Create data object for list items
  const itemData = useMemo(() => ({
    results: sortedResults,
    onResultClick,
    onResultDoubleClick,
    formatFileSize,
    getMatchTypeInfo,
    getScoreColor,
    highlightMatches,
    query,
    selectedIndex: keyboardNavigation.selectedIndex
  }), [
    sortedResults,
    onResultClick,
    onResultDoubleClick,
    formatFileSize,
    getMatchTypeInfo,
    getScoreColor,
    highlightMatches,
    query,
    keyboardNavigation.selectedIndex
  ]);

  // Calculate dynamic height
  const listHeight = Math.min(maxHeight, sortedResults.length * itemHeight);

  // Render loading state
  if (isSearching && results.length === 0) {
    return (
      <div className={`virtualized-search-results ${className}`}>
        <div className="search-status">
          <div className="loading-indicator status-indicator searching">
            <div className="status-dot searching"></div>
            <div className="loading-spinner"></div>
            <span>Searching...</span>
          </div>
        </div>
        
        {/* Skeleton Loading States */}
        <div className="results-list">
          {Array.from({ length: 8 }).map((_, index) => (
            <div key={index} className="virtual-result-item skeleton-loading" style={{ height: itemHeight }}>
              <div className="result-item">
                <div className="result-header">
                  <div className="skeleton skeleton-line medium"></div>
                  <div className="result-badges">
                    <div className="skeleton" style={{ width: '40px', height: '24px', borderRadius: '12px' }}></div>
                    <div className="skeleton" style={{ width: '80px', height: '24px', borderRadius: '12px' }}></div>
                  </div>
                </div>
                <div className="result-details">
                  <div className="skeleton skeleton-line long"></div>
                  <div className="skeleton skeleton-line short"></div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  // Render error state
  if (error) {
    return (
      <div className={`virtualized-search-results ${className}`}>
        <div className="search-error fadeIn">
          <div className="status-indicator error">
            <div className="status-dot error"></div>
            <div className="error-icon">⚠️</div>
          </div>
          <div className="error-message">
            <h4>Search Error</h4>
            <p>{error}</p>
          </div>
        </div>
      </div>
    );
  }

  // Render no results
  if (!isSearching && results.length === 0) {
    return (
      <div className={`virtualized-search-results ${className}`}>
        <div className="no-results fadeIn">
          <div className="no-results-icon bounceIn">🔍</div>
          <h4>No results found</h4>
          <p>Try adjusting your search criteria</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`virtualized-search-results ${className}`}>
      {/* Results Header */}
      <div className="results-header search-stats-enter">
        <div className="results-count">
          <strong>{totalResults}</strong> result{totalResults !== 1 ? 's' : ''} found
          {isSearching && (
            <span className="status-indicator searching">
              <span className="status-dot searching"></span>
              <span className="searching-indicator">searching...</span>
            </span>
          )}
          {!isSearching && totalResults > 0 && (
            <span className="status-indicator completed">
              <span className="status-dot completed"></span>
              <span>complete</span>
            </span>
          )}
        </div>
        {searchId && (
          <div className="search-id">
            Search ID: <code>{searchId}</code>
          </div>
        )}
        <div className="virtualization-info search-controls-enter">
          <span className="performance-badge badge-glow">⚡ Virtualized</span>
          <span className="items-info">
            Showing {sortedResults.length} items (rendering only visible)
          </span>
        </div>
      </div>

      {/* Virtualized Results List */}
      <div className="virtualized-list-container" ref={keyboardNavigation.containerRef}>
        <List
          ref={listRef}
          height={listHeight}
          width="100%"
          itemCount={sortedResults.length}
          itemSize={itemHeight}
          itemData={itemData}
          className="virtualized-list"
          overscanCount={5} // Render 5 extra items above/below viewport
        >
          {ResultItem}
        </List>
      </div>

      {/* Loading More Indicator */}
      {isSearching && results.length > 0 && (
        <div className="loading-more fadeIn">
          <div className="status-indicator searching">
            <div className="status-dot searching"></div>
            <div className="loading-spinner small"></div>
            <span>Finding more results...</span>
          </div>
          <div className="progress-bar indeterminate">
            <div className="progress-bar-fill"></div>
          </div>
        </div>
      )}

      {/* Performance Stats */}
      {sortedResults.length > 100 && (
        <div className="performance-stats">
          <div className="stats-item">
            <span className="stats-label">Total Results:</span>
            <span className="stats-value">{totalResults.toLocaleString()}</span>
          </div>
          <div className="stats-item">
            <span className="stats-label">Rendered Items:</span>
            <span className="stats-value">
              ~{Math.ceil(listHeight / itemHeight)} visible
            </span>
          </div>
          <div className="stats-item">
            <span className="stats-label">Memory Saved:</span>
            <span className="stats-value">
              {Math.round(((sortedResults.length - Math.ceil(listHeight / itemHeight)) / sortedResults.length) * 100)}%
            </span>
          </div>
        </div>
      )}
    </div>
  );
};