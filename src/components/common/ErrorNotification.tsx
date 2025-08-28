import React from 'react';
import './ErrorNotification.css';

export interface ErrorNotificationProps {
  id: string;
  message: string;
  type?: 'error' | 'warning' | 'info' | 'success';
  onDismiss?: (id: string) => void;
  onRetry?: () => void;
  retryLabel?: string;
  autoClose?: boolean;
  duration?: number;
}

export const ErrorNotification: React.FC<ErrorNotificationProps> = ({
  id,
  message,
  type = 'error',
  onDismiss,
  onRetry,
  retryLabel = 'Retry',
  autoClose = false,
  duration = 5000
}) => {
  React.useEffect(() => {
    if (autoClose && duration > 0) {
      const timer = setTimeout(() => {
        onDismiss?.(id);
      }, duration);
      
      return () => clearTimeout(timer);
    }
  }, [autoClose, duration, id, onDismiss]);

  const getIcon = () => {
    switch (type) {
      case 'error':
        return '❌';
      case 'warning':
        return '⚠️';
      case 'info':
        return 'ℹ️';
      default:
        return '❌';
    }
  };

  return (
    <div className={`error-notification ${type}`} role="alert" aria-live="polite">
      <div className="notification-content">
        <span className="notification-icon">{getIcon()}</span>
        <span className="notification-message">{message}</span>
      </div>
      
      <div className="notification-actions">
        {onRetry && (
          <button
            type="button"
            onClick={onRetry}
            className="notification-button retry"
            title="Retry operation"
          >
            🔄 {retryLabel}
          </button>
        )}
        
        {onDismiss && (
          <button
            type="button"
            onClick={() => onDismiss(id)}
            className="notification-button dismiss"
            title="Dismiss notification"
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
};

export default ErrorNotification;