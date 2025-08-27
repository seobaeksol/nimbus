import React from 'react';
import './ProgressIndicator.css';

export interface ProgressInfo {
  id: string;
  operation: 'copy' | 'move' | 'delete';
  fileName: string;
  progress: number; // 0-100
  totalFiles: number;
  currentFile: number;
  isComplete: boolean;
  error?: string;
}

interface ProgressIndicatorProps {
  progress: ProgressInfo;
  onCancel?: (id: string) => void;
  onDismiss?: (id: string) => void;
}

export const ProgressIndicator: React.FC<ProgressIndicatorProps> = ({
  progress,
  onCancel,
  onDismiss
}) => {
  const getOperationText = () => {
    switch (progress.operation) {
      case 'copy': return 'Copying';
      case 'move': return 'Moving';
      case 'delete': return 'Deleting';
      default: return 'Processing';
    }
  };

  const getStatusText = () => {
    if (progress.error) {
      return `Error: ${progress.error}`;
    }
    if (progress.isComplete) {
      return 'Complete';
    }
    if (progress.totalFiles > 1) {
      return `${getOperationText()} ${progress.currentFile} of ${progress.totalFiles} files`;
    }
    return `${getOperationText()} ${progress.fileName}`;
  };

  return (
    <div className={`progress-indicator ${progress.error ? 'error' : ''} ${progress.isComplete ? 'complete' : ''}`}>
      <div className="progress-header">
        <div className="progress-title">{getStatusText()}</div>
        <div className="progress-actions">
          {!progress.isComplete && !progress.error && onCancel && (
            <button 
              className="progress-button cancel"
              onClick={() => onCancel(progress.id)}
              title="Cancel operation"
            >
              ×
            </button>
          )}
          {(progress.isComplete || progress.error) && onDismiss && (
            <button 
              className="progress-button dismiss"
              onClick={() => onDismiss(progress.id)}
              title="Dismiss"
            >
              ×
            </button>
          )}
        </div>
      </div>
      
      {!progress.error && (
        <div className="progress-details">
          <div className="progress-filename">{progress.fileName}</div>
          <div className="progress-bar-container">
            <div 
              className="progress-bar"
              style={{ width: `${progress.progress}%` }}
            />
          </div>
          <div className="progress-percentage">{Math.round(progress.progress)}%</div>
        </div>
      )}
      
      {progress.error && (
        <div className="progress-error">
          {progress.error}
        </div>
      )}
    </div>
  );
};

export default ProgressIndicator;