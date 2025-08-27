import React, { useEffect, useRef } from 'react';
import { FileInfo } from '../../types';
import './ContextMenu.css';

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  onClick?: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
  selectedFiles: FileInfo[];
}

const ContextMenu: React.FC<ContextMenuProps> = ({ x, y, items, onClose, selectedFiles }) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  // Ensure menu stays within viewport
  const menuStyle = {
    position: 'fixed' as const,
    left: `${Math.min(x, window.innerWidth - 200)}px`,
    top: `${Math.min(y, window.innerHeight - items.length * 32)}px`,
    zIndex: 1000,
  };

  const handleItemClick = (item: ContextMenuItem) => {
    if (!item.disabled && item.onClick) {
      item.onClick();
    }
    onClose();
  };

  return (
    <div ref={menuRef} className="context-menu" style={menuStyle}>
      <div className="context-menu-header">
        {selectedFiles.length === 1 
          ? selectedFiles[0].name 
          : `${selectedFiles.length} items selected`
        }
      </div>
      {items.map((item, index) => (
        item.separator ? (
          <div key={index} className="context-menu-separator" />
        ) : (
          <div
            key={item.id}
            className={`context-menu-item ${item.disabled ? 'disabled' : ''}`}
            onClick={() => handleItemClick(item)}
          >
            {item.icon && <span className="context-menu-icon">{item.icon}</span>}
            <span className="context-menu-label">{item.label}</span>
            {item.shortcut && (
              <span className="context-menu-shortcut">{item.shortcut}</span>
            )}
          </div>
        )
      ))}
    </div>
  );
};

export default ContextMenu;