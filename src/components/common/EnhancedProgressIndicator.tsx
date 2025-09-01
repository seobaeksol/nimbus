import React, { useState, useEffect } from 'react';
import './EnhancedProgressIndicator.css';

export interface EnhancedProgressInfo {
  id: string;
  operation: 'copy' | 'move' | 'delete' | 'extract' | 'compress' | 'search';
  fileName: string;
  progress: number; // 0-100
  totalFiles: number;
  currentFile: number;
  bytesProcessed?: number;
  totalBytes?: number;
  speed?: number; // bytes per second
  eta?: number; // estimated time remaining in seconds
  isComplete: boolean;
  error?: string;
  status: 'running' | 'paused' | 'completed' | 'error' | 'cancelled';
}

interface EnhancedProgressIndicatorProps {
  progress: EnhancedProgressInfo;
  onCancel?: (id: string) => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
  onDismiss?: (id: string) => void;
  showDetails?: boolean;
  compact?: boolean;
}

export const EnhancedProgressIndicator: React.FC<EnhancedProgressIndicatorProps> = ({
  progress,
  onCancel,
  onPause,
  onResume,
  onDismiss,
  showDetails = true,
  compact = false
}) => {
  const [isExpanded, setIsExpanded] = useState(!compact);
  const [animatedProgress, setAnimatedProgress] = useState(0);

  // Animate progress bar
  useEffect(() => {
    const timer = setTimeout(() => {
      setAnimatedProgress(progress.progress);
    }, 100);
    return () => clearTimeout(timer);
  }, [progress.progress]);

  const getOperationIcon = () => {
    switch (progress.operation) {
      case 'copy': return '📋';
      case 'move': return '✂️';
      case 'delete': return '🗑️';
      case 'extract': return '📦';
      case 'compress': return '🗜️';
      case 'search': return '🔍';
      default: return '⚙️';
    }
  };

  const getOperationText = () => {
    switch (progress.operation) {
      case 'copy': return 'Copying';
      case 'move': return 'Moving';
      case 'delete': return 'Deleting';
      case 'extract': return 'Extracting';
      case 'compress': return 'Compressing';
      case 'search': return 'Searching';
      default: return 'Processing';
    }
  };

  const getStatusText = () => {
    if (progress.error) {
      return `Error: ${progress.error}`;
    }
    
    switch (progress.status) {
      case 'completed':
        return 'Completed successfully';
      case 'cancelled':
        return 'Operation cancelled';
      case 'paused':
        return 'Paused';
      case 'error':
        return progress.error || 'An error occurred';
      default:
        if (progress.totalFiles > 1) {
          return `${getOperationText()} ${progress.currentFile} of ${progress.totalFiles} files`;
        }
        return `${getOperationText()} ${progress.fileName}`;
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const formatTime = (seconds: number): string => {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
    return `${Math.round(seconds / 3600)}h`;
  };

  const getSpeedText = (): string => {
    if (!progress.speed || progress.speed === 0) return '';
    return `${formatBytes(progress.speed)}/s`;
  };

  const getETAText = (): string => {
    if (!progress.eta || progress.eta === 0) return '';
    return `${formatTime(progress.eta)} remaining`;
  };

  const progressBarClass = `enhanced-progress-bar ${
    progress.status === 'error' ? 'error' : 
    progress.status === 'completed' ? 'completed' :
    progress.status === 'paused' ? 'paused' : ''
  }`;

  return (
    <div className={`enhanced-progress-indicator ${compact ? 'compact' : ''} ${progress.status}`}>
      <div className="enhanced-progress-header">
        <div className="enhanced-progress-info">
          <span className="enhanced-progress-icon" role="img" aria-label={progress.operation}>
            {getOperationIcon()}
          </span>
          <div className="enhanced-progress-text">
            <div className="enhanced-progress-title">{getStatusText()}</div>
            {showDetails && !compact && (
              <div className="enhanced-progress-filename" title={progress.fileName}>
                {progress.fileName}
              </div>
            )}
          </div>
        </div>
        
        <div className="enhanced-progress-actions">
          {progress.status === 'running' && onPause && (
            <button 
              className="enhanced-progress-button pause"
              onClick={() => onPause(progress.id)}
              title="Pause operation"
              aria-label="Pause"
            >
              ⏸️
            </button>
          )}
          
          {progress.status === 'paused' && onResume && (
            <button 
              className="enhanced-progress-button resume"
              onClick={() => onResume(progress.id)}
              title="Resume operation"
              aria-label="Resume"
            >
              ▶️
            </button>
          )}
          
          {!['completed', 'error', 'cancelled'].includes(progress.status) && onCancel && (
            <button 
              className="enhanced-progress-button cancel"
              onClick={() => onCancel(progress.id)}
              title="Cancel operation"
              aria-label="Cancel"
            >
              ✕
            </button>
          )}
          
          {['completed', 'error', 'cancelled'].includes(progress.status) && onDismiss && (
            <button 
              className="enhanced-progress-button dismiss"
              onClick={() => onDismiss(progress.id)}
              title="Dismiss notification"
              aria-label="Dismiss"
            >
              ✕
            </button>
          )}
          
          {!compact && (
            <button 
              className="enhanced-progress-button expand"
              onClick={() => setIsExpanded(!isExpanded)}
              title={isExpanded ? "Collapse details" : "Expand details"}
              aria-label={isExpanded ? "Collapse" : "Expand"}
            >
              {isExpanded ? '▼' : '▶'}
            </button>
          )}
        </div>
      </div>
      
      {!progress.error && (
        <div className="enhanced-progress-bar-container">
          <div 
            className={progressBarClass}
            style={{ 
              width: `${animatedProgress}%`,
              transition: 'width 0.3s ease-out'
            }}
            role="progressbar"
            aria-valuenow={progress.progress}
            aria-valuemin={0}
            aria-valuemax={100}
          />
          <div className="enhanced-progress-percentage">
            {Math.round(progress.progress)}%
          </div>
        </div>
      )}
      
      {isExpanded && showDetails && !compact && (
        <div className="enhanced-progress-details">
          {progress.bytesProcessed !== undefined && progress.totalBytes !== undefined && (
            <div className="enhanced-progress-bytes">
              {formatBytes(progress.bytesProcessed)} / {formatBytes(progress.totalBytes)}
            </div>
          )}
          
          <div className="enhanced-progress-stats">
            {progress.speed && (
              <span className="enhanced-progress-speed">{getSpeedText()}</span>
            )}
            {progress.eta && (
              <span className="enhanced-progress-eta">{getETAText()}</span>
            )}
          </div>
        </div>
      )}
      
      {progress.error && (
        <div className="enhanced-progress-error">
          <span className="enhanced-progress-error-icon">⚠️</span>
          <span className="enhanced-progress-error-text">{progress.error}</span>
        </div>
      )}
    </div>
  );
};

export default EnhancedProgressIndicator;