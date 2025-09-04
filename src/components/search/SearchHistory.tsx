/**
 * Search History Component
 * 
 * Displays and manages search history with options to rerun or remove searches.
 */

import React, { useState, useCallback } from 'react';
import { useSearch } from '@/hooks/useSearch';
import { SearchHistoryEntry } from '@/types';

interface SearchHistoryProps {
  className?: string;
  onSearchSelect?: (entry: SearchHistoryEntry) => void;
}

export const SearchHistory: React.FC<SearchHistoryProps> = ({
  className = '',
  onSearchSelect
}) => {
  const { searchHistory, clearHistory, removeFromHistory, rerunFromHistory } = useSearch();
  const [isExpanded, setIsExpanded] = useState(false);

  const handleRerun = useCallback(async (entry: SearchHistoryEntry) => {
    try {
      await rerunFromHistory(entry.id);
      onSearchSelect?.(entry);
    } catch (error) {
      console.error('Failed to rerun search:', error);
    }
  }, [rerunFromHistory, onSearchSelect]);

  const handleRemove = useCallback((entry: SearchHistoryEntry, event: React.MouseEvent) => {
    event.stopPropagation(); // Prevent triggering rerun
    removeFromHistory(entry.id);
  }, [removeFromHistory]);

  const handleClearAll = useCallback(() => {
    if (confirm('Clear all search history?')) {
      clearHistory();
    }
  }, [clearHistory]);

  const formatQuery = useCallback((entry: SearchHistoryEntry): string => {
    const parts = [];
    if (entry.query.namePattern) parts.push(`Name: "${entry.query.namePattern}"`);
    if (entry.query.contentPattern) parts.push(`Content: "${entry.query.contentPattern}"`);
    if (entry.query.sizeFilter) parts.push(`Size filter`);
    if (entry.query.dateFilter) parts.push(`Date filter`);
    if (entry.query.fileTypeFilter) parts.push(`Type filter`);
    
    return parts.length > 0 ? parts.join(', ') : 'All files';
  }, []);

  const formatRelativeTime = useCallback((timestamp: Date): string => {
    const now = new Date();
    const diff = now.getTime() - timestamp.getTime();
    
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    
    return timestamp.toLocaleDateString();
  }, []);

  if (searchHistory.length === 0) {
    return (
      <div className={`search-history empty ${className}`}>
        <p className="no-history">No search history</p>
      </div>
    );
  }

  return (
    <div className={`search-history ${className}`}>
      <div className="history-header">
        <button 
          className="toggle-expand"
          onClick={() => setIsExpanded(!isExpanded)}
          aria-expanded={isExpanded}
        >
          <span className="expand-icon">{isExpanded ? '▼' : '▶'}</span>
          Search History ({searchHistory.length})
        </button>
        
        <button 
          className="clear-all"
          onClick={handleClearAll}
          disabled={searchHistory.length === 0}
        >
          Clear All
        </button>
      </div>

      {isExpanded && (
        <div className="history-list">
          {searchHistory.map((entry) => (
            <div
              key={entry.id}
              className="history-item"
              onClick={() => handleRerun(entry)}
              title={`Rerun search: ${formatQuery(entry)}`}
            >
              <div className="history-content">
                <div className="history-query">
                  {formatQuery(entry)}
                </div>
                
                <div className="history-meta">
                  <span className="history-path">{entry.query.rootPath}</span>
                  <span className="history-results">{entry.resultCount} results</span>
                  <span className="history-time">{formatRelativeTime(entry.timestamp)}</span>
                </div>
              </div>
              
              <div className="history-actions">
                <button
                  className="remove-btn"
                  onClick={(e) => handleRemove(entry, e)}
                  title="Remove from history"
                  aria-label={`Remove search from history`}
                >
                  ×
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};