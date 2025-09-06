import React from 'react';
import { PluginCardProps, PluginAction } from '../../types/plugins';
import './PluginCard.css';

export const PluginCard: React.FC<PluginCardProps> = ({
  plugin,
  onAction,
  isSelected = false,
  showDetails = false,
}) => {
  const handleAction = (actionType: PluginAction['type']) => {
    onAction({
      id: `${actionType}-${plugin.id}`,
      type: actionType,
      pluginId: plugin.id,
    });
  };

  const getStatusColor = () => {
    switch (plugin.status) {
      case 'Active': return 'var(--success-color)';
      case 'Inactive': return 'var(--warning-color)';
      case 'Error': return 'var(--error-color)';
      case 'Loading': return 'var(--info-color)';
      default: return 'var(--text-secondary)';
    }
  };

  const getTypeIcon = () => {
    switch (plugin.manifest.pluginType) {
      case 'Content': return '📁';
      case 'Protocol': return '🌐';
      case 'Viewer': return '👁️';
      default: return '🔌';
    }
  };

  const canEnable = plugin.status === 'Inactive';
  const canDisable = plugin.status === 'Active';
  const isWorking = plugin.status === 'Loading';

  return (
    <div className={`plugin-card ${isSelected ? 'selected' : ''}`}>
      <div className="plugin-card-header">
        <div className="plugin-icon">
          {getTypeIcon()}
        </div>
        <div className="plugin-info">
          <h3 className="plugin-name">{plugin.manifest.info.name}</h3>
          <p className="plugin-author">by {plugin.manifest.info.author}</p>
          <div className="plugin-meta">
            <span className="plugin-version">v{plugin.manifest.info.version}</span>
            <span 
              className="plugin-status"
              style={{ color: getStatusColor() }}
            >
              {plugin.status}
            </span>
            <span className="plugin-type">{plugin.manifest.pluginType}</span>
          </div>
        </div>
        <div className="plugin-actions">
          {canEnable && (
            <button
              className="plugin-action-btn enable"
              onClick={() => handleAction('enable')}
              disabled={isWorking}
              title="Enable plugin"
            >
              ▶️
            </button>
          )}
          {canDisable && (
            <button
              className="plugin-action-btn disable"
              onClick={() => handleAction('disable')}
              disabled={isWorking}
              title="Disable plugin"
            >
              ⏸️
            </button>
          )}
          <button
            className="plugin-action-btn unload"
            onClick={() => handleAction('unload')}
            disabled={isWorking}
            title="Unload plugin"
          >
            🗑️
          </button>
        </div>
      </div>

      <div className="plugin-description">
        {plugin.manifest.info.description}
      </div>

      {plugin.error && (
        <div className="plugin-error">
          <span className="error-icon">⚠️</span>
          <span className="error-message">{plugin.error}</span>
        </div>
      )}

      {showDetails && (
        <div className="plugin-details">
          <div className="plugin-tags">
            {plugin.manifest.info.tags.map(tag => (
              <span key={tag} className="plugin-tag">{tag}</span>
            ))}
          </div>
          
          <div className="plugin-metadata">
            <div className="metadata-row">
              <label>Loaded:</label>
              <span>{new Date(plugin.loadedAt).toLocaleString()}</span>
            </div>
            <div className="metadata-row">
              <label>Path:</label>
              <span className="plugin-path" title={plugin.pluginPath}>
                {plugin.pluginPath.split('/').pop()}
              </span>
            </div>
            {plugin.manifest.info.homepage && (
              <div className="metadata-row">
                <label>Homepage:</label>
                <a 
                  href={plugin.manifest.info.homepage} 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="plugin-link"
                >
                  {plugin.manifest.info.homepage}
                </a>
              </div>
            )}
            {plugin.manifest.info.license && (
              <div className="metadata-row">
                <label>License:</label>
                <span>{plugin.manifest.info.license}</span>
              </div>
            )}
            {plugin.manifest.dependencies.length > 0 && (
              <div className="metadata-row">
                <label>Dependencies:</label>
                <div className="plugin-dependencies">
                  {plugin.manifest.dependencies.map(dep => (
                    <span key={dep} className="dependency">{dep}</span>
                  ))}
                </div>
              </div>
            )}
          </div>

          <div className="plugin-detail-actions">
            <button
              className="btn btn-outline"
              onClick={() => handleAction('configure')}
              disabled={isWorking}
            >
              ⚙️ Configure
            </button>
            <button
              className="btn btn-outline"
              onClick={() => handleAction('update')}
              disabled={isWorking}
            >
              🔄 Update
            </button>
            <button
              className="btn btn-danger btn-outline"
              onClick={() => handleAction('delete')}
              disabled={isWorking}
            >
              🗑️ Uninstall
            </button>
          </div>
        </div>
      )}

      {isWorking && (
        <div className="plugin-loading">
          <div className="loading-spinner"></div>
          <span>Working...</span>
        </div>
      )}
    </div>
  );
};