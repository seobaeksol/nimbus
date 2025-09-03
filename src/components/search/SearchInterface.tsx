/**
 * Search Interface Component
 * 
 * Complete search interface combining search panel and results display.
 * Provides integrated search functionality with real-time results.
 */

import React, { useCallback, useState } from 'react';
import { SearchPanel } from './SearchPanel';
import { SearchResults } from './SearchResults';
import { SearchResult } from '@/types';

interface SearchInterfaceProps {
  initialPath?: string;
  onFileSelect?: (result: SearchResult) => void;
  onFileOpen?: (result: SearchResult) => void;
  className?: string;
}

export const SearchInterface: React.FC<SearchInterfaceProps> = ({
  initialPath = '/',
  onFileSelect,
  onFileOpen,
  className = ''
}) => {
  const [isExpanded, setIsExpanded] = useState(true);

  // Handle result selection
  const handleResultClick = useCallback((result: SearchResult) => {
    onFileSelect?.(result);
  }, [onFileSelect]);

  // Handle result double-click to open file
  const handleResultDoubleClick = useCallback((result: SearchResult) => {
    onFileOpen?.(result);
  }, [onFileOpen]);

  return (
    <div className={`search-interface ${className}`}>
      {/* Search Interface Header */}
      <div className="search-header">
        <h2>File Search</h2>
        <button 
          className="toggle-expand"
          onClick={() => setIsExpanded(!isExpanded)}
          aria-label={isExpanded ? "Collapse" : "Expand"}
        >
          {isExpanded ? '▼' : '▶'}
        </button>
      </div>

      {isExpanded && (
        <div className="search-content">
          {/* Search Panel */}
          <div className="search-panel-container">
            <SearchPanel
              initialPath={initialPath}
              className="search-panel"
            />
          </div>

          {/* Results Section */}
          <div className="search-results-container">
            <SearchResults
              onResultClick={handleResultClick}
              onResultDoubleClick={handleResultDoubleClick}
              className="search-results"
            />
          </div>
        </div>
      )}
    </div>
  );
};