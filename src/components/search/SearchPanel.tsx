/**
 * Advanced Search Panel Component
 * 
 * Provides a comprehensive search interface with fuzzy matching, filters,
 * and real-time results with highlighting.
 */

import React, { useState, useCallback } from 'react';
import { useSearch } from '@/hooks/useSearch';
import { SearchOptions, SearchQuery, FileCategory } from '@/types';

interface SearchPanelProps {
  initialPath?: string;
  className?: string;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({
  initialPath = '/',
  className = ''
}) => {
  // Search hook
  const { 
    startSearch, 
    cancelSearch, 
    activeSearch, 
    isSearching,
    createDefaultOptions 
  } = useSearch();

  // Form state
  const [searchForm, setSearchForm] = useState({
    namePattern: '',
    contentPattern: '',
    rootPath: initialPath,
  });

  // Advanced options state
  const [options, setOptions] = useState<SearchOptions>(createDefaultOptions());
  const [showAdvanced, setShowAdvanced] = useState(false);

  // Filters state
  const [filters, setFilters] = useState({
    sizeFilter: {
      enabled: false,
      minSize: '',
      maxSize: '',
      unit: 'mb' as const,
    },
    dateFilter: {
      enabled: false,
      dateType: 'modified' as const,
      startDate: '',
      endDate: '',
    },
    fileTypeFilter: {
      enabled: false,
      extensions: [] as string[],
      categories: [] as FileCategory[],
    },
  });

  // Handle search submission
  const handleSearch = useCallback(async () => {
    if (!searchForm.namePattern && !searchForm.contentPattern) {
      alert('Please enter a search pattern');
      return;
    }

    try {
      const query: SearchQuery = {
        rootPath: searchForm.rootPath,
        namePattern: searchForm.namePattern || undefined,
        contentPattern: searchForm.contentPattern || undefined,
        sizeFilter: filters.sizeFilter.enabled ? {
          minSize: filters.sizeFilter.minSize ? parseInt(filters.sizeFilter.minSize) : undefined,
          maxSize: filters.sizeFilter.maxSize ? parseInt(filters.sizeFilter.maxSize) : undefined,
          unit: filters.sizeFilter.unit,
        } : undefined,
        dateFilter: filters.dateFilter.enabled ? {
          dateType: filters.dateFilter.dateType,
          startDate: filters.dateFilter.startDate || undefined,
          endDate: filters.dateFilter.endDate || undefined,
        } : undefined,
        fileTypeFilter: filters.fileTypeFilter.enabled ? {
          extensions: filters.fileTypeFilter.extensions,
          categories: filters.fileTypeFilter.categories,
        } : undefined,
        options,
      };

      await startSearch(query);
    } catch (error) {
      console.error('Search failed:', error);
      alert(`Search failed: ${error}`);
    }
  }, [searchForm, filters, options, startSearch]);

  // Handle search cancellation
  const handleCancel = useCallback(async () => {
    if (activeSearch) {
      try {
        await cancelSearch(activeSearch.id);
      } catch (error) {
        console.error('Cancel failed:', error);
      }
    }
  }, [activeSearch, cancelSearch]);

  // Handle form updates
  const updateSearchForm = useCallback((field: string, value: string) => {
    setSearchForm(prev => ({ ...prev, [field]: value }));
  }, []);

  // Handle options updates
  const updateOption = useCallback((field: keyof SearchOptions, value: any) => {
    setOptions(prev => ({ ...prev, [field]: value }));
  }, []);

  // Handle filter updates
  const updateFilter = useCallback((filterType: string, field: string, value: any) => {
    setFilters(prev => ({
      ...prev,
      [filterType]: {
        ...prev[filterType as keyof typeof prev],
        [field]: value,
      },
    }));
  }, []);

  // Handle extension input
  const handleExtensionInput = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      const value = (e.target as HTMLInputElement).value.trim();
      if (value) {
        const extensions = [...filters.fileTypeFilter.extensions, value];
        updateFilter('fileTypeFilter', 'extensions', extensions);
        (e.target as HTMLInputElement).value = '';
      }
    }
  }, [filters.fileTypeFilter.extensions, updateFilter]);

  // Remove extension
  const removeExtension = useCallback((ext: string) => {
    const extensions = filters.fileTypeFilter.extensions.filter(e => e !== ext);
    updateFilter('fileTypeFilter', 'extensions', extensions);
  }, [filters.fileTypeFilter.extensions, updateFilter]);

  // Toggle category
  const toggleCategory = useCallback((category: FileCategory) => {
    const categories = filters.fileTypeFilter.categories.includes(category)
      ? filters.fileTypeFilter.categories.filter(c => c !== category)
      : [...filters.fileTypeFilter.categories, category];
    updateFilter('fileTypeFilter', 'categories', categories);
  }, [filters.fileTypeFilter.categories, updateFilter]);

  return (
    <div className={`search-panel ${className}`}>
      <div className="search-header">
        <h3>Advanced Search</h3>
        <button 
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="toggle-advanced"
        >
          {showAdvanced ? 'Hide Advanced' : 'Show Advanced'}
        </button>
      </div>

      {/* Basic Search Form */}
      <div className="search-form">
        <div className="form-group">
          <label htmlFor="search-path">Search Path:</label>
          <input
            id="search-path"
            type="text"
            value={searchForm.rootPath}
            onChange={(e) => updateSearchForm('rootPath', e.target.value)}
            placeholder="e.g., /home/user/documents"
          />
        </div>

        <div className="form-group">
          <label htmlFor="name-pattern">File Name Pattern:</label>
          <input
            id="name-pattern"
            type="text"
            value={searchForm.namePattern}
            onChange={(e) => updateSearchForm('namePattern', e.target.value)}
            placeholder="e.g., *.pdf or config"
          />
        </div>

        <div className="form-group">
          <label htmlFor="content-pattern">Content Pattern:</label>
          <input
            id="content-pattern"
            type="text"
            value={searchForm.contentPattern}
            onChange={(e) => updateSearchForm('contentPattern', e.target.value)}
            placeholder="e.g., TODO or function main"
          />
        </div>

        {/* Search Options */}
        <div className="search-options">
          <label>
            <input
              type="checkbox"
              checked={options.useFuzzy}
              onChange={(e) => updateOption('useFuzzy', e.target.checked)}
            />
            Enable Fuzzy Search
          </label>

          {options.useFuzzy && (
            <div className="fuzzy-options">
              <label>
                Fuzzy Threshold: {options.fuzzyThreshold}
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={options.fuzzyThreshold}
                  onChange={(e) => updateOption('fuzzyThreshold', parseInt(e.target.value))}
                />
              </label>
            </div>
          )}

          <label>
            <input
              type="checkbox"
              checked={options.caseSensitive}
              onChange={(e) => updateOption('caseSensitive', e.target.checked)}
            />
            Case Sensitive
          </label>

          <label>
            <input
              type="checkbox"
              checked={options.useRegex}
              onChange={(e) => updateOption('useRegex', e.target.checked)}
            />
            Use Regex
          </label>

          <label>
            <input
              type="checkbox"
              checked={options.includeHidden}
              onChange={(e) => updateOption('includeHidden', e.target.checked)}
            />
            Include Hidden Files
          </label>
        </div>

        {/* Advanced Filters */}
        {showAdvanced && (
          <div className="advanced-filters">
            <h4>Advanced Filters</h4>

            {/* Size Filter */}
            <div className="filter-section">
              <label>
                <input
                  type="checkbox"
                  checked={filters.sizeFilter.enabled}
                  onChange={(e) => updateFilter('sizeFilter', 'enabled', e.target.checked)}
                />
                Size Filter
              </label>
              
              {filters.sizeFilter.enabled && (
                <div className="size-filter">
                  <input
                    type="number"
                    placeholder="Min size"
                    value={filters.sizeFilter.minSize}
                    onChange={(e) => updateFilter('sizeFilter', 'minSize', e.target.value)}
                  />
                  <span>to</span>
                  <input
                    type="number"
                    placeholder="Max size"
                    value={filters.sizeFilter.maxSize}
                    onChange={(e) => updateFilter('sizeFilter', 'maxSize', e.target.value)}
                  />
                  <select
                    value={filters.sizeFilter.unit}
                    onChange={(e) => updateFilter('sizeFilter', 'unit', e.target.value)}
                  >
                    <option value="bytes">Bytes</option>
                    <option value="kb">KB</option>
                    <option value="mb">MB</option>
                    <option value="gb">GB</option>
                  </select>
                </div>
              )}
            </div>

            {/* Date Filter */}
            <div className="filter-section">
              <label>
                <input
                  type="checkbox"
                  checked={filters.dateFilter.enabled}
                  onChange={(e) => updateFilter('dateFilter', 'enabled', e.target.checked)}
                />
                Date Filter
              </label>
              
              {filters.dateFilter.enabled && (
                <div className="date-filter">
                  <select
                    value={filters.dateFilter.dateType}
                    onChange={(e) => updateFilter('dateFilter', 'dateType', e.target.value)}
                  >
                    <option value="modified">Modified</option>
                    <option value="created">Created</option>
                    <option value="accessed">Accessed</option>
                  </select>
                  <input
                    type="date"
                    value={filters.dateFilter.startDate}
                    onChange={(e) => updateFilter('dateFilter', 'startDate', e.target.value)}
                  />
                  <span>to</span>
                  <input
                    type="date"
                    value={filters.dateFilter.endDate}
                    onChange={(e) => updateFilter('dateFilter', 'endDate', e.target.value)}
                  />
                </div>
              )}
            </div>

            {/* File Type Filter */}
            <div className="filter-section">
              <label>
                <input
                  type="checkbox"
                  checked={filters.fileTypeFilter.enabled}
                  onChange={(e) => updateFilter('fileTypeFilter', 'enabled', e.target.checked)}
                />
                File Type Filter
              </label>
              
              {filters.fileTypeFilter.enabled && (
                <div className="filetype-filter">
                  <div className="extensions">
                    <label>Extensions:</label>
                    <input
                      type="text"
                      placeholder="Type extension and press Enter"
                      onKeyDown={handleExtensionInput}
                    />
                    <div className="extension-tags">
                      {filters.fileTypeFilter.extensions.map(ext => (
                        <span key={ext} className="tag">
                          {ext}
                          <button onClick={() => removeExtension(ext)}>×</button>
                        </span>
                      ))}
                    </div>
                  </div>
                  
                  <div className="categories">
                    <label>Categories:</label>
                    {(['documents', 'images', 'audio', 'video', 'archives', 'code'] as FileCategory[]).map(category => (
                      <label key={category}>
                        <input
                          type="checkbox"
                          checked={filters.fileTypeFilter.categories.includes(category)}
                          onChange={() => toggleCategory(category)}
                        />
                        {category}
                      </label>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Action Buttons */}
        <div className="search-actions">
          <button
            onClick={handleSearch}
            disabled={isSearching}
            className="search-btn"
          >
            {isSearching ? 'Searching...' : 'Search'}
          </button>
          
          {isSearching && (
            <button
              onClick={handleCancel}
              className="cancel-btn"
            >
              Cancel
            </button>
          )}
        </div>

        {/* Search Status */}
        {activeSearch && (
          <div className="search-status">
            <p>Status: {activeSearch.status}</p>
            <p>Results: {activeSearch.totalResults}</p>
            {activeSearch.error && (
              <p className="error">Error: {activeSearch.error}</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
};