import React, { useEffect, useState } from 'react';
import { useAppSelector, useAppDispatch } from '../../store';
import { setActivePanel, navigateToPath } from '../../store/slices/panelSlice';
import { CommandExecutor } from '../../services/commandExecutor';
import FilePanel from '../panels/FilePanel';
import PromptDialog from '../common/PromptDialog';
import CommandPalette from '../common/CommandPalette';
import './MultiPanelLayout.css';

const MultiPanelLayout: React.FC = () => {
  const dispatch = useAppDispatch();
  const { panels, activePanelId, gridLayout, panelOrder, clipboardState } = useAppSelector(state => state.panels);
  
  // Initialize CommandExecutor with context
  useEffect(() => {
    CommandExecutor.initialize({
      dispatch,
      panels,
      activePanelId,
      clipboardState
    });
  }, [dispatch, panels, activePanelId, clipboardState]);
  const [addressBarActive, setAddressBarActive] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [promptDialog, setPromptDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    placeholder?: string;
    onConfirm: (value: string) => void;
  }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
  });

  const handlePanelClick = (panelId: string) => {
    dispatch(setActivePanel(panelId));
  };

  const handleCreateFolder = async (panelId: string, input: string) => {
    await CommandExecutor.createFolder(panelId, input);
  };

  const handleCreateFile = async (panelId: string, input: string) => {
    await CommandExecutor.createFile(panelId, input);
  };


  // Global keyboard shortcut handler - only affects the active panel
  useEffect(() => {
    const handleGlobalKeyDown = (event: KeyboardEvent) => {
      // Handle Ctrl+Shift+P for Command Palette (works globally)
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'P') {
        event.preventDefault();
        setCommandPaletteOpen(true);
        return;
      }

      // Only handle other shortcuts if we have an active panel
      if (!activePanelId || !panels[activePanelId]) return;

      // Handle Ctrl+L for address bar focus
      if ((event.ctrlKey || event.metaKey) && event.key === 'l') {
        event.preventDefault();
        setAddressBarActive(true);
        return;
      }

      // Handle Ctrl+N for new folder
      if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
        event.preventDefault();
        setPromptDialog({
          isOpen: true,
          title: 'Create Folder',
          message: 'Enter folder path (relative to current directory or absolute):',
          placeholder: 'foldername or /path/to/folder or subdir/folder',
          onConfirm: (path: string) => {
            if (path) {
              handleCreateFolder(activePanelId, path);
            }
            setPromptDialog({ ...promptDialog, isOpen: false });
          }
        });
        return;
      }

      // Handle Ctrl+T for new file
      if ((event.ctrlKey || event.metaKey) && event.key === 't') {
        event.preventDefault();
        setPromptDialog({
          isOpen: true,
          title: 'Create File',
          message: 'Enter file path (relative to current directory or absolute):',
          placeholder: 'filename.txt or /path/to/file.txt or subdir/file.txt',
          onConfirm: (path: string) => {
            if (path) {
              handleCreateFile(activePanelId, path);
            }
            setPromptDialog({ ...promptDialog, isOpen: false });
          }
        });
        return;
      }

      // Future shortcuts can be added here
      // Examples: Ctrl+W (close tab), F3 (view), F4 (edit), etc.
    };

    document.addEventListener('keydown', handleGlobalKeyDown);
    return () => document.removeEventListener('keydown', handleGlobalKeyDown);
  }, [activePanelId, panels]);

  // Command Palette event handlers
  useEffect(() => {
    const handleCommandEvents = (event: CustomEvent) => {
      const { context } = event.detail;
      
      switch (event.type) {
        case 'command-palette-create-file':
          setPromptDialog({
            isOpen: true,
            title: 'Create File',
            message: 'Enter file path (relative to current directory or absolute):',
            placeholder: 'filename.txt or /path/to/file.txt or subdir/file.txt',
            onConfirm: (path: string) => {
              if (path && context.activePanelId) {
                handleCreateFile(context.activePanelId, path);
              }
              setPromptDialog({ ...promptDialog, isOpen: false });
            }
          });
          break;
          
        case 'command-palette-create-folder':
          setPromptDialog({
            isOpen: true,
            title: 'Create Folder',
            message: 'Enter folder path (relative to current directory or absolute):',
            placeholder: 'foldername or /path/to/folder or subdir/folder',
            onConfirm: (path: string) => {
              if (path && context.activePanelId) {
                handleCreateFolder(context.activePanelId, path);
              }
              setPromptDialog({ ...promptDialog, isOpen: false });
            }
          });
          break;
          
        case 'command-palette-focus-address-bar':
          setAddressBarActive(true);
          break;
          
        case 'command-palette-goto-path':
          setPromptDialog({
            isOpen: true,
            title: 'Go to Path',
            message: 'Enter path to navigate to:',
            placeholder: '/path/to/directory',
            onConfirm: async (path: string) => {
              if (path && context.activePanelId) {
                try {
                  await CommandExecutor.navigateToPath(context.activePanelId, path.trim());
                } catch (error) {
                  // Let CommandExecutor handle the error
                }
              }
              setPromptDialog({ ...promptDialog, isOpen: false });
            }
          });
          break;
      }
    };

    // Add event listeners for command palette events
    const events = [
      'command-palette-create-file',
      'command-palette-create-folder', 
      'command-palette-focus-address-bar',
      'command-palette-goto-path'
    ];
    
    events.forEach(eventType => {
      window.addEventListener(eventType, handleCommandEvents as EventListener);
    });
    
    return () => {
      events.forEach(eventType => {
        window.removeEventListener(eventType, handleCommandEvents as EventListener);
      });
    };
  }, [dispatch, promptDialog]);

  const gridStyle = {
    display: 'grid',
    gridTemplateRows: `repeat(${gridLayout.rows}, 1fr)`,
    gridTemplateColumns: `repeat(${gridLayout.cols}, 1fr)`,
    gap: '1px',
    height: '100%',
    width: '100%',
  };

  const visiblePanels = panelOrder.slice(0, gridLayout.rows * gridLayout.cols);

  return (
    <div className="multi-panel-layout">
      <div className="grid-container" style={gridStyle}>
        {visiblePanels.map((panelId) => {
          const panel = panels[panelId];
          if (!panel) return null;

          const isActive = activePanelId === panelId;

          return (
            <div
              key={panelId}
              className={`panel-wrapper ${isActive ? 'active' : ''}`}
              onClick={() => handlePanelClick(panelId)}
            >
              <FilePanel 
                panel={panel} 
                isActive={isActive}
                addressBarActive={isActive ? addressBarActive : false}
                onAddressBarFocus={() => setAddressBarActive(false)}
              />
            </div>
          );
        })}
      </div>

      <PromptDialog
        isOpen={promptDialog.isOpen}
        title={promptDialog.title}
        message={promptDialog.message}
        placeholder={promptDialog.placeholder}
        onConfirm={promptDialog.onConfirm}
        onCancel={() => setPromptDialog({ ...promptDialog, isOpen: false })}
      />

      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        dispatch={dispatch}
      />
    </div>
  );
};

export default MultiPanelLayout;