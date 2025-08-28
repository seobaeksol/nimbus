import { CreateFileCommand, DeleteFilesCommand, RenameFileCommand } from '../implementations/file';
import { CommandExecutor } from '../../commandExecutor';
import { MockDialogService } from '../services/DialogService';
import { ExecutionContext } from '../types';
import { FileInfo } from '../../../types';

// Mock CommandExecutor static methods
jest.mock('../../commandExecutor', () => ({
  CommandExecutor: {
    createFile: jest.fn(),
    deleteFiles: jest.fn(),
    renameFile: jest.fn()
  }
}));

describe('File Commands', () => {
  let mockDialogService: MockDialogService;
  let executor: CommandExecutor;
  let mockContext: ExecutionContext;

  beforeEach(() => {
    mockDialogService = new MockDialogService();
    executor = new CommandExecutor();
    
    mockContext = {
      panelId: 'panel-1',
      currentPath: '/test/path',
      selectedFiles: [],
      dispatch: jest.fn(),
      clipboardHasFiles: false,
      panels: { 'panel-1': { id: 'panel-1', path: '/test/path' } }
    };

    jest.clearAllMocks();
  });

  describe('CreateFileCommand', () => {
    let command: CreateFileCommand;

    beforeEach(() => {
      command = new CreateFileCommand(executor, mockDialogService);
    });

    it('should have correct metadata', () => {
      expect(command.metadata).toEqual({
        id: 'create-file',
        label: 'New File',
        category: 'File',
        description: 'Create a new file in the current directory',
        icon: 'file-plus',
        shortcut: 'Ctrl+N'
      });
    });

    it('should require current path to execute', () => {
      expect(command.canExecute(mockContext)).toBe(true);
      
      const contextWithoutPath = { ...mockContext, currentPath: '' };
      expect(command.canExecute(contextWithoutPath)).toBe(false);
    });

    it('should create file when user provides name', async () => {
      mockDialogService.queuePromptResponse('test-file.txt');
      (CommandExecutor.createFile as jest.Mock).mockResolvedValue(undefined);

      await command.execute(mockContext);

      expect(CommandExecutor.createFile).toHaveBeenCalledWith('panel-1', 'test-file.txt');
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Created file: test-file.txt',
        type: 'success'
      });
    });

    it('should cancel operation when user cancels prompt', async () => {
      mockDialogService.queuePromptResponse(null);

      await command.execute(mockContext);

      expect(CommandExecutor.createFile).not.toHaveBeenCalled();
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'File creation cancelled',
        type: 'info'
      });
    });
  });

  describe('DeleteFilesCommand', () => {
    let command: DeleteFilesCommand;

    beforeEach(() => {
      command = new DeleteFilesCommand(executor, mockDialogService);
    });

    it('should require file selection', () => {
      expect(command.canExecute(mockContext)).toBe(false);

      const contextWithSelection = {
        ...mockContext,
        selectedFiles: [{ name: 'test.txt' } as FileInfo]
      };
      expect(command.canExecute(contextWithSelection)).toBe(true);
    });

    it('should delete files after confirmation', async () => {
      const selectedFile: FileInfo = {
        name: 'test.txt',
        path: '/test/path/test.txt',
        size: 1024,
        size_formatted: '1 KB',
        modified: '2024-01-01T00:00:00Z',
        file_type: 'File',
        extension: 'txt'
      };

      const contextWithSelection = {
        ...mockContext,
        selectedFiles: [selectedFile]
      };

      mockDialogService.queueConfirmResponse(true);
      (CommandExecutor.deleteFiles as jest.Mock).mockResolvedValue(undefined);

      await command.execute(contextWithSelection);

      expect(CommandExecutor.deleteFiles).toHaveBeenCalledWith('panel-1', [selectedFile]);
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Deleted 1 item',
        type: 'success'
      });
    });

    it('should cancel deletion when user declines confirmation', async () => {
      const contextWithSelection = {
        ...mockContext,
        selectedFiles: [{ name: 'test.txt' } as FileInfo]
      };

      mockDialogService.queueConfirmResponse(false);

      await command.execute(contextWithSelection);

      expect(CommandExecutor.deleteFiles).not.toHaveBeenCalled();
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Delete operation cancelled',
        type: 'info'
      });
    });
  });

  describe('RenameFileCommand', () => {
    let command: RenameFileCommand;

    beforeEach(() => {
      command = new RenameFileCommand(executor, mockDialogService);
    });

    it('should require exactly one file selected', () => {
      expect(command.canExecute(mockContext)).toBe(false);

      const contextWithOne = {
        ...mockContext,
        selectedFiles: [{ name: 'test.txt' } as FileInfo]
      };
      expect(command.canExecute(contextWithOne)).toBe(true);

      const contextWithTwo = {
        ...mockContext,
        selectedFiles: [
          { name: 'test1.txt' } as FileInfo,
          { name: 'test2.txt' } as FileInfo
        ]
      };
      expect(command.canExecute(contextWithTwo)).toBe(false);
    });

    it('should rename file with new name', async () => {
      const selectedFile: FileInfo = {
        name: 'old-name.txt',
        path: '/test/path/old-name.txt',
        size: 1024,
        size_formatted: '1 KB',
        modified: '2024-01-01T00:00:00Z',
        file_type: 'File',
        extension: 'txt'
      };

      const contextWithSelection = {
        ...mockContext,
        selectedFiles: [selectedFile]
      };

      mockDialogService.queuePromptResponse('new-name.txt');
      (CommandExecutor.renameFile as jest.Mock).mockResolvedValue(undefined);

      await command.execute(contextWithSelection);

      expect(CommandExecutor.renameFile).toHaveBeenCalledWith('panel-1', selectedFile, 'new-name.txt');
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Renamed "old-name.txt" to "new-name.txt"',
        type: 'success'
      });
    });

    it('should cancel rename when user cancels or provides same name', async () => {
      const selectedFile = { name: 'test.txt' } as FileInfo;
      const contextWithSelection = {
        ...mockContext,
        selectedFiles: [selectedFile]
      };

      // Test cancellation
      mockDialogService.queuePromptResponse(null);
      await command.execute(contextWithSelection);
      expect(CommandExecutor.renameFile).not.toHaveBeenCalled();

      // Test same name
      mockDialogService.queuePromptResponse('test.txt');
      await command.execute(contextWithSelection);
      expect(CommandExecutor.renameFile).not.toHaveBeenCalled();
    });
  });
});