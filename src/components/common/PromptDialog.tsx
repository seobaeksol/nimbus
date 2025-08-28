import React, { useState, useEffect, useRef } from 'react';
import './PromptDialog.css';

interface PromptDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  defaultValue?: string;
  placeholder?: string;
  confirmText?: string;
  cancelText?: string;
  inputType?: 'text' | 'password';
  maxLength?: number;
  onConfirm: (value: string) => void;
  onCancel: () => void;
}

const PromptDialog: React.FC<PromptDialogProps> = ({
  isOpen,
  title,
  message,
  defaultValue = '',
  placeholder = '',
  confirmText = 'OK',
  cancelText = 'Cancel',
  inputType = 'text',
  maxLength,
  onConfirm,
  onCancel,
}) => {
  const [value, setValue] = useState(defaultValue);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setValue(defaultValue);
      // Focus and select the input after the dialog is rendered
      setTimeout(() => {
        if (inputRef.current) {
          inputRef.current.focus();
          inputRef.current.select();
        }
      }, 0);
    }
  }, [isOpen, defaultValue]);

  if (!isOpen) return null;

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onCancel();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onCancel();
    } else if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleConfirm();
    }
  };

  const handleConfirm = () => {
    onConfirm(value.trim());
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    if (!maxLength || newValue.length <= maxLength) {
      setValue(newValue);
    }
  };

  return (
    <div 
      className="prompt-dialog-backdrop" 
      onClick={handleBackdropClick}
      onKeyDown={handleKeyDown}
      tabIndex={-1}
    >
      <div className="prompt-dialog">
        <div className="prompt-dialog-header">
          <h3 className="prompt-dialog-title">{title}</h3>
        </div>
        
        <div className="prompt-dialog-body">
          <p className="prompt-dialog-message">{message}</p>
          <input
            ref={inputRef}
            type={inputType}
            className="prompt-dialog-input"
            value={value}
            onChange={handleInputChange}
            placeholder={placeholder}
            maxLength={maxLength}
            onKeyDown={handleKeyDown}
          />
          {maxLength && (
            <div className="prompt-dialog-counter">
              {value.length}/{maxLength}
            </div>
          )}
        </div>
        
        <div className="prompt-dialog-footer">
          <button 
            className="prompt-dialog-button cancel"
            onClick={onCancel}
            type="button"
          >
            {cancelText}
          </button>
          <button 
            className="prompt-dialog-button confirm"
            onClick={handleConfirm}
            type="button"
            disabled={!value.trim()}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
};

export default PromptDialog;