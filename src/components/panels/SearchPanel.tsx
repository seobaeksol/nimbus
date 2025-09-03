/**
 * Search Panel Component
 * 
 * Integrated search panel for the main Nimbus interface.
 * Provides file search functionality within the multi-panel layout.
 */

import React, { useCallback, useState } from 'react';
import { useAppDispatch } from '@/store';
import { navigateToPath, setActivePanel, selectFiles } from '@/store/slices/panelSlice';
import { SearchInterface } from '@/components/search';
import { SearchResult } from '@/types';
import './SearchPanel.css';
import '../search/SearchComponents.css';

interface SearchPanelProps {
  isActive?: boolean;
  onClose?: () => void;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({
  isActive = false,
  onClose
}) => {
  const dispatch = useAppDispatch();
  const [selectedResult, setSelectedResult] = useState<SearchResult | null>(null);

  // Handle search result selection - navigate to containing folder
  const handleResultSelect = useCallback((result: SearchResult) => {
    setSelectedResult(result);
  }, []);

  // Handle search result opening - navigate to file location in active file panel
  const handleResultOpen = useCallback(async (result: SearchResult) => {
    // Get the directory containing the file
    const containingDir = result.path.substring(0, result.path.lastIndexOf('/')) || '/';
    const fileName = result.name;
    
    // Find an available file panel to navigate to the result
    // For now, just use panel-1 as the target
    dispatch(navigateToPath({ 
      panelId: 'panel-1', 
      path: containingDir 
    }));
    
    // Make that panel active
    dispatch(setActivePanel('panel-1'));
    
    // Select the found file in the target panel after a brief delay
    // to allow the directory to load
    setTimeout(() => {
      dispatch(selectFiles({
        panelId: 'panel-1',
        fileNames: [fileName],
        toggle: false
      }));
    }, 500);
    
    // Close search panel after navigation
    onClose?.();
  }, [dispatch, onClose]);

  return (
    <div className={`search-panel-wrapper ${isActive ? 'active' : ''}`}>
      <div className="search-panel-header">
        <div className="panel-title">
          <span className="search-icon">🔍</span>
          <h3>Search Files</h3>
        </div>
        <div className="panel-controls">
          {onClose && (
            <button 
              className="close-button"
              onClick={onClose}
              aria-label="Close search panel"
            >
              ✕
            </button>
          )}
        </div>
      </div>

      <div className="search-panel-content">
        <SearchInterface
          onFileSelect={handleResultSelect}
          onFileOpen={handleResultOpen}
          className="integrated-search"
        />
      </div>

      {/* Selected result preview */}
      {selectedResult && (
        <div className="selected-result-preview">
          <div className="preview-header">
            <span className="preview-title">Selected File</span>
          </div>
          <div className="preview-content">
            <div className="result-info">
              <div className="result-name">{selectedResult.name}</div>
              <div className="result-path">{selectedResult.path}</div>
              <div className="result-meta">
                <span className="file-size">
                  {(selectedResult.size / 1024).toFixed(1)} KB
                </span>
                {selectedResult.modified && (
                  <span className="modified-date">
                    Modified: {new Date(selectedResult.modified).toLocaleDateString()}
                  </span>
                )}
              </div>
            </div>
            <div className="result-actions">
              <button
                className="action-button primary"
                onClick={() => handleResultOpen(selectedResult)}
              >
                Open Location
              </button>
              <button
                className="action-button secondary"
                onClick={() => setSelectedResult(null)}
              >
                Clear
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};