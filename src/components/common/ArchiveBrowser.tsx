import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import FileIcon from './FileIcon';
import { formatFileSize, formatFileDate } from '../../utils/fileIcons';
import './ArchiveBrowser.css';

interface ArchiveEntry {
  path: string;
  name: string;
  size: number;
  compressed_size: number;
  modified: string | null;
  is_directory: boolean;
  compression_method: string | null;
  crc32: number | null;
  is_encrypted: boolean;
}

interface ArchiveInfo {
  format: string;
  total_entries: number;
  total_files: number;
  total_directories: number;
  total_size: number;
  compressed_size: number;
  compression_ratio: number | null;
}

interface ArchiveBrowserProps {
  archivePath: string;
  onClose?: () => void;
  onExtract?: (entries: string[], destination: string) => void;
  onPreview?: (entryPath: string) => void;
}

const ArchiveBrowser: React.FC<ArchiveBrowserProps> = ({
  archivePath,
  onClose,
  onExtract,
  onPreview
}) => {
  const [entries, setEntries] = useState<ArchiveEntry[]>([]);
  const [archiveInfo, setArchiveInfo] = useState<ArchiveInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedEntries, setSelectedEntries] = useState<Set<string>>(new Set());
  const [currentPath, setCurrentPath] = useState('');
  const [sortBy, setSortBy] = useState<'name' | 'size' | 'date'>('name');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');

  useEffect(() => {
    loadArchiveContents();
    loadArchiveInfo();
  }, [archivePath, currentPath]);

  const loadArchiveContents = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ArchiveEntry[]>('list_archive_contents', {
        archivePath,
        internalPath: currentPath || null
      });
      setEntries(result);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const loadArchiveInfo = async () => {
    try {
      const info = await invoke<ArchiveInfo>('get_archive_info', {
        archivePath
      });
      setArchiveInfo(info);
    } catch (err) {
      console.warn('Failed to load archive info:', err);
    }
  };

  const handleEntryClick = (entry: ArchiveEntry, isDoubleClick: boolean = false) => {
    if (entry.is_directory && isDoubleClick) {
      // Navigate into directory
      const newPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
      setCurrentPath(newPath);
    } else if (!entry.is_directory && isDoubleClick && onPreview) {
      // Preview file
      const fullPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
      onPreview(fullPath);
    } else {
      // Toggle selection
      const entryPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
      const newSelected = new Set(selectedEntries);
      if (newSelected.has(entryPath)) {
        newSelected.delete(entryPath);
      } else {
        newSelected.add(entryPath);
      }
      setSelectedEntries(newSelected);
    }
  };

  const handleBackClick = () => {
    const pathParts = currentPath.split('/');
    pathParts.pop();
    setCurrentPath(pathParts.join('/'));
  };

  const handleExtractSelected = () => {
    if (selectedEntries.size > 0 && onExtract) {
      onExtract(Array.from(selectedEntries), '');
    }
  };

  const handleExtractAll = () => {
    if (onExtract) {
      onExtract([], ''); // Empty array means extract all
    }
  };

  const sortEntries = (entries: ArchiveEntry[]): ArchiveEntry[] => {
    return [...entries].sort((a, b) => {
      // Directories first
      if (a.is_directory && !b.is_directory) return -1;
      if (!a.is_directory && b.is_directory) return 1;

      let comparison = 0;
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'size':
          comparison = a.size - b.size;
          break;
        case 'date':
          const aDate = a.modified ? new Date(a.modified).getTime() : 0;
          const bDate = b.modified ? new Date(b.modified).getTime() : 0;
          comparison = aDate - bDate;
          break;
      }

      return sortOrder === 'asc' ? comparison : -comparison;
    });
  };

  const filteredEntries = currentPath
    ? entries.filter(entry => entry.path.startsWith(currentPath + '/') || entry.path === currentPath)
    : entries;

  const sortedEntries = sortEntries(filteredEntries);

  if (loading) {
    return (
      <div className="archive-browser loading">
        <div className="loading-spinner">Loading archive contents...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="archive-browser error">
        <div className="error-message">
          <span className="error-icon">⚠️</span>
          Failed to load archive: {error}
        </div>
        {onClose && (
          <button className="close-button" onClick={onClose}>
            Close
          </button>
        )}
      </div>
    );
  }

  return (
    <div className="archive-browser">
      <div className="archive-header">
        <div className="archive-title">
          <FileIcon fileName={archivePath.split('/').pop() || ''} fileType="File" size="large" />
          <div>
            <h3>{archivePath.split('/').pop()}</h3>
            {archiveInfo && (
              <div className="archive-stats">
                {archiveInfo.format} • {archiveInfo.total_files} files • {formatFileSize(archiveInfo.total_size)}
                {archiveInfo.compression_ratio && (
                  <span> • {Math.round(archiveInfo.compression_ratio)}% compressed</span>
                )}
              </div>
            )}
          </div>
        </div>
        
        <div className="archive-actions">
          {selectedEntries.size > 0 && onExtract && (
            <button className="action-button primary" onClick={handleExtractSelected}>
              Extract Selected ({selectedEntries.size})
            </button>
          )}
          {onExtract && (
            <button className="action-button" onClick={handleExtractAll}>
              Extract All
            </button>
          )}
          {onClose && (
            <button className="action-button secondary" onClick={onClose}>
              Close
            </button>
          )}
        </div>
      </div>

      <div className="archive-navigation">
        {currentPath && (
          <button className="nav-button back" onClick={handleBackClick}>
            ← Back
          </button>
        )}
        <div className="current-path">
          {currentPath ? `/${currentPath}` : '/'}
        </div>
        
        <div className="sort-controls">
          <select 
            value={sortBy} 
            onChange={(e) => setSortBy(e.target.value as 'name' | 'size' | 'date')}
          >
            <option value="name">Name</option>
            <option value="size">Size</option>
            <option value="date">Date</option>
          </select>
          <button 
            className="sort-order-button"
            onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')}
          >
            {sortOrder === 'asc' ? '↑' : '↓'}
          </button>
        </div>
      </div>

      <div className="archive-entries">
        {sortedEntries.map((entry) => {
          const entryPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
          const isSelected = selectedEntries.has(entryPath);
          
          return (
            <div
              key={entry.path}
              className={`archive-entry ${isSelected ? 'selected' : ''} ${entry.is_encrypted ? 'encrypted' : ''}`}
              onClick={() => handleEntryClick(entry)}
              onDoubleClick={() => handleEntryClick(entry, true)}
            >
              <div className="entry-icon">
                <FileIcon 
                  fileName={entry.name} 
                  fileType={entry.is_directory ? 'Directory' : 'File'} 
                  size="medium"
                />
                {entry.is_encrypted && (
                  <span className="encryption-badge" title="Encrypted">🔒</span>
                )}
              </div>
              
              <div className="entry-info">
                <div className="entry-name" title={entry.name}>
                  {entry.name}
                </div>
                <div className="entry-details">
                  {!entry.is_directory && (
                    <>
                      <span className="entry-size">
                        {formatFileSize(entry.size)}
                        {entry.compressed_size !== entry.size && (
                          <span className="compressed-size">
                            ({formatFileSize(entry.compressed_size)} compressed)
                          </span>
                        )}
                      </span>
                      {entry.modified && (
                        <span className="entry-date">
                          {formatFileDate(entry.modified)}
                        </span>
                      )}
                    </>
                  )}
                  {entry.compression_method && (
                    <span className="compression-method" title="Compression method">
                      {entry.compression_method}
                    </span>
                  )}
                </div>
              </div>
              
              <div className="entry-actions">
                {!entry.is_directory && onPreview && (
                  <button 
                    className="action-button small"
                    onClick={(e) => {
                      e.stopPropagation();
                      onPreview(entryPath);
                    }}
                    title="Preview file"
                  >
                    👁️
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {sortedEntries.length === 0 && (
        <div className="empty-state">
          <span className="empty-icon">📂</span>
          <p>This directory is empty</p>
        </div>
      )}
    </div>
  );
};

export default ArchiveBrowser;