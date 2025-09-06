import React, { useEffect, useState } from 'react';
import { usePlugins } from '../../hooks/usePlugins';
import { PluginList } from './PluginList';
import { PluginFilters } from './PluginFilters';
import { PluginInstallRequest } from '../../types/plugins';
import './PluginManager.css';

export const PluginManager: React.FC = () => {
  const {
    allPlugins,
    filteredPlugins,
    pluginStats,
    settings,
    filter,
    sortBy,
    isLoading,
    error,
    installProgress,
    initialize,
    discoverPlugins,
    loadPlugin,
    executePluginAction,
    updatePluginSettings,
    installPlugin,
    setPluginFilter,
    clearPluginFilter,
    setPluginSortBy,
    refreshPlugins,
  } = usePlugins();

  const [showInstallDialog, setShowInstallDialog] = useState(false);
  const [showSettingsDialog, setShowSettingsDialog] = useState(false);

  // Initialize plugin system on mount
  useEffect(() => {
    initialize();
  }, [initialize]);

  // Get all available tags for filtering
  const availableTags = React.useMemo(() => {
    const tagSet = new Set<string>();
    allPlugins.forEach(plugin => {
      plugin.manifest.info.tags.forEach(tag => tagSet.add(tag));
    });
    return Array.from(tagSet).sort();
  }, [allPlugins]);

  const handleInstallFromFile = async () => {
    try {
      // In a real app, this would open a file dialog
      const filePath = prompt('Enter plugin file path:');
      if (!filePath) return;

      const request: PluginInstallRequest = {
        source: 'file',
        path: filePath,
      };

      await installPlugin(request);
      setShowInstallDialog(false);
    } catch (error) {
      console.error('Failed to install plugin:', error);
    }
  };

  const handleInstallFromUrl = async () => {
    try {
      const url = prompt('Enter plugin URL:');
      if (!url) return;

      const request: PluginInstallRequest = {
        source: 'url',
        url: url,
      };

      await installPlugin(request);
      setShowInstallDialog(false);
    } catch (error) {
      console.error('Failed to install plugin:', error);
    }
  };

  const handleLoadFromFile = async () => {
    try {
      // In a real app, this would open a file dialog
      const filePath = prompt('Enter plugin file path to load:');
      if (!filePath) return;

      await loadPlugin(filePath);
    } catch (error) {
      console.error('Failed to load plugin:', error);
    }
  };

  return (
    <div className="plugin-manager">
      <div className="plugin-manager-header">
        <div className="header-title">
          <h1>Plugin Manager</h1>
          <div className="plugin-overview">
            <span className="overview-stat">
              {pluginStats.totalPlugins} total
            </span>
            <span className="overview-stat active">
              {pluginStats.activePlugins} active
            </span>
            {pluginStats.errorPlugins > 0 && (
              <span className="overview-stat error">
                {pluginStats.errorPlugins} errors
              </span>
            )}
          </div>
        </div>

        <div className="header-actions">
          <button
            className="btn btn-outline"
            onClick={() => setShowSettingsDialog(true)}
            title="Plugin settings"
          >
            ⚙️ Settings
          </button>
          <button
            className="btn btn-outline"
            onClick={discoverPlugins}
            disabled={isLoading}
            title="Discover plugins"
          >
            🔍 Discover
          </button>
          <button
            className="btn btn-outline"
            onClick={refreshPlugins}
            disabled={isLoading}
            title="Refresh plugin list"
          >
            🔄 Refresh
          </button>
          <div className="dropdown">
            <button
              className="btn btn-primary dropdown-toggle"
              onClick={() => setShowInstallDialog(!showInstallDialog)}
              disabled={isLoading}
            >
              ➕ Add Plugin
            </button>
            {showInstallDialog && (
              <div className="dropdown-menu">
                <button className="dropdown-item" onClick={handleLoadFromFile}>
                  📁 Load from File
                </button>
                <button className="dropdown-item" onClick={handleInstallFromFile}>
                  💾 Install from File
                </button>
                <button className="dropdown-item" onClick={handleInstallFromUrl}>
                  🌐 Install from URL
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {error && (
        <div className="plugin-manager-error">
          <div className="error-content">
            <span className="error-icon">⚠️</span>
            <span className="error-message">{error}</span>
            <button
              className="error-dismiss"
              onClick={() => {/* Clear error */}}
              title="Dismiss error"
            >
              ×
            </button>
          </div>
        </div>
      )}

      {installProgress && (
        <div className="install-progress">
          <div className="progress-content">
            <div className="progress-info">
              <span className="progress-plugin">{installProgress.pluginName}</span>
              <span className="progress-stage">{installProgress.stage}</span>
            </div>
            <div className="progress-bar">
              <div 
                className="progress-fill"
                style={{ width: `${installProgress.progress}%` }}
              />
            </div>
            <span className="progress-percentage">{installProgress.progress}%</span>
          </div>
          {installProgress.message && (
            <div className="progress-message">{installProgress.message}</div>
          )}
        </div>
      )}

      <div className="plugin-manager-content">
        <PluginFilters
          filter={filter}
          sortBy={sortBy}
          availableTags={availableTags}
          onFilterChange={setPluginFilter}
          onSortChange={setPluginSortBy}
          onClearFilter={clearPluginFilter}
          totalCount={allPlugins.length}
          filteredCount={filteredPlugins.length}
        />

        <PluginList
          plugins={filteredPlugins}
          onPluginAction={executePluginAction}
          filter={filter}
          sortBy={sortBy}
        />
      </div>

      {isLoading && (
        <div className="plugin-manager-loading">
          <div className="loading-spinner-large">
            <div className="spinner"></div>
          </div>
          <span>Loading plugins...</span>
        </div>
      )}

      {showSettingsDialog && (
        <div className="modal-overlay" onClick={() => setShowSettingsDialog(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Plugin Settings</h2>
              <button
                className="modal-close"
                onClick={() => setShowSettingsDialog(false)}
              >
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="setting-group">
                <label>Plugin Directories:</label>
                <div className="directory-list">
                  {settings.pluginDirectories.map((dir, index) => (
                    <div key={index} className="directory-item">
                      <span className="directory-path">{dir}</span>
                      <button className="remove-directory">×</button>
                    </div>
                  ))}
                  <button className="add-directory">+ Add Directory</button>
                </div>
              </div>
              
              <div className="setting-group">
                <label>
                  <input
                    type="checkbox"
                    checked={settings.recursiveScan}
                    onChange={e => updatePluginSettings({ recursiveScan: e.target.checked })}
                  />
                  Recursive scan
                </label>
              </div>
              
              <div className="setting-group">
                <label>
                  <input
                    type="checkbox"
                    checked={settings.autoLoad}
                    onChange={e => updatePluginSettings({ autoLoad: e.target.checked })}
                  />
                  Auto-load discovered plugins
                </label>
              </div>
              
              <div className="setting-group">
                <label>
                  Load timeout (seconds):
                  <input
                    type="number"
                    value={settings.loadTimeout}
                    min="1"
                    max="300"
                    onChange={e => updatePluginSettings({ loadTimeout: parseInt(e.target.value) })}
                  />
                </label>
              </div>
              
              <div className="setting-group">
                <label>
                  <input
                    type="checkbox"
                    checked={settings.verifySignatures}
                    onChange={e => updatePluginSettings({ verifySignatures: e.target.checked })}
                  />
                  Verify plugin signatures
                </label>
              </div>
            </div>
            <div className="modal-footer">
              <button
                className="btn btn-outline"
                onClick={() => setShowSettingsDialog(false)}
              >
                Cancel
              </button>
              <button
                className="btn btn-primary"
                onClick={() => setShowSettingsDialog(false)}
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};