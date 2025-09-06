import React from 'react';
import { PluginFilter, PluginSortBy, PluginType, PluginStatus } from '../../types/plugins';
import './PluginFilters.css';

interface PluginFiltersProps {
  filter: PluginFilter;
  sortBy: PluginSortBy;
  availableTags: string[];
  onFilterChange: (filter: Partial<PluginFilter>) => void;
  onSortChange: (sortBy: PluginSortBy) => void;
  onClearFilter: () => void;
  totalCount: number;
  filteredCount: number;
}

export const PluginFilters: React.FC<PluginFiltersProps> = ({
  filter,
  sortBy,
  availableTags,
  onFilterChange,
  onSortChange,
  onClearFilter,
  totalCount,
  filteredCount,
}) => {
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onFilterChange({ searchTerm: e.target.value || undefined });
  };

  const handleTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    onFilterChange({ type: value === 'all' ? undefined : value as PluginType });
  };

  const handleStatusChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    onFilterChange({ status: value === 'all' ? undefined : value as PluginStatus });
  };

  const handleTagToggle = (tag: string) => {
    const currentTags = filter.tags || [];
    const newTags = currentTags.includes(tag)
      ? currentTags.filter(t => t !== tag)
      : [...currentTags, tag];
    
    onFilterChange({ tags: newTags.length > 0 ? newTags : undefined });
  };

  const handleSortFieldChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onSortChange({
      field: e.target.value as PluginSortBy['field'],
      direction: sortBy.direction,
    });
  };

  const handleSortDirectionChange = () => {
    onSortChange({
      field: sortBy.field,
      direction: sortBy.direction === 'asc' ? 'desc' : 'asc',
    });
  };

  const hasActiveFilters = Boolean(
    filter.searchTerm || 
    filter.type || 
    filter.status || 
    (filter.tags && filter.tags.length > 0)
  );

  const isFiltered = filteredCount !== totalCount;

  return (
    <div className="plugin-filters">
      <div className="filters-header">
        <div className="filter-info">
          <span className="filter-count">
            {isFiltered ? `${filteredCount} of ${totalCount}` : totalCount} plugins
          </span>
          {hasActiveFilters && (
            <button
              className="clear-filters-btn"
              onClick={onClearFilter}
              title="Clear all filters"
            >
              Clear filters
            </button>
          )}
        </div>
        
        <div className="sort-controls">
          <label htmlFor="sort-field">Sort by:</label>
          <select
            id="sort-field"
            value={sortBy.field}
            onChange={handleSortFieldChange}
            className="sort-select"
          >
            <option value="name">Name</option>
            <option value="type">Type</option>
            <option value="status">Status</option>
            <option value="author">Author</option>
            <option value="loadedAt">Load Date</option>
          </select>
          <button
            className="sort-direction-btn"
            onClick={handleSortDirectionChange}
            title={`Sort ${sortBy.direction === 'asc' ? 'descending' : 'ascending'}`}
          >
            {sortBy.direction === 'asc' ? '↓' : '↑'}
          </button>
        </div>
      </div>

      <div className="filters-content">
        <div className="filter-row">
          <div className="search-filter">
            <label htmlFor="plugin-search">Search:</label>
            <div className="search-input-container">
              <input
                id="plugin-search"
                type="text"
                placeholder="Search by name, description, author, or tags..."
                value={filter.searchTerm || ''}
                onChange={handleSearchChange}
                className="search-input"
              />
              {filter.searchTerm && (
                <button
                  className="clear-search-btn"
                  onClick={() => onFilterChange({ searchTerm: undefined })}
                  title="Clear search"
                >
                  ×
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="filter-row">
          <div className="select-filter">
            <label htmlFor="type-filter">Type:</label>
            <select
              id="type-filter"
              value={filter.type || 'all'}
              onChange={handleTypeChange}
              className="filter-select"
            >
              <option value="all">All Types</option>
              <option value="Content">📁 Content</option>
              <option value="Protocol">🌐 Protocol</option>
              <option value="Viewer">👁️ Viewer</option>
            </select>
          </div>

          <div className="select-filter">
            <label htmlFor="status-filter">Status:</label>
            <select
              id="status-filter"
              value={filter.status || 'all'}
              onChange={handleStatusChange}
              className="filter-select"
            >
              <option value="all">All Status</option>
              <option value="Active">✅ Active</option>
              <option value="Inactive">⏸️ Inactive</option>
              <option value="Error">❌ Error</option>
              <option value="Loading">🔄 Loading</option>
            </select>
          </div>
        </div>

        {availableTags.length > 0 && (
          <div className="filter-row">
            <div className="tag-filter">
              <label>Tags:</label>
              <div className="tag-list">
                {availableTags.map(tag => {
                  const isSelected = filter.tags?.includes(tag) || false;
                  return (
                    <button
                      key={tag}
                      className={`tag-button ${isSelected ? 'selected' : ''}`}
                      onClick={() => handleTagToggle(tag)}
                      title={`Filter by ${tag} tag`}
                    >
                      {tag}
                      {isSelected && <span className="tag-selected-icon">✓</span>}
                    </button>
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>

      {hasActiveFilters && (
        <div className="active-filters">
          <span className="active-filters-label">Active filters:</span>
          <div className="active-filter-tags">
            {filter.searchTerm && (
              <span className="active-filter-tag">
                Search: "{filter.searchTerm}"
                <button
                  onClick={() => onFilterChange({ searchTerm: undefined })}
                  className="remove-filter-btn"
                >
                  ×
                </button>
              </span>
            )}
            {filter.type && (
              <span className="active-filter-tag">
                Type: {filter.type}
                <button
                  onClick={() => onFilterChange({ type: undefined })}
                  className="remove-filter-btn"
                >
                  ×
                </button>
              </span>
            )}
            {filter.status && (
              <span className="active-filter-tag">
                Status: {filter.status}
                <button
                  onClick={() => onFilterChange({ status: undefined })}
                  className="remove-filter-btn"
                >
                  ×
                </button>
              </span>
            )}
            {filter.tags?.map(tag => (
              <span key={tag} className="active-filter-tag">
                Tag: {tag}
                <button
                  onClick={() => handleTagToggle(tag)}
                  className="remove-filter-btn"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};