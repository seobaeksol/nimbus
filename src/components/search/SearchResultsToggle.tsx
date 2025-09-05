/**
 * Search Results View Toggle Component
 * 
 * Smart component that automatically switches between paginated and virtualized
 * search results based on result count and user preferences.
 */

import React, { useMemo, useCallback, useState } from 'react';
import { useActiveSearchResults } from '@/hooks/useSearch';
import { SearchResults } from './SearchResults';
import { VirtualizedSearchResults } from './VirtualizedSearchResults';
import { SearchResult } from '@/types';
import './SearchAnimations.css';

interface SearchResultsToggleProps {
  onResultClick?: (result: SearchResult) => void;
  onResultDoubleClick?: (result: SearchResult) => void;
  className?: string;
  // Auto-switch threshold (default: 500 results)
  virtualizationThreshold?: number;
  // Allow manual override
  allowManualToggle?: boolean;
}

export const SearchResultsToggle: React.FC<SearchResultsToggleProps> = ({
  onResultClick,
  onResultDoubleClick,
  className = '',
  virtualizationThreshold = 500,
  allowManualToggle = true
}) => {
  const { results, totalResults } = useActiveSearchResults();
  const [manualMode, setManualMode] = useState<'auto' | 'paginated' | 'virtualized'>('auto');

  // Determine which view to use
  const shouldUseVirtualization = useMemo(() => {
    if (manualMode === 'paginated') return false;
    if (manualMode === 'virtualized') return true;
    
    // Auto mode: use virtualization for large datasets
    return totalResults >= virtualizationThreshold;
  }, [totalResults, virtualizationThreshold, manualMode]);

  // Get performance metrics
  const performanceMetrics = useMemo(() => {
    const resultCount = results.length;
    
    if (resultCount === 0) {
      return {
        memoryImpact: 'minimal',
        renderingCost: 'low',
        recommendation: 'either'
      };
    }

    if (resultCount < 100) {
      return {
        memoryImpact: 'minimal',
        renderingCost: 'low', 
        recommendation: 'paginated'
      };
    } else if (resultCount < virtualizationThreshold) {
      return {
        memoryImpact: 'moderate',
        renderingCost: 'moderate',
        recommendation: 'either'
      };
    } else {
      return {
        memoryImpact: 'high',
        renderingCost: 'high',
        recommendation: 'virtualized'
      };
    }
  }, [results.length, virtualizationThreshold]);

  // Handle manual toggle
  const handleModeChange = useCallback((newMode: 'auto' | 'paginated' | 'virtualized') => {
    setManualMode(newMode);
  }, []);

  return (
    <div className={`search-results-toggle ${className}`}>
      {/* Performance Controls */}
      {allowManualToggle && totalResults > 100 && (
        <div className="performance-controls">
          <div className="performance-info">
            <div className="performance-stats">
              <span className="result-count">{totalResults.toLocaleString()} results</span>
              <span className="performance-impact" data-impact={performanceMetrics.memoryImpact}>
                Impact: {performanceMetrics.memoryImpact}
              </span>
              <span className="recommendation" data-recommendation={performanceMetrics.recommendation}>
                Recommended: {performanceMetrics.recommendation}
              </span>
            </div>
          </div>
          
          <div className="view-toggle">
            <label className="toggle-label">View Mode:</label>
            <div className="toggle-buttons">
              <button
                className={`toggle-btn btn-enhanced ${manualMode === 'auto' ? 'active' : ''}`}
                onClick={() => handleModeChange('auto')}
                title="Automatically choose best view based on result count"
              >
                <span className="btn-icon">🤖</span>
                Auto
              </button>
              <button
                className={`toggle-btn btn-enhanced ${manualMode === 'paginated' ? 'active' : ''}`}
                onClick={() => handleModeChange('paginated')}
                title="Use paginated view with traditional pagination controls"
              >
                <span className="btn-icon">📄</span>
                Paginated
              </button>
              <button
                className={`toggle-btn btn-enhanced ${manualMode === 'virtualized' ? 'active' : ''}`}
                onClick={() => handleModeChange('virtualized')}
                title="Use virtualized view for maximum performance"
              >
                <span className="btn-icon">⚡</span>
                Virtualized
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Performance Warning for Large Datasets */}
      {!shouldUseVirtualization && totalResults > virtualizationThreshold && (
        <div className="performance-warning">
          <div className="warning-content">
            <span className="warning-icon">⚠️</span>
            <div className="warning-text">
              <strong>Performance Notice:</strong> You have {totalResults.toLocaleString()} results. 
              Consider switching to virtualized view for better performance.
            </div>
            <button
              className="switch-btn btn-enhanced"
              onClick={() => handleModeChange('virtualized')}
            >
              Switch to Virtualized
            </button>
          </div>
        </div>
      )}

      {/* Current View Indicator */}
      {allowManualToggle && (
        <div className="current-view-info">
          <div className="view-indicator">
            <span className="indicator-icon">
              {shouldUseVirtualization ? '⚡' : '📄'}
            </span>
            <span className="indicator-text">
              Currently using <strong>{shouldUseVirtualization ? 'Virtualized' : 'Paginated'}</strong> view
              {manualMode === 'auto' && <em> (auto-selected)</em>}
            </span>
          </div>
          
          {shouldUseVirtualization && (
            <div className="virtualization-benefits">
              <div className="benefit-item">
                <span className="benefit-icon">🚀</span>
                <span className="benefit-text">Smooth scrolling for large datasets</span>
              </div>
              <div className="benefit-item">
                <span className="benefit-icon">💾</span>
                <span className="benefit-text">Reduced memory usage</span>
              </div>
              <div className="benefit-item">
                <span className="benefit-icon">⚡</span>
                <span className="benefit-text">Faster rendering performance</span>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Render Appropriate Component */}
      {shouldUseVirtualization ? (
        <VirtualizedSearchResults
          onResultClick={onResultClick}
          onResultDoubleClick={onResultDoubleClick}
          className="virtualized-view"
        />
      ) : (
        <SearchResults
          onResultClick={onResultClick}
          onResultDoubleClick={onResultDoubleClick}
          className="paginated-view"
        />
      )}
    </div>
  );
};