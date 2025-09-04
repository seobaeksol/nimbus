/**
 * Saved Searches Component
 * 
 * Provides CRUD interface for managing saved search templates.
 * Allows users to save frequently used searches and rerun them.
 */

import React, { useCallback, useState } from 'react';
import { useAppDispatch, useAppSelector } from '@/store';
import { 
  saveSavedSearch, 
  updateSavedSearch, 
  deleteSavedSearch, 
  useSavedSearch, 
  clearSavedSearches 
} from '@/store/slices/searchSlice';
import { SavedSearch, SearchQuery } from '@/types';

interface SavedSearchesProps {
  onSearchSelect?: (savedSearch: SavedSearch) => void;
  onCreateSearch?: (query: SearchQuery) => void;
  className?: string;
}

interface SavedSearchFormData {
  name: string;
  description: string;
  tags: string[];
}

export const SavedSearches: React.FC<SavedSearchesProps> = ({
  onSearchSelect,
  onCreateSearch,
  className = ''
}) => {
  const dispatch = useAppDispatch();
  const savedSearches = useAppSelector(state => state.search.savedSearches);
  
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingSearch, setEditingSearch] = useState<SavedSearch | null>(null);
  const [searchFormData, setSearchFormData] = useState<SavedSearchFormData>({
    name: '',
    description: '',
    tags: []
  });

  // Handle saving a new search
  const handleSaveSearch = useCallback((query: SearchQuery, formData: SavedSearchFormData) => {
    const newSavedSearch: SavedSearch = {
      id: `saved_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      name: formData.name,
      description: formData.description,
      query,
      createdAt: new Date(),
      useCount: 0,
      tags: formData.tags
    };
    
    dispatch(saveSavedSearch(newSavedSearch));
    setShowCreateForm(false);
    resetForm();
  }, [dispatch]);

  // Handle updating an existing search
  const handleUpdateSearch = useCallback((updatedSearch: SavedSearch) => {
    dispatch(updateSavedSearch(updatedSearch));
    setEditingSearch(null);
    resetForm();
  }, [dispatch]);

  // Handle deleting a search
  const handleDeleteSearch = useCallback((searchId: string) => {
    if (confirm('Are you sure you want to delete this saved search?')) {
      dispatch(deleteSavedSearch(searchId));
    }
  }, [dispatch]);

  // Handle using a saved search
  const handleUseSearch = useCallback((savedSearch: SavedSearch) => {
    dispatch(useSavedSearch(savedSearch.id));
    onSearchSelect?.(savedSearch);
    onCreateSearch?.(savedSearch.query);
  }, [dispatch, onSearchSelect, onCreateSearch]);

  // Handle clearing all saved searches
  const handleClearAll = useCallback(() => {
    if (confirm('Are you sure you want to delete all saved searches?')) {
      dispatch(clearSavedSearches());
    }
  }, [dispatch]);

  // Reset form data
  const resetForm = useCallback(() => {
    setSearchFormData({
      name: '',
      description: '',
      tags: []
    });
  }, []);

  // Start editing a search
  const startEditing = useCallback((savedSearch: SavedSearch) => {
    setEditingSearch(savedSearch);
    setSearchFormData({
      name: savedSearch.name,
      description: savedSearch.description || '',
      tags: savedSearch.tags || []
    });
    setShowCreateForm(true);
  }, []);

  // Format search query for display
  const formatQuerySummary = useCallback((query: SearchQuery): string => {
    const parts: string[] = [];
    
    if (query.namePattern) {
      parts.push(`Name: "${query.namePattern}"`);
    }
    if (query.contentPattern) {
      parts.push(`Content: "${query.contentPattern}"`);
    }
    if (query.fileTypeFilter?.extensions.length) {
      parts.push(`Extensions: ${query.fileTypeFilter.extensions.join(', ')}`);
    }
    if (query.sizeFilter) {
      parts.push(`Size: ${query.sizeFilter.minSize || 0}-${query.sizeFilter.maxSize || '∞'} ${query.sizeFilter.unit}`);
    }
    
    return parts.join(' | ') || 'Empty search';
  }, []);

  // Handle form submission
  const handleSubmitForm = useCallback((e: React.FormEvent, query?: SearchQuery) => {
    e.preventDefault();
    
    if (!searchFormData.name.trim()) {
      alert('Please enter a name for the saved search');
      return;
    }
    
    if (editingSearch) {
      const updatedSearch: SavedSearch = {
        ...editingSearch,
        name: searchFormData.name,
        description: searchFormData.description,
        tags: searchFormData.tags
      };
      handleUpdateSearch(updatedSearch);
    } else if (query) {
      handleSaveSearch(query, searchFormData);
    }
  }, [searchFormData, editingSearch, handleSaveSearch, handleUpdateSearch]);

  return (
    <div className={`saved-searches ${className}`}>
      {/* Header */}
      <div className="saved-searches-header">
        <h3>Saved Searches</h3>
        <div className="header-actions">
          <button
            className="btn btn-primary"
            onClick={() => setShowCreateForm(true)}
            title="Save current search"
          >
            Save Current
          </button>
          {savedSearches.length > 0 && (
            <button
              className="btn btn-danger"
              onClick={handleClearAll}
              title="Clear all saved searches"
            >
              Clear All
            </button>
          )}
        </div>
      </div>

      {/* Create/Edit Form */}
      {showCreateForm && (
        <div className="save-search-form">
          <form onSubmit={(e) => handleSubmitForm(e)}>
            <div className="form-group">
              <label htmlFor="search-name">Name</label>
              <input
                type="text"
                id="search-name"
                value={searchFormData.name}
                onChange={(e) => setSearchFormData(prev => ({ ...prev, name: e.target.value }))}
                placeholder="Enter search name"
                required
              />
            </div>
            
            <div className="form-group">
              <label htmlFor="search-description">Description</label>
              <textarea
                id="search-description"
                value={searchFormData.description}
                onChange={(e) => setSearchFormData(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Optional description"
                rows={2}
              />
            </div>
            
            <div className="form-group">
              <label htmlFor="search-tags">Tags (comma-separated)</label>
              <input
                type="text"
                id="search-tags"
                value={searchFormData.tags.join(', ')}
                onChange={(e) => setSearchFormData(prev => ({ 
                  ...prev, 
                  tags: e.target.value.split(',').map(tag => tag.trim()).filter(tag => tag) 
                }))}
                placeholder="Optional tags"
              />
            </div>
            
            <div className="form-actions">
              <button type="submit" className="btn btn-primary">
                {editingSearch ? 'Update' : 'Save'} Search
              </button>
              <button 
                type="button" 
                className="btn btn-secondary"
                onClick={() => {
                  setShowCreateForm(false);
                  setEditingSearch(null);
                  resetForm();
                }}
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Saved Searches List */}
      {savedSearches.length === 0 ? (
        <div className="no-saved-searches">
          <div className="empty-state">
            <div className="empty-icon">🔖</div>
            <h4>No saved searches</h4>
            <p>Save frequently used searches for quick access</p>
          </div>
        </div>
      ) : (
        <div className="saved-searches-list">
          {savedSearches.map((savedSearch) => (
            <div key={savedSearch.id} className="saved-search-item">
              {/* Search Header */}
              <div className="saved-search-header">
                <div className="search-name">
                  <strong>{savedSearch.name}</strong>
                  {savedSearch.description && (
                    <span className="search-description">{savedSearch.description}</span>
                  )}
                </div>
                <div className="search-actions">
                  <button
                    className="btn btn-sm btn-primary"
                    onClick={() => handleUseSearch(savedSearch)}
                    title="Use this search"
                  >
                    Use
                  </button>
                  <button
                    className="btn btn-sm btn-secondary"
                    onClick={() => startEditing(savedSearch)}
                    title="Edit search"
                  >
                    Edit
                  </button>
                  <button
                    className="btn btn-sm btn-danger"
                    onClick={() => handleDeleteSearch(savedSearch.id)}
                    title="Delete search"
                  >
                    Delete
                  </button>
                </div>
              </div>

              {/* Search Details */}
              <div className="saved-search-details">
                <div className="search-query">
                  <strong>Query:</strong> {formatQuerySummary(savedSearch.query)}
                </div>
                <div className="search-meta">
                  <span className="created-date">
                    Created: {savedSearch.createdAt.toLocaleDateString()}
                  </span>
                  <span className="use-count">
                    Used: {savedSearch.useCount} times
                  </span>
                  {savedSearch.lastUsed && (
                    <span className="last-used">
                      Last used: {savedSearch.lastUsed.toLocaleDateString()}
                    </span>
                  )}
                </div>
                {savedSearch.tags && savedSearch.tags.length > 0 && (
                  <div className="search-tags">
                    {savedSearch.tags.map((tag, index) => (
                      <span key={index} className="tag">{tag}</span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};