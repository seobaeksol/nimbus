import React, { useEffect, useState } from 'react';
import { useAppSelector, useAppDispatch } from '../../store';
import { setActivePanel, setLoading, setError, navigateToPath } from '../../store/slices/panelSlice';
import { FileService } from '../../services/fileService';
import FilePanel from '../panels/FilePanel';
import PromptDialog from '../common/PromptDialog';
import './MultiPanelLayout.css';

const MultiPanelLayout: React.FC = () => {
  const dispatch = useAppDispatch();
  const { panels, activePanelId, gridLayout, panelOrder } = useAppSelector(state => state.panels);
  const [addressBarActive, setAddressBarActive] = useState(false);
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

  const handleCreateFolder = async (panelId: string, name: string) => {
    try {
      const panel = panels[panelId];
      if (!panel) return;

      dispatch(setLoading({ panelId, isLoading: true }));
      await FileService.createDirectory(panel.currentPath, name);
      
      // Refresh the panel to show the new folder
      dispatch(navigateToPath({ panelId, path: panel.currentPath }));
    } catch (error) {
      console.error('Failed to create folder:', error);
      dispatch(setError({ 
        panelId, 
        error: `Failed to create folder: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const handleCreateFile = async (panelId: string, name: string) => {
    try {
      const panel = panels[panelId];
      if (!panel) return;

      dispatch(setLoading({ panelId, isLoading: true }));
      await FileService.createFile(panel.currentPath, name);
      
      // Refresh the panel to show the new file
      dispatch(navigateToPath({ panelId, path: panel.currentPath }));
    } catch (error) {
      console.error('Failed to create file:', error);
      dispatch(setError({ 
        panelId, 
        error: `Failed to create file: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  // Global keyboard shortcut handler - only affects the active panel
  useEffect(() => {
    const handleGlobalKeyDown = (event: KeyboardEvent) => {
      // Only handle shortcuts if we have an active panel
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
          message: 'Enter folder name:',
          placeholder: 'New Folder',
          onConfirm: (name: string) => {
            if (name) {
              handleCreateFolder(activePanelId, name);
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
          message: 'Enter file name:',
          placeholder: 'filename.txt',
          onConfirm: (name: string) => {
            if (name) {
              handleCreateFile(activePanelId, name);
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
    </div>
  );
};

export default MultiPanelLayout;