import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { CommandService } from '../commands/services/CommandService';
import { CommandExecutorService } from '../commands/services/CommandExecutorService';
import { FileService } from '../fileService';
import { PathAliasService } from '../pathAliasService';
import { createTestStore, mockPanelState, createMockFileInfo } from '../../test/testUtils';
import type { ExecutionContext } from '../commands/types';

// Mock dependencies
vi.mock('../fileService');
vi.mock('../pathAliasService');

const mockFileService = vi.mocked(FileService);
const mockPathAliasService = vi.mocked(PathAliasService);

describe('CommandExecutorService', () => {
  let store: ReturnType<typeof createTestStore>;
  let mockDispatch: ReturnType<typeof vi.fn>;
  let testContext: ExecutionContext;

  beforeEach(() => {
    store = createTestStore({ panels: mockPanelState });
    mockDispatch = vi.fn();
    
    testContext = {
      panelId: 'panel-1',
      currentPath: '/test/path',
      selectedFiles: [],
      dispatch: mockDispatch,
      panels: mockPanelState.panels,
      clipboardHasFiles: false,
      clipboardState: mockPanelState.clipboardState,
    };

    CommandService.initialize(mockDispatch);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('File Operations', () => {
    describe('createFile', () => {
      it('should create a file with simple name in current directory', async () => {
        mockFileService.createFile.mockResolvedValue(undefined);

        const commandService = CommandService.getInstance();
        await commandService.executeCommand('create-file', testContext);

        expect(mockFileService.createFile).toHaveBeenCalledWith('/test/path', expect.any(String));
        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/setLoading',
            payload: { panelId: 'panel-1', isLoading: true }
          })
        );
      });

      it('should create a file with relative path', async () => {
        mockFileService.createFile.mockResolvedValue(undefined);

        await CommandExecutor.createFile('panel-1', 'subdir/test.txt');

        expect(mockFileService.createFile).toHaveBeenCalledWith('/test/path/subdir', 'test.txt');
      });

      it('should create a file with absolute path', async () => {
        mockFileService.createFile.mockResolvedValue(undefined);

        await CommandExecutor.createFile('panel-1', '/absolute/path/test.txt');

        expect(mockFileService.createFile).toHaveBeenCalledWith('/absolute/path', 'test.txt');
      });

      it('should handle creation errors gracefully', async () => {
        const error = new Error('Permission denied');
        mockFileService.createFile.mockRejectedValue(error);

        await CommandExecutor.createFile('panel-1', 'test.txt');

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/addNotification',
            payload: expect.objectContaining({
              message: 'Failed to create file: Permission denied',
              type: 'error'
            })
          })
        );
      });

      it('should always clear loading state', async () => {
        const error = new Error('Test error');
        mockFileService.createFile.mockRejectedValue(error);

        await CommandExecutor.createFile('panel-1', 'test.txt');

        const loadingCalls = mockDispatch.mock.calls.filter(call => 
          call[0].type === 'panels/setLoading'
        );
        
        expect(loadingCalls).toHaveLength(2);
        expect(loadingCalls[0][0].payload).toEqual({ panelId: 'panel-1', isLoading: true });
        expect(loadingCalls[1][0].payload).toEqual({ panelId: 'panel-1', isLoading: false });
      });
    });

    describe('deleteFiles', () => {
      it('should delete multiple files and refresh directory', async () => {
        const files = [
          createMockFileInfo({ name: 'file1.txt', path: '/test/path/file1.txt' }),
          createMockFileInfo({ name: 'file2.txt', path: '/test/path/file2.txt' }),
        ];

        mockFileService.deleteItem.mockResolvedValue(undefined);
        mockFileService.listDirectory.mockResolvedValue([]);

        await CommandExecutor.deleteFiles('panel-1', files);

        expect(mockFileService.deleteItem).toHaveBeenCalledTimes(2);
        expect(mockFileService.deleteItem).toHaveBeenCalledWith('/test/path/file1.txt');
        expect(mockFileService.deleteItem).toHaveBeenCalledWith('/test/path/file2.txt');
        expect(mockFileService.listDirectory).toHaveBeenCalledWith('/test/path');
      });

      it('should clear selection after deletion', async () => {
        const files = [createMockFileInfo({ name: 'file1.txt' })];
        mockFileService.deleteItem.mockResolvedValue(undefined);
        mockFileService.listDirectory.mockResolvedValue([]);

        await CommandExecutor.deleteFiles('panel-1', files);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/selectFiles',
            payload: { panelId: 'panel-1', fileNames: [] }
          })
        );
      });
    });

    describe('renameFile', () => {
      it('should rename a file and refresh directory', async () => {
        const file = createMockFileInfo({ 
          name: 'oldname.txt', 
          path: '/test/path/oldname.txt' 
        });

        mockFileService.renameItem.mockResolvedValue(undefined);
        mockFileService.listDirectory.mockResolvedValue([]);

        await CommandExecutor.renameFile('panel-1', file, 'newname.txt');

        expect(mockFileService.renameItem).toHaveBeenCalledWith('/test/path/oldname.txt', 'newname.txt');
        expect(mockFileService.listDirectory).toHaveBeenCalledWith('/test/path');
      });
    });

    describe('copyFiles and cutFiles', () => {
      it('should copy files to clipboard', async () => {
        const files = [createMockFileInfo({ name: 'file1.txt' })];

        await CommandExecutor.copyFiles('panel-1', files);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/copyFilesToClipboard',
            payload: { panelId: 'panel-1', files }
          })
        );
      });

      it('should cut files to clipboard', async () => {
        const files = [createMockFileInfo({ name: 'file1.txt' })];

        await CommandExecutor.cutFiles('panel-1', files);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/cutFilesToClipboard',
            payload: { panelId: 'panel-1', files }
          })
        );
      });
    });
  });

  describe('Navigation Operations', () => {
    describe('navigateToDirectory', () => {
      it('should navigate to subdirectory', async () => {
        const directory = createMockFileInfo({ 
          name: 'subdir', 
          file_type: 'Directory' 
        });

        await CommandExecutor.navigateToDirectory('panel-1', directory);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: '/test/path/subdir' }
          })
        );
      });

      it('should handle root directory navigation', async () => {
        // Update test context to simulate root directory
        testContext.panels['panel-1'].currentPath = '/';
        CommandExecutor.initialize(testContext);

        const directory = createMockFileInfo({ 
          name: 'home', 
          file_type: 'Directory' 
        });

        await CommandExecutor.navigateToDirectory('panel-1', directory);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: '/home' }
          })
        );
      });
    });

    describe('navigateToParent', () => {
      it('should navigate to parent directory', async () => {
        await CommandExecutor.navigateToParent('panel-1');

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: '/test' }
          })
        );
      });

      it('should stay at root when already at root', async () => {
        testContext.panels['panel-1'].currentPath = '/';
        CommandExecutor.initialize(testContext);

        await CommandExecutor.navigateToParent('panel-1');

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: '/' }
          })
        );
      });
    });

    describe('navigateToPath', () => {
      it('should resolve path and navigate', async () => {
        const inputPath = '~/Documents';
        const resolvedPath = '/home/user/Documents';
        
        mockFileService.resolvePath.mockResolvedValue(resolvedPath);

        await CommandExecutor.navigateToPath('panel-1', inputPath);

        expect(mockFileService.resolvePath).toHaveBeenCalledWith(inputPath);
        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: resolvedPath }
          })
        );
      });

      it('should throw error for invalid paths', async () => {
        const error = new Error('Invalid path');
        mockFileService.resolvePath.mockRejectedValue(error);

        await expect(CommandExecutor.navigateToPath('panel-1', 'invalid://path')).rejects.toThrow('Invalid path');
      });
    });

    describe('Common directory navigation', () => {
      beforeEach(() => {
        mockPathAliasService.resolvePath.mockImplementation((alias: string) => {
          const paths = {
            '~': '/home/user',
            'Documents': '/home/user/Documents',
            'Desktop': '/home/user/Desktop',
            'Downloads': '/home/user/Downloads',
            'Applications': '/Applications',
          };
          return Promise.resolve(paths[alias as keyof typeof paths] || alias);
        });
      });

      it('should navigate to home directory', async () => {
        await CommandExecutor.goToHome('panel-1');

        expect(mockPathAliasService.resolvePath).toHaveBeenCalledWith('~');
        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: '/home/user' }
          })
        );
      });

      it('should navigate to Documents', async () => {
        await CommandExecutor.goToDocuments('panel-1');

        expect(mockPathAliasService.resolvePath).toHaveBeenCalledWith('Documents');
        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/navigateToPath',
            payload: { panelId: 'panel-1', path: '/home/user/Documents' }
          })
        );
      });
    });
  });

  describe('Drag and Drop Operations', () => {
    describe('startDrag', () => {
      it('should start drag operation with selected files', async () => {
        // Update panel to have selected files
        testContext.panels['panel-1'].selectedFiles = ['file1.txt', 'file2.txt'];
        CommandExecutor.initialize(testContext);

        const file = createMockFileInfo({ name: 'file1.txt' });
        const mockEvent = {
          dataTransfer: {
            setData: vi.fn(),
            setDragImage: vi.fn(),
          },
          currentTarget: {
            cloneNode: vi.fn().mockReturnValue(document.createElement('div')),
          },
        } as any;

        await CommandExecutor.startDrag('panel-1', file, false, mockEvent);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/startDrag',
            payload: {
              panelId: 'panel-1',
              fileNames: ['file1.txt', 'file2.txt'],
              operation: 'move'
            }
          })
        );
      });

      it('should start drag operation with copy when Ctrl is pressed', async () => {
        const file = createMockFileInfo({ name: 'file1.txt' });
        const mockEvent = {
          dataTransfer: {
            setData: vi.fn(),
            setDragImage: vi.fn(),
          },
          currentTarget: {
            cloneNode: vi.fn().mockReturnValue(document.createElement('div')),
          },
        } as any;

        await CommandExecutor.startDrag('panel-1', file, true, mockEvent);

        expect(mockDispatch).toHaveBeenCalledWith(
          expect.objectContaining({
            type: 'panels/startDrag',
            payload: expect.objectContaining({
              operation: 'copy'
            })
          })
        );
      });
    });
  });

  describe('Directory Loading', () => {
    it('should load directory and update panel files', async () => {
      const mockFiles = [
        createMockFileInfo({ name: 'file1.txt' }),
        createMockFileInfo({ name: 'file2.txt' }),
      ];

      mockFileService.listDirectory.mockResolvedValue(mockFiles);

      await CommandExecutor.loadDirectory('panel-1', '/test/path');

      expect(mockFileService.listDirectory).toHaveBeenCalledWith('/test/path');
      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'panels/setFiles',
          payload: { panelId: 'panel-1', files: mockFiles }
        })
      );
    });

    it('should handle directory loading errors', async () => {
      const error = new Error('Access denied');
      mockFileService.listDirectory.mockRejectedValue(error);

      await CommandExecutor.loadDirectory('panel-1', '/restricted/path');

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'panels/addNotification',
          payload: expect.objectContaining({
            message: 'Cannot access directory "/restricted/path": Access denied',
            type: 'error'
          })
        })
      );
    });
  });

  describe('Path parsing', () => {
    it('should parse relative paths correctly', async () => {
      mockFileService.createFile.mockResolvedValue(undefined);

      await CommandExecutor.createFile('panel-1', 'subdir/file.txt');

      expect(mockFileService.createFile).toHaveBeenCalledWith('/test/path/subdir', 'file.txt');
    });

    it('should parse absolute paths correctly', async () => {
      mockFileService.createFile.mockResolvedValue(undefined);

      await CommandExecutor.createFile('panel-1', '/absolute/path/file.txt');

      expect(mockFileService.createFile).toHaveBeenCalledWith('/absolute/path', 'file.txt');
    });

    it('should handle Windows-style paths', async () => {
      mockFileService.createFile.mockResolvedValue(undefined);

      await CommandExecutor.createFile('panel-1', 'C:\\Windows\\System32\\file.txt');

      expect(mockFileService.createFile).toHaveBeenCalledWith('C:/Windows/System32', 'file.txt');
    });

    it('should throw error for empty filename', async () => {
      await expect(CommandExecutor.createFile('panel-1', '/test/path/')).rejects.toThrow('Filename cannot be empty');
    });
  });

  describe('Error handling', () => {
    it('should handle missing panel gracefully', async () => {
      await CommandExecutor.createFile('non-existent-panel', 'test.txt');

      // Should not crash and should not call FileService
      expect(mockFileService.createFile).not.toHaveBeenCalled();
    });

    it('should handle null/undefined inputs', async () => {
      await expect(CommandExecutor.createFile('panel-1', '')).rejects.toThrow();
      await CommandExecutor.deleteFiles('panel-1', []);
      
      // Should not crash for empty arrays
      expect(mockFileService.deleteItem).not.toHaveBeenCalled();
    });
  });
});