import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders, createMockFileInfo, createMockDirectoryInfo } from '../../../test/testUtils';
import FilePanel from '../FilePanel';
import { CommandExecutor } from '../../../services/commandExecutor';
import type { Panel } from '../../../store/slices/panelSlice';

// Mock CommandExecutor
vi.mock('../../../services/commandExecutor', () => ({
  CommandExecutor: {
    loadDirectory: vi.fn(),
    deleteFiles: vi.fn(),
    renameFile: vi.fn(),
    copyFiles: vi.fn(),
    cutFiles: vi.fn(),
    pasteFiles: vi.fn(),
    navigateToDirectory: vi.fn(),
    navigateToParent: vi.fn(),
    navigateToPath: vi.fn(),
    handleError: vi.fn(),
    startDrag: vi.fn(),
    endDrag: vi.fn(),
    updateDragOperation: vi.fn(),
    handleDrop: vi.fn(),
  },
}));

const mockCommandExecutor = vi.mocked(CommandExecutor);

describe('FilePanel', () => {
  const user = userEvent.setup();
  let mockPanel: Panel;

  beforeEach(() => {
    mockPanel = {
      id: 'test-panel',
      currentPath: '/test/path',
      files: [
        createMockFileInfo({ name: 'file1.txt', path: '/test/path/file1.txt' }),
        createMockFileInfo({ name: 'file2.txt', path: '/test/path/file2.txt' }),
        createMockDirectoryInfo({ name: 'folder1', path: '/test/path/folder1' }),
      ],
      selectedFiles: [],
      isLoading: false,
      error: null,
      sortBy: 'name',
      sortOrder: 'asc',
      viewMode: 'list',
    };
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Component Rendering', () => {
    it('should render file list correctly', () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      expect(screen.getByText('file1.txt')).toBeInTheDocument();
      expect(screen.getByText('file2.txt')).toBeInTheDocument();
      expect(screen.getByText('folder1')).toBeInTheDocument();
    });

    it('should show loading state', () => {
      const loadingPanel = { ...mockPanel, isLoading: true };
      renderWithProviders(<FilePanel panel={loadingPanel} isActive={true} />);

      expect(screen.getByText('Loading...')).toBeInTheDocument();
    });

    it('should render address bar', () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const addressBar = screen.getByDisplayValue('/test/path');
      expect(addressBar).toBeInTheDocument();
    });
  });

  describe('Directory Loading', () => {
    it('should load directory on mount', () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      expect(mockCommandExecutor.loadDirectory).toHaveBeenCalledWith('test-panel', '/test/path');
    });

    it('should reload directory when path changes', () => {
      const { rerender } = renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const updatedPanel = { ...mockPanel, currentPath: '/new/path' };
      rerender(<FilePanel panel={updatedPanel} isActive={true} />);

      expect(mockCommandExecutor.loadDirectory).toHaveBeenCalledWith('test-panel', '/new/path');
    });
  });

  describe('File Selection', () => {
    it('should select file on click', async () => {
      const { store } = renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file1 = screen.getByText('file1.txt');
      await user.click(file1);

      const state = store.getState();
      expect(state.panels.panels['test-panel']?.selectedFiles).toContain('file1.txt');
    });

    it('should handle multi-select with Ctrl+click', async () => {
      const { store } = renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file1 = screen.getByText('file1.txt');
      const file2 = screen.getByText('file2.txt');

      await user.click(file1);
      await user.keyboard('[ControlLeft>]');
      await user.click(file2);
      await user.keyboard('[/ControlLeft]');

      const state = store.getState();
      const selectedFiles = state.panels.panels['test-panel']?.selectedFiles || [];
      expect(selectedFiles).toContain('file1.txt');
      expect(selectedFiles).toContain('file2.txt');
    });
  });

  describe('File Operations', () => {
    it('should navigate to directory on double-click', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const folder = screen.getByText('folder1');
      await user.dblClick(folder);

      expect(mockCommandExecutor.navigateToDirectory).toHaveBeenCalledWith(
        'test-panel',
        expect.objectContaining({ name: 'folder1', file_type: 'Directory' })
      );
    });

    it('should not navigate on double-click for files', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file = screen.getByText('file1.txt');
      await user.dblClick(file);

      expect(mockCommandExecutor.navigateToDirectory).not.toHaveBeenCalled();
    });

    it('should navigate to parent on back button click', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const backButton = screen.getByRole('button', { name: /←/ });
      await user.click(backButton);

      expect(mockCommandExecutor.navigateToParent).toHaveBeenCalledWith('test-panel');
    });

    it('should disable back button at root', () => {
      const rootPanel = { ...mockPanel, currentPath: '/' };
      renderWithProviders(<FilePanel panel={rootPanel} isActive={true} />);

      const backButton = screen.getByRole('button', { name: /←/ });
      expect(backButton).toBeDisabled();
    });
  });

  describe('Keyboard Shortcuts', () => {
    it('should delete files on Delete key when files are selected', async () => {
      const panelWithSelection = {
        ...mockPanel,
        selectedFiles: ['file1.txt'],
      };

      renderWithProviders(<FilePanel panel={panelWithSelection} isActive={true} />);

      // Mock window.confirm
      const originalConfirm = global.confirm;
      global.confirm = vi.fn().mockReturnValue(true);

      try {
        await user.keyboard('[Delete]');

        expect(mockCommandExecutor.deleteFiles).toHaveBeenCalledWith(
          'test-panel',
          expect.arrayContaining([expect.objectContaining({ name: 'file1.txt' })])
        );
      } finally {
        global.confirm = originalConfirm;
      }
    });

    it('should not delete files when Delete is cancelled', async () => {
      const panelWithSelection = {
        ...mockPanel,
        selectedFiles: ['file1.txt'],
      };

      renderWithProviders(<FilePanel panel={panelWithSelection} isActive={true} />);

      const originalConfirm = global.confirm;
      global.confirm = vi.fn().mockReturnValue(false);

      try {
        await user.keyboard('[Delete]');

        expect(mockCommandExecutor.deleteFiles).not.toHaveBeenCalled();
      } finally {
        global.confirm = originalConfirm;
      }
    });

    it('should copy files on Ctrl+C', async () => {
      const panelWithSelection = {
        ...mockPanel,
        selectedFiles: ['file1.txt'],
      };

      renderWithProviders(<FilePanel panel={panelWithSelection} isActive={true} />);

      await user.keyboard('[ControlLeft>]c[/ControlLeft]');

      expect(mockCommandExecutor.copyFiles).toHaveBeenCalledWith(
        'test-panel',
        expect.arrayContaining([expect.objectContaining({ name: 'file1.txt' })])
      );
    });

    it('should cut files on Ctrl+X', async () => {
      const panelWithSelection = {
        ...mockPanel,
        selectedFiles: ['file1.txt'],
      };

      renderWithProviders(<FilePanel panel={panelWithSelection} isActive={true} />);

      await user.keyboard('[ControlLeft>]x[/ControlLeft]');

      expect(mockCommandExecutor.cutFiles).toHaveBeenCalledWith(
        'test-panel',
        expect.arrayContaining([expect.objectContaining({ name: 'file1.txt' })])
      );
    });

    it('should paste files on Ctrl+V', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      await user.keyboard('[ControlLeft>]v[/ControlLeft]');

      expect(mockCommandExecutor.pasteFiles).toHaveBeenCalledWith('test-panel');
    });

    it('should not handle keyboard shortcuts when panel is not active', async () => {
      const panelWithSelection = {
        ...mockPanel,
        selectedFiles: ['file1.txt'],
      };

      renderWithProviders(<FilePanel panel={panelWithSelection} isActive={false} />);

      await user.keyboard('[Delete]');

      expect(mockCommandExecutor.deleteFiles).not.toHaveBeenCalled();
    });
  });

  describe('Address Bar Navigation', () => {
    it('should navigate to path when address bar is submitted', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} addressBarActive={true} />);

      const addressBar = screen.getByDisplayValue('/test/path');
      await user.clear(addressBar);
      await user.type(addressBar, '/new/path');
      await user.keyboard('[Enter]');

      expect(mockCommandExecutor.navigateToPath).toHaveBeenCalledWith('test-panel', '/new/path');
    });

    it('should handle address bar navigation errors', async () => {
      mockCommandExecutor.navigateToPath.mockRejectedValue(new Error('Invalid path'));

      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} addressBarActive={true} />);

      const addressBar = screen.getByDisplayValue('/test/path');
      await user.clear(addressBar);
      await user.type(addressBar, '/invalid/path');
      await user.keyboard('[Enter]');

      expect(mockCommandExecutor.handleError).toHaveBeenCalledWith('test-panel', 'Invalid path');
    });
  });

  describe('Context Menu', () => {
    it('should show context menu on right-click', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file1 = screen.getByText('file1.txt');
      await user.pointer({ target: file1, keys: '[MouseRight]' });

      await waitFor(() => {
        expect(screen.getByText('Copy')).toBeInTheDocument();
        expect(screen.getByText('Cut')).toBeInTheDocument();
        expect(screen.getByText('Delete')).toBeInTheDocument();
        expect(screen.getByText('Rename')).toBeInTheDocument();
      });
    });

    it('should execute rename from context menu', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file1 = screen.getByText('file1.txt');
      await user.pointer({ target: file1, keys: '[MouseRight]' });

      await waitFor(() => {
        expect(screen.getByText('Rename')).toBeInTheDocument();
      });

      const originalPrompt = global.prompt;
      global.prompt = vi.fn().mockReturnValue('renamed-file.txt');

      try {
        await user.click(screen.getByText('Rename'));

        expect(mockCommandExecutor.renameFile).toHaveBeenCalledWith(
          'test-panel',
          expect.objectContaining({ name: 'file1.txt' }),
          'renamed-file.txt'
        );
      } finally {
        global.prompt = originalPrompt;
      }
    });

    it('should show paste option when clipboard has files', () => {
      const stateWithClipboard = {
        panels: {
          ...mockPanel,
          clipboardState: {
            hasFiles: true,
            files: [createMockFileInfo({ name: 'clipboard-file.txt' })],
            operation: 'copy' as const,
            sourcePanelId: 'other-panel',
          },
        },
      };

      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />, {
        preloadedState: stateWithClipboard,
      });

      // Right-click to show context menu
      const file1 = screen.getByText('file1.txt');
      fireEvent.contextMenu(file1);

      expect(screen.getByText(/Paste/)).toBeInTheDocument();
    });
  });

  describe('Drag and Drop', () => {
    it('should start drag operation', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file1 = screen.getByText('file1.txt');
      const mockDragEvent = {
        dataTransfer: {
          setData: vi.fn(),
          setDragImage: vi.fn(),
          effectAllowed: '',
        },
        currentTarget: {
          cloneNode: vi.fn().mockReturnValue(document.createElement('div')),
        },
        ctrlKey: false,
      };

      fireEvent.dragStart(file1, mockDragEvent);

      expect(mockCommandExecutor.startDrag).toHaveBeenCalledWith(
        'test-panel',
        expect.objectContaining({ name: 'file1.txt' }),
        false,
        mockDragEvent
      );
    });

    it('should end drag operation', async () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const file1 = screen.getByText('file1.txt');
      fireEvent.dragEnd(file1);

      expect(mockCommandExecutor.endDrag).toHaveBeenCalled();
    });

    it('should handle drop operation', async () => {
      const stateWithDrag = {
        panels: {
          ...mockPanel,
          dragState: {
            isDragging: true,
            draggedFiles: ['file1.txt'],
            sourcePanelId: 'other-panel',
            dragOperation: 'move' as const,
          },
        },
      };

      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />, {
        preloadedState: stateWithDrag,
      });

      const filePanel = screen.getByRole('main', { hidden: true }) || document.body.firstElementChild;
      if (filePanel) {
        fireEvent.drop(filePanel);

        expect(mockCommandExecutor.handleDrop).toHaveBeenCalledWith(
          'test-panel',
          expect.objectContaining({
            isDragging: true,
            draggedFiles: ['file1.txt'],
            sourcePanelId: 'other-panel',
            dragOperation: 'move',
          })
        );
      }
    });
  });

  describe('File Sorting and Display', () => {
    it('should sort files by name by default', () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      const fileElements = screen.getAllByText(/\.(txt|)$/);
      expect(fileElements[0]).toHaveTextContent('file1.txt');
      expect(fileElements[1]).toHaveTextContent('file2.txt');
    });

    it('should display folder icon for directories', () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      // Look for folder emoji or icon
      expect(screen.getByText('📁')).toBeInTheDocument();
    });

    it('should display file icon for files', () => {
      renderWithProviders(<FilePanel panel={mockPanel} isActive={true} />);

      // Should have multiple file icons for the two files
      const fileIcons = screen.getAllByText('📄');
      expect(fileIcons).toHaveLength(2);
    });
  });

  describe('Error Handling', () => {
    it('should handle CommandExecutor errors gracefully', async () => {
      mockCommandExecutor.deleteFiles.mockRejectedValue(new Error('Delete failed'));

      const panelWithSelection = {
        ...mockPanel,
        selectedFiles: ['file1.txt'],
      };

      renderWithProviders(<FilePanel panel={panelWithSelection} isActive={true} />);

      const originalConfirm = global.confirm;
      global.confirm = vi.fn().mockReturnValue(true);

      try {
        await user.keyboard('[Delete]');

        // Should not crash the component
        expect(screen.getByText('file1.txt')).toBeInTheDocument();
      } finally {
        global.confirm = originalConfirm;
      }
    });
  });
});