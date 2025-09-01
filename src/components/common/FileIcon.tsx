import React from 'react';
import { getFileIcon } from '../../utils/fileIcons';
import './FileIcon.css';

interface FileIconProps {
  fileName: string;
  fileType: 'Directory' | 'File' | 'Symlink';
  size?: 'small' | 'medium' | 'large';
  isOpen?: boolean; // For folders
  showExtension?: boolean;
  className?: string;
}

/**
 * Enhanced file icon component with better file type recognition
 */
const FileIcon: React.FC<FileIconProps> = ({
  fileName,
  fileType,
  size = 'medium',
  isOpen = false,
  showExtension = false,
  className = ''
}) => {
  const iconInfo = getFileIcon(fileName, fileType, isOpen);
  
  const getExtension = () => {
    if (fileType === 'Directory' || !showExtension) return null;
    const lastDotIndex = fileName.lastIndexOf('.');
    if (lastDotIndex === -1 || lastDotIndex === 0) return null;
    return fileName.substring(lastDotIndex + 1).toUpperCase();
  };

  const extension = getExtension();

  return (
    <div 
      className={`file-icon file-icon--${size} file-icon--${iconInfo.category} ${className}`}
      style={{ color: iconInfo.color }}
      title={`${fileType}: ${fileName}`}
    >
      <span className="file-icon__symbol" aria-label={iconInfo.category}>
        {iconInfo.icon}
      </span>
      {extension && (
        <span className="file-icon__extension">
          {extension}
        </span>
      )}
    </div>
  );
};

export default FileIcon;