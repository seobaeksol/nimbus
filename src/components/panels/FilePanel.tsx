import React, { useEffect, useState } from 'react';
import { Panel, selectFiles, navigateToPath, setFiles, setLoading, setError, startDrag, endDrag, setDragOperation, addProgressIndicator, updateProgressIndicator, removeProgressIndicator } from '../../store/slices/panelSlice';
import { useAppDispatch, useAppSelector } from '../../store';
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
  const { dragState, panels } = useAppSelector(state => state.panels);
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
  const [isDragOver, setIsDragOver] = useState(false);
  const [dragOverCounter, setDragOverCounter] = useState(0);

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

  // Drag and Drop handlers
  const handleDragStart = (e: React.DragEvent, file: FileInfo) => {
    // If the dragged file is not selected, select only it
    const filesToDrag = panel.selectedFiles.includes(file.name) 
      ? panel.selectedFiles 
      : [file.name];

    if (!panel.selectedFiles.includes(file.name)) {
      dispatch(selectFiles({ panelId: panel.id, fileNames: [file.name] }));
    }

    const operation = e.ctrlKey ? 'copy' : 'move';
    dispatch(startDrag({ panelId: panel.id, fileNames: filesToDrag, operation }));

    // Set drag data for compatibility
    e.dataTransfer.setData('text/plain', JSON.stringify({
      files: filesToDrag,
      sourcePanelId: panel.id,
      operation,
    }));

    e.dataTransfer.effectAllowed = e.ctrlKey ? 'copy' : 'move';
    
    // Create custom drag image
    const dragElement = e.currentTarget.cloneNode(true) as HTMLElement;
    dragElement.style.transform = 'rotate(5deg)';
    dragElement.style.opacity = '0.8';
    document.body.appendChild(dragElement);
    e.dataTransfer.setDragImage(dragElement, 10, 10);
    
    setTimeout(() => document.body.removeChild(dragElement), 0);
  };

  const handleDragEnd = () => {
    dispatch(endDrag());
    setIsDragOver(false);
    setDragOverCounter(0);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = e.ctrlKey ? 'copy' : 'move';
    
    // Update drag operation based on modifier keys
    if (dragState.isDragging && dragState.sourcePanelId !== panel.id) {
      const newOperation = e.ctrlKey ? 'copy' : 'move';
      if (dragState.dragOperation !== newOperation) {
        dispatch(setDragOperation(newOperation));
      }
    }
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOverCounter(prev => prev + 1);
    
    if (dragState.isDragging && dragState.sourcePanelId !== panel.id) {
      setIsDragOver(true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOverCounter(prev => prev - 1);
    
    if (dragOverCounter <= 1) {
      setIsDragOver(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    setDragOverCounter(0);

    if (!dragState.isDragging || dragState.sourcePanelId === panel.id) {
      return;
    }

    let currentProgressId: string | null = null;

    try {
      dispatch(setLoading({ panelId: panel.id, isLoading: true }));
      
      const sourcePanelId = dragState.sourcePanelId!;
      const filesToTransfer = dragState.draggedFiles;
      const operation = dragState.dragOperation || 'move';

      // Get source panel files to find the correct paths
      const sourcePanel = panels[sourcePanelId];
      const sourcePanelFiles = sourcePanel?.files || [];

      // Create progress indicator
      const progressId = `${operation}-${Date.now()}`;
      const totalFiles = filesToTransfer.length;
      currentProgressId = progressId;
      
      dispatch(addProgressIndicator({
        id: progressId,
        operation,
        fileName: totalFiles > 1 ? `${totalFiles} items` : filesToTransfer[0],
        progress: 0,
        totalFiles,
        currentFile: 0,
        isComplete: false
      }));

      for (let i = 0; i < filesToTransfer.length; i++) {
        const fileName = filesToTransfer[i];
        const sourceFile = sourcePanelFiles.find(f => f.name === fileName);
        if (!sourceFile) continue;

        // Update progress
        dispatch(updateProgressIndicator({
          id: progressId,
          updates: {
            fileName,
            currentFile: i + 1,
            progress: ((i + 1) / totalFiles) * 100
          }
        }));

        const srcPath = sourceFile.path;
        // Construct destination path properly, handling root directory case
        const dstPath = panel.currentPath === '/' 
          ? `/${fileName}` 
          : `${panel.currentPath}/${fileName}`;

        if (operation === 'copy') {
          await FileService.copyItem(srcPath, dstPath);
        } else {
          await FileService.moveItem(srcPath, dstPath);
        }
      }

      // Mark progress as complete
      dispatch(updateProgressIndicator({
        id: progressId,
        updates: {
          isComplete: true,
          progress: 100
        }
      }));

      // Auto-remove completed progress after 3 seconds
      setTimeout(() => {
        dispatch(removeProgressIndicator(progressId));
      }, 3000);

      // Refresh destination panel
      await loadDirectory(panel.currentPath);
      
      // If it was a move operation, also refresh the source panel
      if (operation === 'move' && sourcePanelId !== panel.id) {
        // Trigger refresh of source panel by dispatching navigation action
        const sourcePanel = panels[sourcePanelId];
        if (sourcePanel) {
          dispatch(navigateToPath({ panelId: sourcePanelId, path: sourcePanel.currentPath }));
        }
      }
      
      dispatch(endDrag());
      
    } catch (error) {
      console.error('Failed to transfer files:', error);
      
      // Update progress indicator with error
      if (currentProgressId) {
        dispatch(updateProgressIndicator({
          id: currentProgressId,
          updates: {
            error: error instanceof Error ? error.message : 'Unknown error',
            isComplete: false
          }
        }));
      }
      
      dispatch(setError({ 
        panelId: panel.id, 
        error: `Failed to transfer files: ${error instanceof Error ? error.message : 'Unknown error'}`
      }));
    }
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

  const isDragTarget = isDragOver && dragState.isDragging && dragState.sourcePanelId !== panel.id;
  const panelClassName = `file-panel ${isDragTarget ? 'drag-target' : ''} ${
    isDragTarget ? (dragState.dragOperation === 'copy' ? 'copy-mode' : 'move-mode') : ''
  }`;

  return (
    <div 
      className={panelClassName}
      onDragOver={handleDragOver}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
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
            className={`file-item ${panel.selectedFiles.includes(file.name) ? 'selected' : ''} ${
              dragState.isDragging && dragState.draggedFiles.includes(file.name) ? 'dragging' : ''
            }`}
            draggable
            onClick={(e) => handleFileClick(file, e)}
            onDoubleClick={() => handleFileDoubleClick(file)}
            onContextMenu={(e) => handleRightClick(e, file)}
            onDragStart={(e) => handleDragStart(e, file)}
            onDragEnd={handleDragEnd}
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