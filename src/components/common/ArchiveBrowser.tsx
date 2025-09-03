import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import FileIcon from './FileIcon';
import { formatFileSize, formatFileDate, getArchiveFormatInfo } from '../../utils/fileIcons';
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

interface CompressionOptions {
  level: number;
  method: string;
  password?: string;
  solid: boolean;
}

interface ArchiveBrowserProps {
  archivePath: string;
  onClose?: () => void;
  onExtract?: (entries: string[], destination: string) => void;
  onPreview?: (entryPath: string) => void;
  onCreateArchive?: (files: string[], archivePath: string, format: string, options: CompressionOptions) => void;
  mode?: 'browse' | 'create';
  sourceFiles?: string[];
}

const ArchiveBrowser: React.FC<ArchiveBrowserProps> = ({
  archivePath,
  onClose,
  onExtract,
  onPreview,
  onCreateArchive,
  mode = 'browse',
  sourceFiles = []
}) => {
  const [entries, setEntries] = useState<ArchiveEntry[]>([]);
  const [archiveInfo, setArchiveInfo] = useState<ArchiveInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedEntries, setSelectedEntries] = useState<Set<string>>(new Set());
  const [currentPath, setCurrentPath] = useState('');
  const [sortBy, setSortBy] = useState<'name' | 'size' | 'date'>('name');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  
  // Archive creation state
  const [selectedFormat, setSelectedFormat] = useState<'zip' | 'tar' | 'tar.gz' | 'tar.bz2' | '7z'>('zip');
  const [compressionLevel, setCompressionLevel] = useState(6);
  const [compressionMethod, setCompressionMethod] = useState('deflate');
  const [usePassword, setUsePassword] = useState(false);
  const [password, setPassword] = useState('');
  const [useSolidCompression, setUseSolidCompression] = useState(false);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [creationProgress, setCreationProgress] = useState({ current: 0, total: 0, currentFile: '', speed: 0, eta: 0 });

  useEffect(() => {
    if (mode === 'browse') {
      loadArchiveContents();
      loadArchiveInfo();
    } else {
      setLoading(false);
    }
  }, [archivePath, currentPath, mode]);

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

  const archiveFileName = archivePath.split('/').pop() || '';
  const formatInfo = getArchiveFormatInfo(archiveFileName);

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

  const handleCreateArchive = async () => {
    if (!onCreateArchive || sourceFiles.length === 0) return;
    
    setIsCreating(true);
    try {
      const options: CompressionOptions = {
        level: compressionLevel,
        method: compressionMethod,
        password: usePassword ? password : undefined,
        solid: useSolidCompression && selectedFormat === '7z'
      };
      
      await onCreateArchive(sourceFiles, archivePath, selectedFormat, options);
      setShowCreateDialog(false);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsCreating(false);
    }
  };

  const getCompressionMethods = () => {
    switch (selectedFormat) {
      case 'zip':
        return ['store', 'deflate', 'deflate64', 'bzip2'];
      case 'tar':
        return ['store'];
      case 'tar.gz':
        return ['gzip'];
      case 'tar.bz2':
        return ['bzip2'];
      case '7z':
        return ['lzma', 'lzma2', 'ppmd', 'bzip2', 'deflate'];
      default:
        return ['deflate'];
    }
  };

  const getMaxCompressionLevel = () => {
    switch (selectedFormat) {
      case '7z':
        return 9;
      case 'zip':
        return 9;
      case 'tar.gz':
        return 9;
      case 'tar.bz2':
        return 9;
      default:
        return 6;
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
          <FileIcon fileName={archiveFileName} fileType="File" size="large" />
          <div>
            <h3>{archiveFileName}</h3>
            <div className="archive-format-info">
              <span className="format-badge" title={formatInfo.description}>
                {formatInfo.format}
              </span>
              <span className="format-name">{formatInfo.displayName}</span>
              {formatInfo.supportsEncryption && (
                <span className="feature-badge encryption" title="Supports encryption">🔐</span>
              )}
              {formatInfo.supportsCompression && (
                <span className="feature-badge compression" title="Supports compression">📦</span>
              )}
            </div>
            {archiveInfo && (
              <div className="archive-stats">
                <div className="stats-primary">
                  {archiveInfo.total_files} files • {archiveInfo.total_directories} directories • {formatFileSize(archiveInfo.total_size)}
                </div>
                <div className="stats-secondary">
                  {archiveInfo.compression_ratio && (
                    <>
                      <div className="compression-info">
                        <span className="compression-ratio">
                          {Math.round(archiveInfo.compression_ratio)}% compression ratio
                        </span>
                        <span className="size-saved">
                          ({formatFileSize(archiveInfo.total_size - archiveInfo.compressed_size)} saved)
                        </span>
                      </div>
                      <div className="compression-efficiency">
                        <div className="efficiency-bar">
                          <div 
                            className="efficiency-fill"
                            style={{ width: `${Math.min(archiveInfo.compression_ratio, 100)}%` }}
                          />
                        </div>
                      </div>
                    </>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
        
        <div className="archive-actions">
          {mode === 'browse' && (
            <>
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
            </>
          )}
          {mode === 'create' && onCreateArchive && (
            <button 
              className="action-button primary" 
              onClick={() => setShowCreateDialog(true)}
              disabled={sourceFiles.length === 0}
            >
              Create Archive ({sourceFiles.length} files)
            </button>
          )}
          {onClose && (
            <button className="action-button secondary" onClick={onClose}>
              {mode === 'create' ? 'Cancel' : 'Close'}
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
                            ({formatFileSize(entry.compressed_size)} • {Math.round((1 - entry.compressed_size / entry.size) * 100)}%)
                          </span>
                        )}
                      </span>
                      {entry.modified && (
                        <span className="entry-date">
                          {formatFileDate(entry.modified)}
                        </span>
                      )}
                      {entry.crc32 !== null && (
                        <span className="entry-crc" title={`CRC32: ${entry.crc32.toString(16).toUpperCase()}`}>
                          ✓ CRC
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

      {mode === 'browse' && sortedEntries.length === 0 && (
        <div className="empty-state">
          <span className="empty-icon">📂</span>
          <p>This directory is empty</p>
        </div>
      )}

      {mode === 'create' && (
        <div className="create-mode-content">
          <div className="source-files-list">
            <h4>Files to compress ({sourceFiles.length}):</h4>
            <div className="file-list">
              {sourceFiles.slice(0, 10).map((file, index) => (
                <div key={index} className="source-file-item">
                  <FileIcon fileName={file.split('/').pop() || ''} fileType="File" size="small" />
                  <span className="file-name">{file}</span>
                </div>
              ))}
              {sourceFiles.length > 10 && (
                <div className="more-files">
                  ... and {sourceFiles.length - 10} more files
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Compression Settings Dialog */}
      {showCreateDialog && (
        <div className="modal-overlay" onClick={() => setShowCreateDialog(false)}>
          <div className="compression-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h3>Archive Compression Settings</h3>
              <button className="close-dialog" onClick={() => setShowCreateDialog(false)}>✕</button>
            </div>
            
            <div className="dialog-content">
              {/* Format Selection */}
              <div className="setting-group">
                <label>Archive Format:</label>
                <select 
                  value={selectedFormat} 
                  onChange={(e) => {
                    setSelectedFormat(e.target.value as any);
                    setCompressionMethod(getCompressionMethods()[0]);
                  }}
                >
                  <option value="zip">ZIP - Universal compatibility</option>
                  <option value="tar">TAR - No compression, Unix standard</option>
                  <option value="tar.gz">TAR.GZ - Good compression, widely supported</option>
                  <option value="tar.bz2">TAR.BZ2 - Better compression, slower</option>
                  <option value="7z">7Z - Best compression ratio</option>
                </select>
              </div>

              {/* Compression Level */}
              <div className="setting-group">
                <label>Compression Level: {compressionLevel}</label>
                <input
                  type="range"
                  min="0"
                  max={getMaxCompressionLevel()}
                  value={compressionLevel}
                  onChange={(e) => setCompressionLevel(parseInt(e.target.value))}
                  className="compression-slider"
                />
                <div className="level-labels">
                  <span>Store (0)</span>
                  <span>Fast</span>
                  <span>Best ({getMaxCompressionLevel()})</span>
                </div>
              </div>

              {/* Compression Method */}
              {getCompressionMethods().length > 1 && (
                <div className="setting-group">
                  <label>Compression Method:</label>
                  <select 
                    value={compressionMethod} 
                    onChange={(e) => setCompressionMethod(e.target.value)}
                  >
                    {getCompressionMethods().map(method => (
                      <option key={method} value={method}>{method.toUpperCase()}</option>
                    ))}
                  </select>
                </div>
              )}

              {/* Password Protection */}
              <div className="setting-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={usePassword}
                    onChange={(e) => setUsePassword(e.target.checked)}
                  />
                  Password Protection
                </label>
                {usePassword && (
                  <input
                    type="password"
                    placeholder="Enter password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="password-input"
                  />
                )}
              </div>

              {/* Solid Compression (7z only) */}
              {selectedFormat === '7z' && (
                <div className="setting-group">
                  <label className="checkbox-label">
                    <input
                      type="checkbox"
                      checked={useSolidCompression}
                      onChange={(e) => setUseSolidCompression(e.target.checked)}
                    />
                    Solid Compression (Better ratio, slower)
                  </label>
                </div>
              )}
            </div>
            
            <div className="dialog-actions">
              <button 
                className="action-button secondary" 
                onClick={() => setShowCreateDialog(false)}
                disabled={isCreating}
              >
                Cancel
              </button>
              <button 
                className="action-button primary" 
                onClick={handleCreateArchive}
                disabled={isCreating || (usePassword && !password)}
              >
                {isCreating ? 'Creating...' : 'Create Archive'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Progress Tracking Overlay */}
      {isCreating && (
        <div className="modal-overlay">
          <div className="progress-dialog">
            <div className="progress-header">
              <h3>Creating Archive</h3>
              <div className="progress-stats">
                {creationProgress.current > 0 && (
                  <>
                    <span>{creationProgress.current} of {creationProgress.total} files</span>
                    {creationProgress.speed > 0 && (
                      <span className="progress-speed">
                        {(creationProgress.speed / (1024 * 1024)).toFixed(1)} MB/s
                      </span>
                    )}
                    {creationProgress.eta > 0 && (
                      <span className="progress-eta">
                        {Math.ceil(creationProgress.eta)}s remaining
                      </span>
                    )}
                  </>
                )}
              </div>
            </div>
            
            <div className="progress-content">
              <div className="progress-bar-container">
                <div 
                  className="progress-bar"
                  style={{ 
                    width: creationProgress.total > 0 
                      ? `${(creationProgress.current / creationProgress.total) * 100}%` 
                      : '0%' 
                  }}
                />
              </div>
              
              {creationProgress.currentFile && (
                <div className="current-file">
                  <span className="file-label">Current:</span>
                  <span className="file-path">{creationProgress.currentFile}</span>
                </div>
              )}
              
              <div className="compression-details">
                <div className="detail-item">
                  <span className="label">Format:</span>
                  <span className="value">{selectedFormat.toUpperCase()}</span>
                </div>
                <div className="detail-item">
                  <span className="label">Method:</span>
                  <span className="value">{compressionMethod.toUpperCase()}</span>
                </div>
                <div className="detail-item">
                  <span className="label">Level:</span>
                  <span className="value">{compressionLevel}</span>
                </div>
                {usePassword && (
                  <div className="detail-item">
                    <span className="label">Password:</span>
                    <span className="value">Protected 🔒</span>
                  </div>
                )}
              </div>
            </div>
            
            <div className="progress-actions">
              <button 
                className="action-button secondary"
                onClick={() => {
                  // TODO: Cancel operation
                  setIsCreating(false);
                  setShowCreateDialog(false);
                }}
                disabled={false}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ArchiveBrowser;