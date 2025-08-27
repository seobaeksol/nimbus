import React from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { RootState } from '../../store';
import { removeProgressIndicator, clearCompletedProgress } from '../../store/slices/panelSlice';
import ProgressIndicator from './ProgressIndicator';
import './ProgressContainer.css';

export const ProgressContainer: React.FC = () => {
  const dispatch = useDispatch();
  const progressIndicators = useSelector((state: RootState) => state.panels.progressIndicators);

  const handleDismiss = (id: string) => {
    dispatch(removeProgressIndicator(id));
  };

  const handleCancel = (id: string) => {
    // For now, just remove the indicator
    // In a full implementation, you'd cancel the actual operation
    dispatch(removeProgressIndicator(id));
  };

  const handleClearAll = () => {
    dispatch(clearCompletedProgress());
  };

  if (progressIndicators.length === 0) {
    return null;
  }

  const hasCompleted = progressIndicators.some(p => p.isComplete || p.error);

  return (
    <div className="progress-container">
      <div className="progress-container-header">
        <span className="progress-container-title">
          File Operations ({progressIndicators.length})
        </span>
        {hasCompleted && (
          <button
            className="progress-clear-button"
            onClick={handleClearAll}
            title="Clear completed operations"
          >
            Clear Completed
          </button>
        )}
      </div>
      <div className="progress-list">
        {progressIndicators.map((progress) => (
          <ProgressIndicator
            key={progress.id}
            progress={progress}
            onCancel={handleCancel}
            onDismiss={handleDismiss}
          />
        ))}
      </div>
    </div>
  );
};

export default ProgressContainer;