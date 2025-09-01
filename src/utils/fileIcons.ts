//! Enhanced file icon system for better file type recognition
//!
//! Provides comprehensive file type detection and icon mapping
//! for a professional file manager experience.

export interface FileIconInfo {
  icon: string;
  color: string;
  category: 'document' | 'image' | 'audio' | 'video' | 'archive' | 'code' | 'executable' | 'folder' | 'system';
}

// File extension to icon mapping
const FILE_ICONS: Record<string, FileIconInfo> = {
  // Documents
  'txt': { icon: '📄', color: '#666666', category: 'document' },
  'md': { icon: '📝', color: '#000000', category: 'document' },
  'pdf': { icon: '📕', color: '#DC2626', category: 'document' },
  'doc': { icon: '📘', color: '#2563EB', category: 'document' },
  'docx': { icon: '📘', color: '#2563EB', category: 'document' },
  'xls': { icon: '📗', color: '#16A34A', category: 'document' },
  'xlsx': { icon: '📗', color: '#16A34A', category: 'document' },
  'ppt': { icon: '📙', color: '#EA580C', category: 'document' },
  'pptx': { icon: '📙', color: '#EA580C', category: 'document' },
  'rtf': { icon: '📄', color: '#666666', category: 'document' },
  
  // Images
  'jpg': { icon: '🖼️', color: '#10B981', category: 'image' },
  'jpeg': { icon: '🖼️', color: '#10B981', category: 'image' },
  'png': { icon: '🖼️', color: '#10B981', category: 'image' },
  'gif': { icon: '🎭', color: '#F59E0B', category: 'image' },
  'bmp': { icon: '🖼️', color: '#10B981', category: 'image' },
  'svg': { icon: '🎨', color: '#8B5CF6', category: 'image' },
  'webp': { icon: '🖼️', color: '#10B981', category: 'image' },
  'ico': { icon: '🖼️', color: '#10B981', category: 'image' },
  'tiff': { icon: '🖼️', color: '#10B981', category: 'image' },
  'tif': { icon: '🖼️', color: '#10B981', category: 'image' },
  
  // Audio
  'mp3': { icon: '🎵', color: '#EC4899', category: 'audio' },
  'wav': { icon: '🎵', color: '#EC4899', category: 'audio' },
  'flac': { icon: '🎵', color: '#EC4899', category: 'audio' },
  'aac': { icon: '🎵', color: '#EC4899', category: 'audio' },
  'ogg': { icon: '🎵', color: '#EC4899', category: 'audio' },
  'wma': { icon: '🎵', color: '#EC4899', category: 'audio' },
  'm4a': { icon: '🎵', color: '#EC4899', category: 'audio' },
  
  // Video
  'mp4': { icon: '🎬', color: '#DC2626', category: 'video' },
  'avi': { icon: '🎬', color: '#DC2626', category: 'video' },
  'mov': { icon: '🎬', color: '#DC2626', category: 'video' },
  'wmv': { icon: '🎬', color: '#DC2626', category: 'video' },
  'flv': { icon: '🎬', color: '#DC2626', category: 'video' },
  'webm': { icon: '🎬', color: '#DC2626', category: 'video' },
  'mkv': { icon: '🎬', color: '#DC2626', category: 'video' },
  'm4v': { icon: '🎬', color: '#DC2626', category: 'video' },
  '3gp': { icon: '🎬', color: '#DC2626', category: 'video' },
  
  // Archives
  'zip': { icon: '📦', color: '#F59E0B', category: 'archive' },
  'rar': { icon: '📦', color: '#F59E0B', category: 'archive' },
  '7z': { icon: '📦', color: '#F59E0B', category: 'archive' },
  'tar': { icon: '📦', color: '#F59E0B', category: 'archive' },
  'gz': { icon: '📦', color: '#F59E0B', category: 'archive' },
  'bz2': { icon: '📦', color: '#F59E0B', category: 'archive' },
  'xz': { icon: '📦', color: '#F59E0B', category: 'archive' },
  'lz': { icon: '📦', color: '#F59E0B', category: 'archive' },
  
  // Code files
  'js': { icon: '💻', color: '#F7DF1E', category: 'code' },
  'ts': { icon: '💻', color: '#3178C6', category: 'code' },
  'jsx': { icon: '⚛️', color: '#61DAFB', category: 'code' },
  'tsx': { icon: '⚛️', color: '#61DAFB', category: 'code' },
  'html': { icon: '🌐', color: '#E34F26', category: 'code' },
  'css': { icon: '🎨', color: '#1572B6', category: 'code' },
  'scss': { icon: '🎨', color: '#CF649A', category: 'code' },
  'sass': { icon: '🎨', color: '#CF649A', category: 'code' },
  'less': { icon: '🎨', color: '#1D365D', category: 'code' },
  'py': { icon: '🐍', color: '#3776AB', category: 'code' },
  'java': { icon: '☕', color: '#ED8B00', category: 'code' },
  'c': { icon: '🔧', color: '#A8B9CC', category: 'code' },
  'cpp': { icon: '🔧', color: '#00599C', category: 'code' },
  'h': { icon: '🔧', color: '#A8B9CC', category: 'code' },
  'cs': { icon: '🔷', color: '#239120', category: 'code' },
  'php': { icon: '🐘', color: '#777BB4', category: 'code' },
  'rb': { icon: '💎', color: '#CC342D', category: 'code' },
  'go': { icon: '🐹', color: '#00ADD8', category: 'code' },
  'rs': { icon: '🦀', color: '#CE422B', category: 'code' },
  'swift': { icon: '🦉', color: '#FA7343', category: 'code' },
  'kt': { icon: '🟣', color: '#7F52FF', category: 'code' },
  'dart': { icon: '🎯', color: '#0175C2', category: 'code' },
  'vue': { icon: '💚', color: '#4FC08D', category: 'code' },
  'json': { icon: '📋', color: '#000000', category: 'code' },
  'xml': { icon: '📋', color: '#FF6600', category: 'code' },
  'yaml': { icon: '📋', color: '#CB171E', category: 'code' },
  'yml': { icon: '📋', color: '#CB171E', category: 'code' },
  'toml': { icon: '📋', color: '#9C4121', category: 'code' },
  
  // Executables
  'exe': { icon: '⚙️', color: '#666666', category: 'executable' },
  'msi': { icon: '⚙️', color: '#666666', category: 'executable' },
  'deb': { icon: '📦', color: '#A80030', category: 'executable' },
  'rpm': { icon: '📦', color: '#EE0000', category: 'executable' },
  'dmg': { icon: '💿', color: '#666666', category: 'executable' },
  'app': { icon: '📱', color: '#007AFF', category: 'executable' },
  'AppImage': { icon: '🐧', color: '#FCC624', category: 'executable' },
  'snap': { icon: '📦', color: '#E95420', category: 'executable' },
  'flatpak': { icon: '📦', color: '#4A90E2', category: 'executable' },
  
  // System files
  'log': { icon: '📜', color: '#666666', category: 'system' },
  'cfg': { icon: '⚙️', color: '#666666', category: 'system' },
  'conf': { icon: '⚙️', color: '#666666', category: 'system' },
  'ini': { icon: '⚙️', color: '#666666', category: 'system' },
  'bat': { icon: '🖥️', color: '#4A90E2', category: 'system' },
  'sh': { icon: '🖥️', color: '#4EAA25', category: 'system' },
  'ps1': { icon: '🖥️', color: '#012456', category: 'system' },
  'command': { icon: '🖥️', color: '#666666', category: 'system' },
};

// Fallback icons for unknown extensions
const FALLBACK_ICONS: Record<string, FileIconInfo> = {
  file: { icon: '📄', color: '#666666', category: 'document' },
  folder: { icon: '📁', color: '#0078d4', category: 'folder' },
  folderOpen: { icon: '📂', color: '#0078d4', category: 'folder' },
};

/**
 * Get file icon information based on file name and type
 */
export function getFileIcon(fileName: string, fileType: 'Directory' | 'File' | 'Symlink', isOpen?: boolean): FileIconInfo {
  if (fileType === 'Directory') {
    return isOpen ? FALLBACK_ICONS.folderOpen : FALLBACK_ICONS.folder;
  }

  if (fileType === 'Symlink') {
    // For symlinks, show a link icon
    return { icon: '🔗', color: '#8B5CF6', category: 'system' };
  }

  // Extract extension from file name
  const extension = getFileExtension(fileName).toLowerCase();
  
  // Look up in file icons map
  const iconInfo = FILE_ICONS[extension];
  if (iconInfo) {
    return iconInfo;
  }

  // Return fallback for unknown file types
  return FALLBACK_ICONS.file;
}

/**
 * Extract file extension from filename
 */
function getFileExtension(fileName: string): string {
  const lastDotIndex = fileName.lastIndexOf('.');
  if (lastDotIndex === -1 || lastDotIndex === 0) {
    return '';
  }
  return fileName.substring(lastDotIndex + 1);
}

/**
 * Get file category for grouping and filtering
 */
export function getFileCategory(fileName: string, fileType: 'Directory' | 'File' | 'Symlink'): string {
  const iconInfo = getFileIcon(fileName, fileType);
  return iconInfo.category;
}

/**
 * Check if file is an archive that can be browsed
 */
export function isArchiveFile(fileName: string): boolean {
  const extension = getFileExtension(fileName).toLowerCase();
  return ['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz'].includes(extension);
}

/**
 * Check if file is an image that can be previewed
 */
export function isImageFile(fileName: string): boolean {
  const extension = getFileExtension(fileName).toLowerCase();
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'ico', 'tiff', 'tif'].includes(extension);
}

/**
 * Check if file is a text file that can be viewed
 */
export function isTextFile(fileName: string): boolean {
  const extension = getFileExtension(fileName).toLowerCase();
  const textExtensions = [
    'txt', 'md', 'json', 'xml', 'yaml', 'yml', 'toml', 'ini', 'cfg', 'conf',
    'js', 'ts', 'jsx', 'tsx', 'html', 'css', 'scss', 'sass', 'less',
    'py', 'java', 'c', 'cpp', 'h', 'cs', 'php', 'rb', 'go', 'rs',
    'swift', 'kt', 'dart', 'vue', 'sh', 'bat', 'ps1', 'log'
  ];
  return textExtensions.includes(extension);
}

/**
 * Get MIME type estimate based on file extension
 */
export function getMimeTypeEstimate(fileName: string): string {
  const extension = getFileExtension(fileName).toLowerCase();
  
  const mimeTypes: Record<string, string> = {
    // Text
    'txt': 'text/plain',
    'md': 'text/markdown',
    'html': 'text/html',
    'css': 'text/css',
    'js': 'application/javascript',
    'json': 'application/json',
    'xml': 'application/xml',
    
    // Images
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'png': 'image/png',
    'gif': 'image/gif',
    'svg': 'image/svg+xml',
    'webp': 'image/webp',
    'bmp': 'image/bmp',
    'ico': 'image/x-icon',
    
    // Audio
    'mp3': 'audio/mpeg',
    'wav': 'audio/wav',
    'flac': 'audio/flac',
    'ogg': 'audio/ogg',
    
    // Video
    'mp4': 'video/mp4',
    'avi': 'video/x-msvideo',
    'mov': 'video/quicktime',
    'webm': 'video/webm',
    'mkv': 'video/x-matroska',
    
    // Archives
    'zip': 'application/zip',
    '7z': 'application/x-7z-compressed',
    'tar': 'application/x-tar',
    'gz': 'application/gzip',
    'rar': 'application/vnd.rar',
    
    // Documents
    'pdf': 'application/pdf',
    'doc': 'application/msword',
    'docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'xls': 'application/vnd.ms-excel',
    'xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    'ppt': 'application/vnd.ms-powerpoint',
    'pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
  };
  
  return mimeTypes[extension] || 'application/octet-stream';
}

/**
 * Format file size in human-readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

/**
 * Format file date in human-readable format
 */
export function formatFileDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  if (diffDays === 0) {
    return 'Today ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (diffDays === 1) {
    return 'Yesterday';
  } else if (diffDays < 7) {
    return `${diffDays} days ago`;
  } else {
    return date.toLocaleDateString();
  }
}