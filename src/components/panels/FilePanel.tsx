import React, { useEffect } from 'react';
import { Panel, selectFiles, navigateToPath, setFiles, setLoading, setError } from '../../store/slices/panelSlice';
import { useAppDispatch } from '../../store';
import { FileService } from '../../services/fileService';
import { FileInfo } from '../../types';
import './FilePanel.css';

interface FilePanelProps {
  panel: Panel;
}

const FilePanel: React.FC<FilePanelProps> = ({ panel }) => {
  const dispatch = useAppDispatch();

  useEffect(() => {
    loadDirectory(panel.currentPath);
  }, [panel.currentPath]);

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
    </div>
  );
};

export default FilePanel;