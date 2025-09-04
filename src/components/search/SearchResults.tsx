/**
 * Search Results Display Component
 * 
 * Displays search results with highlighting, relevance scores, and file previews.
 * Supports sorting, filtering, and interaction with search results.
 */

import React, { useCallback, useMemo } from 'react';
import { useActiveSearchResults } from '@/hooks/useSearch';
import { SearchResult } from '@/types';
import { SearchPagination } from './SearchPagination';

interface SearchResultsProps {
  onResultClick?: (result: SearchResult) => void;
  onResultDoubleClick?: (result: SearchResult) => void;
  className?: string;
}

export const SearchResults: React.FC<SearchResultsProps> = ({
  onResultClick,
  onResultDoubleClick,
  className = ''
}) => {
  const { results, isSearching, searchId, totalResults, error, pagination } = useActiveSearchResults();

  // Sort results by relevance score (descending)
  const sortedResults = useMemo(() => {
    return [...results].sort((a, b) => b.relevanceScore - a.relevanceScore);
  }, [results]);

  // Paginate results for display
  const paginatedResults = useMemo(() => {
    if (!pagination) return sortedResults;
    
    const startIndex = pagination.page * pagination.pageSize;
    const endIndex = startIndex + pagination.pageSize;
    return sortedResults.slice(startIndex, endIndex);
  }, [sortedResults, pagination]);

  // Handle result click
  const handleResultClick = useCallback((result: SearchResult) => {
    onResultClick?.(result);
  }, [onResultClick]);

  // Handle result double-click
  const handleResultDoubleClick = useCallback((result: SearchResult) => {
    onResultDoubleClick?.(result);
  }, [onResultDoubleClick]);

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
  const getMatchTypeInfo = useCallback((matchType: string) => {
    switch (matchType.toLowerCase()) {
      case 'exactname':
        return { label: 'Exact Name', className: 'match-exact', color: '#10b981' };
      case 'fuzzyname':
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

  // Highlight matched text
  const highlightMatches = useCallback((text: string, matches?: any[]): React.ReactNode => {
    if (!matches || matches.length === 0) {
      return text;
    }

    // For now, just return the text as-is
    // TODO: Implement proper highlighting based on match positions
    return text;
  }, []);

  // Render loading state
  if (isSearching && results.length === 0) {
    return (
      <div className={`search-results ${className}`}>
        <div className="search-status">
          <div className="loading-indicator">
            <div className="spinner"></div>
            <span>Searching...</span>
          </div>
        </div>
      </div>
    );
  }

  // Render error state
  if (error) {
    return (
      <div className={`search-results ${className}`}>
        <div className="search-error">
          <div className="error-icon">⚠️</div>
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
      <div className={`search-results ${className}`}>
        <div className="no-results">
          <div className="no-results-icon">🔍</div>
          <h4>No results found</h4>
          <p>Try adjusting your search criteria</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`search-results ${className}`}>
      {/* Results Header */}
      <div className="results-header">
        <div className="results-count">
          <strong>{totalResults}</strong> result{totalResults !== 1 ? 's' : ''} found
          {isSearching && <span className="searching-indicator"> (searching...)</span>}
        </div>
        {searchId && (
          <div className="search-id">
            Search ID: <code>{searchId}</code>
          </div>
        )}
      </div>

      {/* Pagination Controls */}
      {pagination && pagination.totalPages > 1 && (
        <SearchPagination
          searchId={searchId!}
          currentPage={pagination.page}
          pageSize={pagination.pageSize}
          totalPages={pagination.totalPages}
          totalResults={totalResults}
          className="search-pagination-top"
        />
      )}

      {/* Results List */}
      <div className="results-list">
        {paginatedResults.map((result, index) => {
          const matchInfo = getMatchTypeInfo(result.matchType);
          const scoreColor = getScoreColor(result.relevanceScore);

          return (
            <div
              key={`${result.path}-${index}`}
              className="result-item"
              onClick={() => handleResultClick(result)}
              onDoubleClick={() => handleResultDoubleClick(result)}
              role="button"
              tabIndex={0}
            >
              {/* Result Header */}
              <div className="result-header">
                <div className="result-name">
                  {highlightMatches(result.name, result.matches)}
                </div>
                <div className="result-badges">
                  {/* Relevance Score */}
                  <div
                    className="relevance-score"
                    style={{ backgroundColor: scoreColor }}
                    title="Relevance Score"
                  >
                    {result.relevanceScore}
                  </div>
                  {/* Match Type */}
                  <div
                    className={`match-type ${matchInfo.className}`}
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
                    {formatFileSize(result.size)}
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
                    {result.matches.slice(0, 3).map((match, matchIndex) => (
                      <div key={matchIndex} className="match-snippet">
                        <span className="line-number">Line {match.lineNumber}:</span>
                        <span className="match-text">
                          {highlightMatches(match.lineContent, [match])}
                        </span>
                      </div>
                    ))}
                    {result.matches.length > 3 && (
                      <div className="more-matches">
                        +{result.matches.length - 3} more match{result.matches.length - 3 !== 1 ? 'es' : ''}
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Loading More Indicator */}
      {isSearching && results.length > 0 && (
        <div className="loading-more">
          <div className="spinner"></div>
          <span>Finding more results...</span>
        </div>
      )}

      {/* Bottom Pagination Controls */}
      {pagination && pagination.totalPages > 1 && (
        <SearchPagination
          searchId={searchId!}
          currentPage={pagination.page}
          pageSize={pagination.pageSize}
          totalPages={pagination.totalPages}
          totalResults={totalResults}
          className="search-pagination-bottom"
        />
      )}
    </div>
  );
};