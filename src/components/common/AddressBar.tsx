import React, { useState, useRef, useEffect } from 'react';
import { PathAliasService } from '../../services/pathAliasService';
import './AddressBar.css';

export interface AddressBarProps {
  currentPath: string;
  isActive?: boolean;
  onNavigate: (path: string) => void;
  onError?: (error: string) => void;
  onFocus?: () => void;
  className?: string;
}

export const AddressBar: React.FC<AddressBarProps> = ({
  currentPath,
  isActive = false,
  onNavigate,
  onError,
  onFocus,
  className = ''
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus input when entering edit mode or when Ctrl+L is pressed
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  // External focus trigger (for Ctrl+L)
  useEffect(() => {
    if (isActive && !isEditing) {
      handleStartEdit();
    }
  }, [isActive]);

  const handleStartEdit = () => {
    setInputValue(currentPath);
    setIsEditing(true);
    setError(null);
    onFocus?.();
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setInputValue('');
    setError(null);
  };

  const handleConfirmEdit = async () => {
    const trimmedPath = inputValue.trim();
    
    if (!trimmedPath) {
      setError('Path cannot be empty');
      return;
    }

    if (trimmedPath === currentPath) {
      handleCancelEdit();
      return;
    }

    try {
      await onNavigate(trimmedPath);
      setIsEditing(false);
      setInputValue('');
      setError(null);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Invalid path';
      setError(errorMessage);
      onError?.(errorMessage);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'Enter':
        e.preventDefault();
        handleConfirmEdit();
        break;
      case 'Escape':
        e.preventDefault();
        handleCancelEdit();
        break;
    }
  };

  const handleClick = () => {
    if (!isEditing) {
      handleStartEdit();
    }
  };

  const handleBlur = (e: React.FocusEvent) => {
    // Don't cancel edit if clicking within the address bar
    if (e.currentTarget.contains(e.relatedTarget as Node)) {
      return;
    }
    handleCancelEdit();
  };

  const formatDisplayPath = (path: string): string => {
    try {
      // Use PathAliasService for consistent path formatting
      return PathAliasService.formatForDisplay(path);
    } catch (error) {
      // Fallback to simple formatting if service fails
      if (path === '/') return '/';
      
      // Replace home directory with ~ for display
      const homeDirs = ['/Users/', '/home/', 'C:\\Users\\'];
      for (const homeDir of homeDirs) {
        if (path.startsWith(homeDir)) {
          const username = path.split('/')[2] || path.split('\\')[2];
          if (username) {
            return path.replace(`${homeDir}${username}`, '~');
          }
        }
      }
      
      return path;
    }
  };

  const displayPath = formatDisplayPath(currentPath);

  if (isEditing) {
    return (
      <div className={`address-bar editing ${error ? 'error' : ''} ${className}`}>
        <div className="address-input-container">
          <input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            onBlur={handleBlur}
            className="address-input"
            placeholder="Enter path..."
            aria-label="Address bar input"
            spellCheck={false}
          />
          <div className="address-actions">
            <button
              type="button"
              onClick={handleConfirmEdit}
              className="address-action-button confirm"
              title="Navigate (Enter)"
              aria-label="Navigate to path"
            >
              ✓
            </button>
            <button
              type="button"
              onClick={handleCancelEdit}
              className="address-action-button cancel"
              title="Cancel (Escape)"
              aria-label="Cancel edit"
            >
              ×
            </button>
          </div>
        </div>
        {error && (
          <div className="address-error" role="alert" aria-live="polite">
            {error}
          </div>
        )}
      </div>
    );
  }

  return (
    <div 
      className={`address-bar display ${className}`}
      onClick={handleClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          handleClick();
        }
      }}
      aria-label={`Current path: ${displayPath}. Click to edit.`}
      title="Click to edit address (Ctrl+L)"
    >
      <div className="address-display">
        <span className="address-path">{displayPath}</span>
        <div className="address-edit-hint">
          <span className="edit-icon">✏️</span>
        </div>
      </div>
    </div>
  );
};

export default AddressBar;