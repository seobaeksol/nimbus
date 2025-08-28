import React from 'react';
import { useAppSelector, useAppDispatch } from '../../store';
import { removeNotification } from '../../store/slices/panelSlice';
import ErrorNotification from './ErrorNotification';
import './NotificationContainer.css';

interface NotificationContainerProps {
  panelId?: string; // If specified, only show notifications for this panel
  className?: string;
}

const NotificationContainer: React.FC<NotificationContainerProps> = ({ 
  panelId, 
  className = '' 
}) => {
  const dispatch = useAppDispatch();
  const { notifications } = useAppSelector(state => state.panels);

  // Filter notifications by panel if specified
  const relevantNotifications = panelId 
    ? notifications.filter(n => n.panelId === panelId)
    : notifications;

  const handleDismiss = (id: string) => {
    dispatch(removeNotification(id));
  };

  const handleRetry = (notification: any) => {
    // TODO: Implement retry logic based on notification.retryAction
    console.log('Retry action:', notification.retryAction, notification.retryData);
    // For now, just dismiss the notification
    dispatch(removeNotification(notification.id));
  };

  if (relevantNotifications.length === 0) {
    return null;
  }

  return (
    <div className={`notification-container ${className}`}>
      {relevantNotifications.map((notification) => (
        <ErrorNotification
          key={notification.id}
          id={notification.id}
          message={notification.message}
          type={notification.type}
          onDismiss={handleDismiss}
          onRetry={notification.retryAction ? () => handleRetry(notification) : undefined}
          retryLabel={notification.retryAction ? 'Retry' : undefined}
          autoClose={notification.autoClose}
          duration={notification.duration}
        />
      ))}
    </div>
  );
};

export default NotificationContainer;