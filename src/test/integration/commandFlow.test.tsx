import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderWithProviders, mockFileService, createMockFileInfo } from '../testUtils';
import { CommandService } from '../../services/commands/services/CommandService';
import { ExecutionContext } from '../../services/commands/types';

// Mock CommandExecutor to test integration points
vi.mock('../../services/commandExecutor', () => ({
  CommandExecutor: {
    initialize: vi.fn(),
    createFile: vi.fn(),
    createFolder: vi.fn(),
    deleteFiles: vi.fn(),
    renameFile: vi.fn(),
    copyFiles: vi.fn(),
    cutFiles: vi.fn(),
    pasteFiles: vi.fn(),
    goToHome: vi.fn(),
    goToDocuments: vi.fn(),
    goToDesktop: vi.fn(),
    navigateToPath: vi.fn(),
    loadDirectory: vi.fn(),
    focusAddressBar: vi.fn(),
  },
}));

const mockCommandExecutor = vi.mocked(CommandExecutor);

describe('Command Flow Integration Tests', () => {
  let mockContext: CommandContext;

  beforeEach(() => {
    mockContext = {
      activePanelId: 'panel-1',
      selectedFiles: [],
      currentPath: '/test/path',
      dispatch: vi.fn(),
      panels: {},
      clipboardHasFiles: false,
    };

    CommandRegistry.initialize();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('File Operations Command Flow', () => {
    it('should execute create file command through CommandRegistry', async () => {
      // Get the create file command from registry
      const commands = CommandRegistry.searchCommands('create file', mockContext);
      const createFileCommand = commands.find(cmd => cmd.id === 'file.create.file');

      expect(createFileCommand).toBeDefined();
      expect(createFileCommand?.label).toBe('File: Create New File');

      // Mock prompt to return a filename
      const originalPrompt = global.prompt;
      global.prompt = vi.fn().mockReturnValue('test-file.txt');

      try {
        // Execute the command
        await createFileCommand?.action(mockContext);

        // Verify CommandExecutor was called
        expect(mockCommandExecutor.createFile).toHaveBeenCalledWith('panel-1', 'test-file.txt');
      } finally {
        global.prompt = originalPrompt;
      }
    });

    it('should execute delete files command with selected files', async () => {
      const selectedFiles = [
        createMockFileInfo({ name: 'file1.txt' }),
        createMockFileInfo({ name: 'file2.txt' }),
      ];

      const contextWithSelection = {
        ...mockContext,
        selectedFiles,
      };

      const commands = CommandRegistry.searchCommands('delete', contextWithSelection);
      const deleteCommand = commands.find(cmd => cmd.id === 'file.delete');

      expect(deleteCommand).toBeDefined();

      // Mock confirm dialog
      const originalConfirm = global.confirm;
      global.confirm = vi.fn().mockReturnValue(true);

      try {
        await deleteCommand?.action(contextWithSelection);

        expect(mockCommandExecutor.deleteFiles).toHaveBeenCalledWith('panel-1', selectedFiles);
      } finally {
        global.confirm = originalConfirm;
      }
    });

    it('should not show delete command when no files selected', () => {
      const commands = CommandRegistry.searchCommands('delete', mockContext);
      const deleteCommand = commands.find(cmd => cmd.id === 'file.delete');

      // Command should not appear when no files are selected
      expect(deleteCommand).toBeUndefined();
    });

    it('should execute copy command with selected files', async () => {
      const selectedFiles = [createMockFileInfo({ name: 'file1.txt' })];
      const contextWithSelection = { ...mockContext, selectedFiles };

      const commands = CommandRegistry.searchCommands('copy', contextWithSelection);
      const copyCommand = commands.find(cmd => cmd.id === 'file.copy');

      expect(copyCommand).toBeDefined();

      await copyCommand?.action(contextWithSelection);

      expect(mockCommandExecutor.copyFiles).toHaveBeenCalledWith('panel-1', selectedFiles);
    });

    it('should execute paste command when clipboard has files', async () => {
      const contextWithClipboard = { ...mockContext, clipboardHasFiles: true };

      const commands = CommandRegistry.searchCommands('paste', contextWithClipboard);
      const pasteCommand = commands.find(cmd => cmd.id === 'file.paste');

      expect(pasteCommand).toBeDefined();

      await pasteCommand?.action(contextWithClipboard);

      expect(mockCommandExecutor.pasteFiles).toHaveBeenCalledWith('panel-1');
    });
  });

  describe('Navigation Commands Flow', () => {
    it('should execute go to home command', async () => {
      const commands = CommandRegistry.searchCommands('home', mockContext);
      const homeCommand = commands.find(cmd => cmd.id === 'navigation.home');

      expect(homeCommand).toBeDefined();
      expect(homeCommand?.label).toBe('Go: Home Directory');

      await homeCommand?.action(mockContext);

      expect(mockCommandExecutor.goToHome).toHaveBeenCalledWith('panel-1');
    });

    it('should execute go to documents command', async () => {
      const commands = CommandRegistry.searchCommands('documents', mockContext);
      const documentsCommand = commands.find(cmd => cmd.id === 'navigation.documents');

      expect(documentsCommand).toBeDefined();

      await documentsCommand?.action(mockContext);

      expect(mockCommandExecutor.goToDocuments).toHaveBeenCalledWith('panel-1');
    });

    it('should execute address bar focus command', async () => {
      const commands = CommandRegistry.searchCommands('address', mockContext);
      const addressCommand = commands.find(cmd => cmd.id === 'navigation.address-bar');

      expect(addressCommand).toBeDefined();

      // Mock the custom event dispatch
      const originalDispatchEvent = window.dispatchEvent;
      const mockDispatchEvent = vi.fn();
      window.dispatchEvent = mockDispatchEvent;

      try {
        await addressCommand?.action(mockContext);

        expect(mockDispatchEvent).toHaveBeenCalledWith(
          expect.any(CustomEvent)
        );
      } finally {
        window.dispatchEvent = originalDispatchEvent;
      }
    });
  });

  describe('Command Search and Filtering', () => {
    it('should filter commands based on search term', () => {
      const allCommands = CommandRegistry.searchCommands('', mockContext);
      const fileCommands = CommandRegistry.searchCommands('file', mockContext);
      const navCommands = CommandRegistry.searchCommands('go', mockContext);

      expect(allCommands.length).toBeGreaterThan(0);
      expect(fileCommands.length).toBeGreaterThan(0);
      expect(navCommands.length).toBeGreaterThan(0);

      // File commands should be a subset of all commands
      expect(fileCommands.length).toBeLessThanOrEqual(allCommands.length);

      // Each file command should contain 'file' in label or description
      fileCommands.forEach(cmd => {
        const hasFileInLabel = cmd.label.toLowerCase().includes('file');
        const hasFileInDescription = cmd.description?.toLowerCase().includes('file');
        expect(hasFileInLabel || hasFileInDescription).toBe(true);
      });
    });

    it('should respect command visibility conditions', () => {
      // Test with no selected files
      const noSelectionCommands = CommandRegistry.searchCommands('', mockContext);
      const deleteCommand = noSelectionCommands.find(cmd => cmd.id === 'file.delete');
      expect(deleteCommand).toBeUndefined();

      // Test with selected files
      const contextWithSelection = {
        ...mockContext,
        selectedFiles: [createMockFileInfo({ name: 'file1.txt' })],
      };
      const selectionCommands = CommandRegistry.searchCommands('', contextWithSelection);
      const deleteWithSelectionCommand = selectionCommands.find(cmd => cmd.id === 'file.delete');
      expect(deleteWithSelectionCommand).toBeDefined();
    });

    it('should handle empty search results gracefully', () => {
      const noResults = CommandRegistry.searchCommands('xyz123nonexistent', mockContext);
      expect(noResults).toEqual([]);
    });
  });

  describe('Command Categories', () => {
    it('should categorize commands correctly', () => {
      const allCommands = CommandRegistry.searchCommands('', mockContext);
      
      const fileCommands = allCommands.filter(cmd => cmd.category === 'File');
      const navCommands = allCommands.filter(cmd => cmd.category === 'Navigation');

      expect(fileCommands.length).toBeGreaterThan(0);
      expect(navCommands.length).toBeGreaterThan(0);

      // Verify some expected commands exist
      expect(fileCommands.some(cmd => cmd.id === 'file.create.file')).toBe(true);
      expect(fileCommands.some(cmd => cmd.id === 'file.create.folder')).toBe(true);
      expect(navCommands.some(cmd => cmd.id === 'navigation.home')).toBe(true);
    });

    it('should provide appropriate shortcuts for commands', () => {
      const allCommands = CommandRegistry.searchCommands('', mockContext);
      
      const createFileCmd = allCommands.find(cmd => cmd.id === 'file.create.file');
      const createFolderCmd = allCommands.find(cmd => cmd.id === 'file.create.folder');
      const addressBarCmd = allCommands.find(cmd => cmd.id === 'navigation.address-bar');

      expect(createFileCmd?.shortcut).toBe('Ctrl+T');
      expect(createFolderCmd?.shortcut).toBe('Ctrl+N');
      expect(addressBarCmd?.shortcut).toBe('Ctrl+L');
    });
  });

  describe('Error Handling in Command Flow', () => {
    it('should handle CommandExecutor errors gracefully', async () => {
      const error = new Error('Test error');
      mockCommandExecutor.createFile.mockRejectedValue(error);

      const commands = CommandRegistry.searchCommands('create file', mockContext);
      const createFileCommand = commands.find(cmd => cmd.id === 'file.create.file');

      const originalPrompt = global.prompt;
      global.prompt = vi.fn().mockReturnValue('test.txt');

      try {
        // This should not throw - CommandExecutor should handle the error
        await createFileCommand?.action(mockContext);

        expect(mockCommandExecutor.createFile).toHaveBeenCalled();
      } finally {
        global.prompt = originalPrompt;
      }
    });

    it('should handle missing panel ID gracefully', async () => {
      const contextWithoutPanel = { ...mockContext, activePanelId: null };
      
      const commands = CommandRegistry.searchCommands('create file', contextWithoutPanel);
      const createFileCommand = commands.find(cmd => cmd.id === 'file.create.file');

      const originalPrompt = global.prompt;
      global.prompt = vi.fn().mockReturnValue('test.txt');

      try {
        await createFileCommand?.action(contextWithoutPanel);

        // Should not call CommandExecutor when no active panel
        expect(mockCommandExecutor.createFile).not.toHaveBeenCalled();
      } finally {
        global.prompt = originalPrompt;
      }
    });

    it('should handle cancelled user dialogs', async () => {
      const commands = CommandRegistry.searchCommands('create file', mockContext);
      const createFileCommand = commands.find(cmd => cmd.id === 'file.create.file');

      const originalPrompt = global.prompt;
      global.prompt = vi.fn().mockReturnValue(null); // User cancelled

      try {
        await createFileCommand?.action(mockContext);

        // Should not call CommandExecutor when user cancels
        expect(mockCommandExecutor.createFile).not.toHaveBeenCalled();
      } finally {
        global.prompt = originalPrompt;
      }
    });
  });
});