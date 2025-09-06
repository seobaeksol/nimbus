import React, { useState } from 'react';
import { PluginListProps, PluginSortBy } from '../../types/plugins';
import { PluginCard } from './PluginCard';
import './PluginList.css';

export const PluginList: React.FC<PluginListProps> = ({
  plugins,
  onPluginAction,
  filter,
  sortBy,
}) => {
  const [selectedPluginId, setSelectedPluginId] = useState<string | undefined>();
  const [showDetails, setShowDetails] = useState<Record<string, boolean>>({});

  const handleCardClick = (pluginId: string) => {
    setSelectedPluginId(prevId => prevId === pluginId ? undefined : pluginId);
  };

  const toggleDetails = (pluginId: string) => {
    setShowDetails(prev => ({
      ...prev,
      [pluginId]: !prev[pluginId]
    }));
  };

  const getFilteredPlugins = () => {
    let filtered = plugins;

    if (filter?.type) {
      filtered = filtered.filter(plugin => plugin.manifest.pluginType === filter.type);
    }

    if (filter?.status) {
      filtered = filtered.filter(plugin => plugin.status === filter.status);
    }

    if (filter?.searchTerm) {
      const searchTerm = filter.searchTerm.toLowerCase();
      filtered = filtered.filter(plugin =>
        plugin.manifest.info.name.toLowerCase().includes(searchTerm) ||
        plugin.manifest.info.description.toLowerCase().includes(searchTerm) ||
        plugin.manifest.info.author.toLowerCase().includes(searchTerm) ||
        plugin.manifest.info.tags.some(tag => tag.toLowerCase().includes(searchTerm))
      );
    }

    if (filter?.tags && filter.tags.length > 0) {
      filtered = filtered.filter(plugin =>
        filter.tags!.every(tag => plugin.manifest.info.tags.includes(tag))
      );
    }

    return filtered;
  };

  const getSortedPlugins = (pluginsToSort: typeof plugins) => {
    if (!sortBy) return pluginsToSort;

    return [...pluginsToSort].sort((a, b) => {
      let aValue: any;
      let bValue: any;

      switch (sortBy.field) {
        case 'name':
          aValue = a.manifest.info.name.toLowerCase();
          bValue = b.manifest.info.name.toLowerCase();
          break;
        case 'type':
          aValue = a.manifest.pluginType;
          bValue = b.manifest.pluginType;
          break;
        case 'status':
          aValue = a.status;
          bValue = b.status;
          break;
        case 'loadedAt':
          aValue = new Date(a.loadedAt);
          bValue = new Date(b.loadedAt);
          break;
        case 'author':
          aValue = a.manifest.info.author.toLowerCase();
          bValue = b.manifest.info.author.toLowerCase();
          break;
        default:
          return 0;
      }

      if (aValue < bValue) {
        return sortBy.direction === 'asc' ? -1 : 1;
      }
      if (aValue > bValue) {
        return sortBy.direction === 'asc' ? 1 : -1;
      }
      return 0;
    });
  };

  const displayedPlugins = getSortedPlugins(getFilteredPlugins());

  const getStatusCounts = () => {
    const counts = {
      total: plugins.length,
      active: plugins.filter(p => p.status === 'Active').length,
      inactive: plugins.filter(p => p.status === 'Inactive').length,
      error: plugins.filter(p => p.status === 'Error').length,
      loading: plugins.filter(p => p.status === 'Loading').length,
    };
    return counts;
  };

  const getTypeCounts = () => {
    const counts = {
      content: plugins.filter(p => p.manifest.pluginType === 'Content').length,
      protocol: plugins.filter(p => p.manifest.pluginType === 'Protocol').length,
      viewer: plugins.filter(p => p.manifest.pluginType === 'Viewer').length,
    };
    return counts;
  };

  const statusCounts = getStatusCounts();
  const typeCounts = getTypeCounts();

  if (plugins.length === 0) {
    return (
      <div className="plugin-list-empty">
        <div className="empty-icon">🔌</div>
        <h3>No plugins loaded</h3>
        <p>Load plugins to extend Nimbus functionality.</p>
      </div>
    );
  }

  return (
    <div className="plugin-list">
      <div className="plugin-list-header">
        <div className="plugin-summary">
          <h2>Plugins ({displayedPlugins.length})</h2>
          <div className="plugin-stats">
            <div className="stat-group">
              <span className="stat-label">Status:</span>
              <span className="stat active" title="Active plugins">
                ✅ {statusCounts.active}
              </span>
              <span className="stat inactive" title="Inactive plugins">
                ⏸️ {statusCounts.inactive}
              </span>
              {statusCounts.error > 0 && (
                <span className="stat error" title="Plugins with errors">
                  ❌ {statusCounts.error}
                </span>
              )}
              {statusCounts.loading > 0 && (
                <span className="stat loading" title="Loading plugins">
                  🔄 {statusCounts.loading}
                </span>
              )}
            </div>
            <div className="stat-group">
              <span className="stat-label">Types:</span>
              <span className="stat type" title="Content plugins">
                📁 {typeCounts.content}
              </span>
              <span className="stat type" title="Protocol plugins">
                🌐 {typeCounts.protocol}
              </span>
              <span className="stat type" title="Viewer plugins">
                👁️ {typeCounts.viewer}
              </span>
            </div>
          </div>
        </div>

        <div className="plugin-list-actions">
          <button
            className="btn btn-outline"
            onClick={() => setShowDetails(prev => {
              const allExpanded = Object.values(prev).every(Boolean);
              return displayedPlugins.reduce((acc, plugin) => {
                acc[plugin.id] = !allExpanded;
                return acc;
              }, {} as Record<string, boolean>);
            })}
            title="Toggle details for all plugins"
          >
            {Object.values(showDetails).every(Boolean) ? '📕 Collapse All' : '📖 Expand All'}
          </button>
        </div>
      </div>

      {displayedPlugins.length === 0 ? (
        <div className="plugin-list-empty filtered">
          <div className="empty-icon">🔍</div>
          <h3>No plugins match your filters</h3>
          <p>Try adjusting your search criteria.</p>
        </div>
      ) : (
        <div className="plugin-cards">
          {displayedPlugins.map(plugin => (
            <div key={plugin.id} className="plugin-card-container">
              <PluginCard
                plugin={plugin}
                onAction={onPluginAction}
                isSelected={selectedPluginId === plugin.id}
                showDetails={showDetails[plugin.id] || false}
              />
              <div className="plugin-card-overlay">
                <button
                  className="details-toggle"
                  onClick={() => toggleDetails(plugin.id)}
                  title={showDetails[plugin.id] ? 'Hide details' : 'Show details'}
                >
                  {showDetails[plugin.id] ? '▼' : '▶'}
                </button>
                <button
                  className="select-toggle"
                  onClick={() => handleCardClick(plugin.id)}
                  title={selectedPluginId === plugin.id ? 'Deselect' : 'Select'}
                >
                  {selectedPluginId === plugin.id ? '☑️' : '☐'}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};