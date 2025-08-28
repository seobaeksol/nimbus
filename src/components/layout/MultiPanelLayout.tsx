import React, { useEffect, useState } from 'react';
import { useAppSelector, useAppDispatch } from '../../store';
import { setActivePanel, setLoading, setError, navigateToPath } from '../../store/slices/panelSlice';
import { FileService } from '../../services/fileService';
import FilePanel from '../panels/FilePanel';
import PromptDialog from '../common/PromptDialog';
import CommandPalette from '../common/CommandPalette';
import './MultiPanelLayout.css';

const MultiPanelLayout: React.FC = () => {
  const dispatch = useAppDispatch();
  const { panels, activePanelId, gridLayout, panelOrder } = useAppSelector(state => state.panels);
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
    try {
      const panel = panels[panelId];
      if (!panel) return;

      dispatch(setLoading({ panelId, isLoading: true }));
      
      // Parse the input to determine target directory and folder name
      const { targetDir, fileName: folderName } = parseCreateFileInput(input, panel.currentPath);
      
      // Create the folder
      await FileService.createDirectory(targetDir, folderName);
      
      // Navigate to the directory containing the created folder if it's different from current
      const finalPath = targetDir === panel.currentPath ? panel.currentPath : targetDir;
      dispatch(navigateToPath({ panelId, path: finalPath }));
    } catch (error) {
      console.error('Failed to create folder:', error);
      dispatch(setLoading({ panelId, isLoading: false }));
      dispatch(setError({ 
        panelId, 
        error: `Failed to create folder: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const handleCreateFile = async (panelId: string, input: string) => {
    try {
      const panel = panels[panelId];
      if (!panel) return;

      dispatch(setLoading({ panelId, isLoading: true }));
      
      // Parse the input to determine target directory and filename
      const { targetDir, fileName } = parseCreateFileInput(input, panel.currentPath);
      
      // Create the file
      await FileService.createFile(targetDir, fileName);
      
      // Navigate to the directory containing the created file if it's different from current
      const finalPath = targetDir === panel.currentPath ? panel.currentPath : targetDir;
      dispatch(navigateToPath({ panelId, path: finalPath }));
    } catch (error) {
      console.error('Failed to create file:', error);
      dispatch(setLoading({ panelId, isLoading: false }));
      dispatch(setError({ 
        panelId, 
        error: `Failed to create file: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const parseCreateFileInput = (input: string, currentPath: string) => {
    const trimmedInput = input.trim();
    
    // Check if input is an absolute path (starts with / or C:\ on Windows)
    const isAbsolute = trimmedInput.startsWith('/') || /^[A-Za-z]:\\/.test(trimmedInput);
    
    let fullPath: string;
    if (isAbsolute) {
      fullPath = trimmedInput;
    } else {
      // Relative path - resolve against current panel path
      fullPath = currentPath.endsWith('/') 
        ? currentPath + trimmedInput 
        : currentPath + '/' + trimmedInput;
    }
    
    // Normalize path separators
    fullPath = fullPath.replace(/\\/g, '/');
    
    // Split into directory and filename
    const lastSlashIndex = fullPath.lastIndexOf('/');
    if (lastSlashIndex === -1) {
      // No directory separator, create in current directory
      return {
        targetDir: currentPath,
        fileName: fullPath
      };
    }
    
    const targetDir = lastSlashIndex === 0 ? '/' : fullPath.substring(0, lastSlashIndex);
    const fileName = fullPath.substring(lastSlashIndex + 1);
    
    if (!fileName) {
      throw new Error('Filename cannot be empty');
    }
    
    return { targetDir, fileName };
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
            onConfirm: (path: string) => {
              if (path && context.activePanelId) {
                dispatch(navigateToPath({ 
                  panelId: context.activePanelId, 
                  path: path.trim() 
                }));
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