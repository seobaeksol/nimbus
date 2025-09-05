/**
 * Search Result Context Menu Hook
 * 
 * Provides context menu functionality specifically for search results,
 * with actions like open, copy path, reveal in folder, etc.
 */

import { useState, useCallback } from 'react';
import { SearchResult } from '@/types';
import { ContextMenuItem } from '../components/common/ContextMenu';

export interface SearchResultContextMenuState {
  isVisible: boolean;
  x: number;
  y: number;
  selectedResults: SearchResult[];
}

export interface UseSearchResultContextOptions {
  onOpenFile?: (result: SearchResult) => void;
  onRevealInFolder?: (result: SearchResult) => void;
  onCopyPath?: (result: SearchResult) => void;
  onCopyName?: (result: SearchResult) => void;
  onDeleteFile?: (results: SearchResult[]) => void;
  onViewProperties?: (result: SearchResult) => void;
}

export function useSearchResultContext(options: UseSearchResultContextOptions = {}) {
  const [contextMenu, setContextMenu] = useState<SearchResultContextMenuState>({
    isVisible: false,
    x: 0,
    y: 0,
    selectedResults: []
  });

  const {
    onOpenFile,
    onRevealInFolder,
    onCopyPath,
    onCopyName,
    onDeleteFile,
    onViewProperties
  } = options;

  const showContextMenu = useCallback((
    event: React.MouseEvent,
    selectedResults: SearchResult[]
  ) => {
    event.preventDefault();
    event.stopPropagation();

    setContextMenu({
      isVisible: true,
      x: event.clientX,
      y: event.clientY,
      selectedResults
    });
  }, []);

  const hideContextMenu = useCallback(() => {
    setContextMenu(prev => ({ ...prev, isVisible: false }));
  }, []);

  const handleOpenFile = useCallback(() => {
    if (contextMenu.selectedResults.length > 0 && onOpenFile) {
      onOpenFile(contextMenu.selectedResults[0]);
    }
    hideContextMenu();
  }, [contextMenu.selectedResults, onOpenFile, hideContextMenu]);

  const handleRevealInFolder = useCallback(() => {
    if (contextMenu.selectedResults.length > 0 && onRevealInFolder) {
      onRevealInFolder(contextMenu.selectedResults[0]);
    }
    hideContextMenu();
  }, [contextMenu.selectedResults, onRevealInFolder, hideContextMenu]);

  const handleCopyPath = useCallback(async () => {
    if (contextMenu.selectedResults.length > 0 && onCopyPath) {
      onCopyPath(contextMenu.selectedResults[0]);
      
      // Also copy to clipboard
      try {
        await navigator.clipboard.writeText(contextMenu.selectedResults[0].path);
      } catch (error) {
        console.warn('Failed to copy to clipboard:', error);
      }
    }
    hideContextMenu();
  }, [contextMenu.selectedResults, onCopyPath, hideContextMenu]);

  const handleCopyName = useCallback(async () => {
    if (contextMenu.selectedResults.length > 0 && onCopyName) {
      onCopyName(contextMenu.selectedResults[0]);
      
      // Also copy to clipboard
      try {
        await navigator.clipboard.writeText(contextMenu.selectedResults[0].name);
      } catch (error) {
        console.warn('Failed to copy to clipboard:', error);
      }
    }
    hideContextMenu();
  }, [contextMenu.selectedResults, onCopyName, hideContextMenu]);

  const handleDeleteFiles = useCallback(() => {
    if (contextMenu.selectedResults.length > 0 && onDeleteFile) {
      onDeleteFile(contextMenu.selectedResults);
    }
    hideContextMenu();
  }, [contextMenu.selectedResults, onDeleteFile, hideContextMenu]);

  const handleViewProperties = useCallback(() => {
    if (contextMenu.selectedResults.length > 0 && onViewProperties) {
      onViewProperties(contextMenu.selectedResults[0]);
    }
    hideContextMenu();
  }, [contextMenu.selectedResults, onViewProperties, hideContextMenu]);

  const getContextMenuItems = useCallback((): ContextMenuItem[] => {
    const selectedCount = contextMenu.selectedResults.length;
    const singleFile = selectedCount === 1;
    const multipleFiles = selectedCount > 1;

    const items: ContextMenuItem[] = [];

    if (singleFile) {
      items.push(
        {
          id: 'open',
          label: 'Open',
          icon: '📂',
          shortcut: 'Enter',
          onClick: handleOpenFile
        },
        {
          id: 'separator-1',
          label: '',
          separator: true
        }
      );
    }

    items.push(
      {
        id: 'copy-path',
        label: singleFile ? 'Copy Path' : 'Copy Paths',
        icon: '📋',
        shortcut: 'Ctrl+C',
        onClick: handleCopyPath
      },
      {
        id: 'copy-name',
        label: singleFile ? 'Copy Name' : 'Copy Names',
        icon: '📄',
        onClick: handleCopyName
      }
    );

    if (singleFile) {
      items.push(
        {
          id: 'reveal',
          label: 'Reveal in Folder',
          icon: '🗂️',
          shortcut: 'Ctrl+R',
          onClick: handleRevealInFolder
        }
      );
    }

    items.push(
      {
        id: 'separator-2',
        label: '',
        separator: true
      },
      {
        id: 'delete',
        label: singleFile ? 'Delete' : `Delete ${selectedCount} files`,
        icon: '🗑️',
        shortcut: 'Del',
        onClick: handleDeleteFiles
      }
    );

    if (singleFile) {
      items.push(
        {
          id: 'separator-3',
          label: '',
          separator: true
        },
        {
          id: 'properties',
          label: 'Properties',
          icon: 'ℹ️',
          shortcut: 'Alt+Enter',
          onClick: handleViewProperties
        }
      );
    }

    return items;
  }, [
    contextMenu.selectedResults,
    handleOpenFile,
    handleCopyPath,
    handleCopyName,
    handleRevealInFolder,
    handleDeleteFiles,
    handleViewProperties
  ]);

  // Keyboard shortcuts for context menu actions
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!contextMenu.isVisible || contextMenu.selectedResults.length === 0) return;

    switch (event.key) {
      case 'Enter':
        event.preventDefault();
        handleOpenFile();
        break;
      case 'c':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          handleCopyPath();
        }
        break;
      case 'r':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          handleRevealInFolder();
        }
        break;
      case 'Delete':
      case 'Backspace':
        event.preventDefault();
        handleDeleteFiles();
        break;
      case 'F10':
        if (event.altKey) {
          event.preventDefault();
          handleViewProperties();
        }
        break;
    }
  }, [
    contextMenu.isVisible,
    contextMenu.selectedResults,
    handleOpenFile,
    handleCopyPath,
    handleRevealInFolder,
    handleDeleteFiles,
    handleViewProperties
  ]);

  return {
    contextMenu,
    showContextMenu,
    hideContextMenu,
    getContextMenuItems,
    handleKeyDown,
    
    // Individual actions (can be called directly)
    actions: {
      openFile: handleOpenFile,
      revealInFolder: handleRevealInFolder,
      copyPath: handleCopyPath,
      copyName: handleCopyName,
      deleteFiles: handleDeleteFiles,
      viewProperties: handleViewProperties
    }
  };
}