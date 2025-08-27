import React, { useEffect, useState } from 'react';
import { Panel, selectFiles, navigateToPath, setFiles, setLoading, setError } from '../../store/slices/panelSlice';
import { useAppDispatch } from '../../store';
import { FileService } from '../../services/fileService';
import { FileInfo } from '../../types';
import ContextMenu, { ContextMenuItem } from '../common/ContextMenu';
import ConfirmDialog from '../common/ConfirmDialog';
import './FilePanel.css';

interface FilePanelProps {
  panel: Panel;
}

const FilePanel: React.FC<FilePanelProps> = ({ panel }) => {
  const dispatch = useAppDispatch();
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    selectedFiles: FileInfo[];
  } | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    variant?: 'default' | 'danger';
    onConfirm: () => void;
  }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
  });

  useEffect(() => {
    loadDirectory(panel.currentPath);
  }, [panel.currentPath]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const selectedFileInfos = panel.files.filter(f => panel.selectedFiles.includes(f.name));
      
      if (selectedFileInfos.length === 0) return;
      
      switch (event.key) {
        case 'Delete':
          event.preventDefault();
          const message = selectedFileInfos.length === 1 
            ? `Are you sure you want to delete "${selectedFileInfos[0].name}"?`
            : `Are you sure you want to delete ${selectedFileInfos.length} selected items?`;
          
          showConfirmDialog(
            'Confirm Delete',
            message,
            () => handleDeleteFiles(selectedFileInfos),
            'danger'
          );
          break;
          
        case 'F2':
          event.preventDefault();
          if (selectedFileInfos.length === 1) {
            const newName = prompt('Enter new name:', selectedFileInfos[0].name);
            if (newName && newName !== selectedFileInfos[0].name) {
              handleRenameFile(selectedFileInfos[0], newName);
            }
          }
          break;
          
        case 'c':
          if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            // TODO: Implement copy to clipboard
            console.log('Copy files:', selectedFileInfos.map(f => f.name));
          }
          break;
          
        case 'x':
          if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            // TODO: Implement cut to clipboard
            console.log('Cut files:', selectedFileInfos.map(f => f.name));
          }
          break;
          
        case 'v':
          if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            // TODO: Implement paste from clipboard
            console.log('Paste files to:', panel.currentPath);
          }
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [panel.files, panel.selectedFiles, panel.currentPath]);

  const loadDirectory = async (path: string) => {
    try {
      dispatch(setLoading({ panelId: panel.id, isLoading: true }));
      const files = await FileService.listDirectory(path);
      dispatch(setFiles({ panelId: panel.id, files }));
    } catch (error) {
      console.error('Failed to load directory:', error);
      dispatch(setError({ 
        panelId: panel.id, 
        error: error instanceof Error ? error.message : 'Failed to load directory'
      }));
    }
  };

  const handleFileClick = (file: FileInfo, event: React.MouseEvent) => {
    if (event.ctrlKey || event.metaKey) {
      // Multi-select with Ctrl/Cmd
      dispatch(selectFiles({ 
        panelId: panel.id, 
        fileNames: [file.name], 
        toggle: true 
      }));
    } else if (event.shiftKey && panel.selectedFiles.length > 0) {
      // Range select with Shift
      const fileNames = panel.files.map(f => f.name);
      const lastSelected = panel.selectedFiles[panel.selectedFiles.length - 1];
      const lastIndex = fileNames.indexOf(lastSelected);
      const currentIndex = fileNames.indexOf(file.name);
      
      const start = Math.min(lastIndex, currentIndex);
      const end = Math.max(lastIndex, currentIndex);
      const rangeFiles = fileNames.slice(start, end + 1);
      
      dispatch(selectFiles({ panelId: panel.id, fileNames: rangeFiles }));
    } else {
      // Single select
      dispatch(selectFiles({ panelId: panel.id, fileNames: [file.name] }));
    }
  };

  const handleFileDoubleClick = (file: FileInfo) => {
    if (file.file_type === 'Directory') {
      const newPath = panel.currentPath === '/' 
        ? `/${file.name}` 
        : `${panel.currentPath}/${file.name}`;
      dispatch(navigateToPath({ panelId: panel.id, path: newPath }));
    }
  };

  const handleBackClick = () => {
    const parentPath = panel.currentPath.split('/').slice(0, -1).join('/') || '/';
    dispatch(navigateToPath({ panelId: panel.id, path: parentPath }));
  };

  const handleRightClick = (e: React.MouseEvent, file: FileInfo) => {
    e.preventDefault();
    
    // Select the right-clicked file if it's not already selected
    if (!panel.selectedFiles.includes(file.name)) {
      dispatch(selectFiles({ panelId: panel.id, fileNames: [file.name] }));
    }
    
    const selectedFileInfos = panel.files.filter(f => 
      panel.selectedFiles.includes(f.name) || f.name === file.name
    );
    
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      selectedFiles: selectedFileInfos,
    });
  };

  const handleDeleteFiles = async (filesToDelete: FileInfo[]) => {
    try {
      dispatch(setLoading({ panelId: panel.id, isLoading: true }));
      
      for (const file of filesToDelete) {
        await FileService.deleteItem(file.path);
      }
      
      // Refresh directory listing
      await loadDirectory(panel.currentPath);
      
      // Clear selection
      dispatch(selectFiles({ panelId: panel.id, fileNames: [] }));
    } catch (error) {
      console.error('Failed to delete files:', error);
      dispatch(setError({ 
        panelId: panel.id, 
        error: `Failed to delete files: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const handleRenameFile = async (file: FileInfo, newName: string) => {
    try {
      dispatch(setLoading({ panelId: panel.id, isLoading: true }));
      await FileService.renameItem(file.path, newName);
      await loadDirectory(panel.currentPath);
    } catch (error) {
      console.error('Failed to rename file:', error);
      dispatch(setError({ 
        panelId: panel.id, 
        error: `Failed to rename file: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const handleCreateFile = async (name: string) => {
    try {
      dispatch(setLoading({ panelId: panel.id, isLoading: true }));
      await FileService.createFile(panel.currentPath, name);
      await loadDirectory(panel.currentPath);
    } catch (error) {
      console.error('Failed to create file:', error);
      dispatch(setError({ 
        panelId: panel.id, 
        error: `Failed to create file: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const handleCreateFolder = async (name: string) => {
    try {
      dispatch(setLoading({ panelId: panel.id, isLoading: true }));
      await FileService.createDirectory(panel.currentPath, name);
      await loadDirectory(panel.currentPath);
    } catch (error) {
      console.error('Failed to create folder:', error);
      dispatch(setError({ 
        panelId: panel.id, 
        error: `Failed to create folder: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
  };

  const showConfirmDialog = (title: string, message: string, onConfirm: () => void, variant: 'default' | 'danger' = 'default') => {
    setConfirmDialog({
      isOpen: true,
      title,
      message,
      variant,
      onConfirm,
    });
  };

  const getContextMenuItems = (selectedFiles: FileInfo[]): ContextMenuItem[] => {
    const items: ContextMenuItem[] = [];
    const hasSelection = selectedFiles.length > 0;
    const isSingleFile = selectedFiles.length === 1;

    if (hasSelection) {
      items.push({
        id: 'rename',
        label: 'Rename',
        icon: '✏️',
        shortcut: 'F2',
        disabled: !isSingleFile,
        onClick: () => {
          if (isSingleFile) {
            const newName = prompt('Enter new name:', selectedFiles[0].name);
            if (newName && newName !== selectedFiles[0].name) {
              handleRenameFile(selectedFiles[0], newName);
            }
          }
        },
      });

      items.push({ separator: true } as ContextMenuItem);

      items.push({
        id: 'delete',
        label: selectedFiles.length === 1 ? 'Delete' : `Delete ${selectedFiles.length} items`,
        icon: '🗑️',
        shortcut: 'Del',
        onClick: () => {
          const message = selectedFiles.length === 1 
            ? `Are you sure you want to delete "${selectedFiles[0].name}"?`
            : `Are you sure you want to delete ${selectedFiles.length} selected items?`;
          
          showConfirmDialog(
            'Confirm Delete',
            message,
            () => handleDeleteFiles(selectedFiles),
            'danger'
          );
        },
      });

      items.push({ separator: true } as ContextMenuItem);
    }

    // Add "New" options
    items.push({
      id: 'new-file',
      label: 'New File',
      icon: '📄',
      onClick: () => {
        const name = prompt('Enter file name:');
        if (name) {
          handleCreateFile(name);
        }
      },
    });

    items.push({
      id: 'new-folder',
      label: 'New Folder',
      icon: '📁',
      onClick: () => {
        const name = prompt('Enter folder name:');
        if (name) {
          handleCreateFolder(name);
        }
      },
    });

    return items;
  };

  const formatFileSize = (size: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let unitIndex = 0;
    let fileSize = size;

    while (fileSize >= 1024 && unitIndex < units.length - 1) {
      fileSize /= 1024;
      unitIndex++;
    }

    return `${fileSize.toFixed(1)} ${units[unitIndex]}`;
  };

  const formatDate = (isoString: string): string => {
    return new Date(isoString).toLocaleDateString();
  };

  const sortedFiles = React.useMemo(() => {
    const files = [...panel.files];
    return files.sort((a, b) => {
      let comparison = 0;
      
      switch (panel.sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'size':
          comparison = a.size - b.size;
          break;
        case 'modified':
          comparison = new Date(a.modified).getTime() - new Date(b.modified).getTime();
          break;
        case 'type':
          comparison = (a.file_type === 'Directory' ? 'folder' : a.extension || '').localeCompare(
            b.file_type === 'Directory' ? 'folder' : b.extension || ''
          );
          break;
      }

      return panel.sortOrder === 'asc' ? comparison : -comparison;
    });
  }, [panel.files, panel.sortBy, panel.sortOrder]);

  if (panel.isLoading) {
    return (
      <div className="file-panel loading">
        <div className="panel-header">
          <span className="path">{panel.currentPath}</span>
        </div>
        <div className="loading-content">Loading...</div>
      </div>
    );
  }

  if (panel.error) {
    return (
      <div className="file-panel error">
        <div className="panel-header">
          <span className="path">{panel.currentPath}</span>
        </div>
        <div className="error-content">{panel.error}</div>
      </div>
    );
  }

  return (
    <div className="file-panel">
      <div className="panel-header">
        <button 
          className="back-button" 
          onClick={handleBackClick}
          disabled={panel.currentPath === '/'}
        >
          ←
        </button>
        <span className="path">{panel.currentPath}</span>
      </div>
      
      <div className="file-list">
        {sortedFiles.map((file) => (
          <div
            key={file.name}
            className={`file-item ${panel.selectedFiles.includes(file.name) ? 'selected' : ''}`}
            onClick={(e) => handleFileClick(file, e)}
            onDoubleClick={() => handleFileDoubleClick(file)}
            onContextMenu={(e) => handleRightClick(e, file)}
          >
            <span className="file-icon">
              {file.file_type === 'Directory' ? '📁' : '📄'}
            </span>
            <span className="file-name">{file.name}</span>
            <span className="file-size">
              {file.file_type === 'Directory' ? '' : formatFileSize(file.size)}
            </span>
            <span className="file-date">
              {formatDate(file.modified)}
            </span>
          </div>
        ))}
      </div>

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={getContextMenuItems(contextMenu.selectedFiles)}
          onClose={() => setContextMenu(null)}
          selectedFiles={contextMenu.selectedFiles}
        />
      )}

      <ConfirmDialog
        isOpen={confirmDialog.isOpen}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant={confirmDialog.variant}
        onConfirm={() => {
          confirmDialog.onConfirm();
          setConfirmDialog({ ...confirmDialog, isOpen: false });
        }}
        onCancel={() => setConfirmDialog({ ...confirmDialog, isOpen: false })}
      />
    </div>
  );
};

export default FilePanel;