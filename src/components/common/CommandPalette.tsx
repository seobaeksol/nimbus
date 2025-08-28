import React, { useState, useEffect, useRef } from 'react';
import { useAppSelector, AppDispatch } from '../../store';
import { CommandRegistry } from '../../services/commandRegistry';
import { Command, CommandContext } from '../../types/commands';
import './CommandPalette.css';

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  dispatch: AppDispatch;
}

const CommandPalette: React.FC<CommandPaletteProps> = ({ isOpen, onClose, dispatch: appDispatch }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [filteredCommands, setFilteredCommands] = useState<Command[]>([]);
  
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Get application state for command context
  const { panels, activePanelId } = useAppSelector(state => state.panels);
  const { clipboardState } = useAppSelector(state => state.panels);
  
  const activePanel = activePanelId ? panels[activePanelId] : null;
  const selectedFiles = activePanel?.selectedFiles.map(fileName => 
    activePanel.files.find(file => file.name === fileName)
  ).filter(Boolean) || [];

  // Initialize command registry
  useEffect(() => {
    CommandRegistry.initialize();
  }, []);

  // Create command context
  const commandContext: CommandContext = {
    activePanelId,
    selectedFiles: selectedFiles as any[], // Type assertion for now
    currentPath: activePanel?.currentPath || '/',
    dispatch: appDispatch,
    panels,
    clipboardHasFiles: clipboardState.hasFiles
  };

  // Filter commands based on search term
  useEffect(() => {
    const commands = CommandRegistry.searchCommands(searchTerm, commandContext);
    setFilteredCommands(commands);
    setSelectedIndex(0);
  }, [searchTerm, activePanelId, activePanel?.selectedFiles, clipboardState.hasFiles]);

  // Focus input when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      setSearchTerm('');
      setSelectedIndex(0);
      inputRef.current.focus();
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current && filteredCommands.length > 0) {
      const selectedElement = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({
          block: 'nearest',
          behavior: 'smooth'
        });
      }
    }
  }, [selectedIndex, filteredCommands]);

  if (!isOpen) return null;

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'Escape':
        onClose();
        break;
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => 
          prev < filteredCommands.length - 1 ? prev + 1 : prev
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => prev > 0 ? prev - 1 : prev);
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          executeCommand(filteredCommands[selectedIndex]);
        }
        break;
      case 'Tab':
        e.preventDefault();
        // Cycle through results
        setSelectedIndex(prev => 
          prev < filteredCommands.length - 1 ? prev + 1 : 0
        );
        break;
    }
  };

  const executeCommand = (command: Command) => {
    // Close palette first
    onClose();
    
    // Create fresh context at execution time to ensure current state
    const currentActivePanel = activePanelId ? panels[activePanelId] : null;
    const currentSelectedFiles = currentActivePanel?.selectedFiles.map(fileName => 
      currentActivePanel.files.find(file => file.name === fileName)
    ).filter(Boolean) || [];

    const executionContext: CommandContext = {
      activePanelId,
      selectedFiles: currentSelectedFiles as any[],
      currentPath: currentActivePanel?.currentPath || '/',
      dispatch: appDispatch,
      panels,
      clipboardHasFiles: clipboardState.hasFiles
    };
    
    // Execute command with fresh context
    try {
      command.action(executionContext);
    } catch (error) {
      console.error('Failed to execute command:', command.id, error);
    }
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  const formatShortcut = (shortcut?: string) => {
    if (!shortcut) return null;
    
    // Replace common shortcuts with symbols
    return shortcut
      .replace('Ctrl+', '⌘')
      .replace('Alt+', '⌥')
      .replace('Shift+', '⇧');
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'File': return '📁';
      case 'Navigation': return '🧭';
      case 'Panel': return '🗂️';
      case 'View': return '👁️';
      case 'System': return '⚙️';
      default: return '🔧';
    }
  };

  return (
    <div 
      className="command-palette-backdrop" 
      onClick={handleBackdropClick}
      tabIndex={-1}
    >
      <div className="command-palette">
        <div className="command-palette-header">
          <div className="command-palette-search">
            <span className="command-palette-search-icon">🔍</span>
            <input
              ref={inputRef}
              type="text"
              className="command-palette-input"
              placeholder="Type a command or search..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              onKeyDown={handleKeyDown}
            />
          </div>
        </div>

        <div className="command-palette-body" ref={listRef}>
          {filteredCommands.length === 0 ? (
            <div className="command-palette-empty">
              {searchTerm ? 'No commands found' : 'No commands available'}
            </div>
          ) : (
            filteredCommands.map((command, index) => (
              <div
                key={command.id}
                className={`command-palette-item ${index === selectedIndex ? 'selected' : ''}`}
                onClick={() => executeCommand(command)}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <div className="command-palette-item-content">
                  <div className="command-palette-item-icon">
                    {command.icon || getCategoryIcon(command.category)}
                  </div>
                  <div className="command-palette-item-main">
                    <div className="command-palette-item-label">
                      {command.label}
                    </div>
                    {command.description && (
                      <div className="command-palette-item-description">
                        {command.description}
                      </div>
                    )}
                  </div>
                  <div className="command-palette-item-meta">
                    <div className="command-palette-item-category">
                      {command.category}
                    </div>
                    {command.shortcut && (
                      <div className="command-palette-item-shortcut">
                        {formatShortcut(command.shortcut)}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="command-palette-footer">
          <div className="command-palette-help">
            <span className="command-palette-help-item">↑↓ Navigate</span>
            <span className="command-palette-help-item">↵ Execute</span>
            <span className="command-palette-help-item">⎋ Close</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CommandPalette;