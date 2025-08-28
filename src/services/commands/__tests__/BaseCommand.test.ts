import { BaseCommand } from '../base/BaseCommand';
import { CommandMetadata, ExecutionContext } from '../types';
import { CommandExecutor } from '../../commandExecutor';
import { MockDialogService } from '../services/DialogService';

// Test concrete implementation of BaseCommand
class TestCommand extends BaseCommand {
  constructor(executor: CommandExecutor, dialogService: MockDialogService) {
    const metadata: CommandMetadata = {
      id: 'test-command',
      label: 'Test Command',
      category: 'Test',
      description: 'A test command for unit testing'
    };
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    this.showSuccess('Test command executed');
  }
}

describe('BaseCommand', () => {
  let testCommand: TestCommand;
  let mockDialogService: MockDialogService;
  let executor: CommandExecutor;
  let mockContext: ExecutionContext;

  beforeEach(() => {
    mockDialogService = new MockDialogService();
    executor = new CommandExecutor();
    testCommand = new TestCommand(executor, mockDialogService);
    
    mockContext = {
      panelId: 'panel-1',
      currentPath: '/test/path',
      selectedFiles: [],
      dispatch: jest.fn(),
      clipboardHasFiles: false,
      panels: { 'panel-1': { id: 'panel-1', path: '/test/path' } }
    };
  });

  describe('Metadata', () => {
    it('should expose command metadata', () => {
      expect(testCommand.metadata).toEqual({
        id: 'test-command',
        label: 'Test Command',
        category: 'Test',
        description: 'A test command for unit testing'
      });
    });
  });

  describe('Execution Control', () => {
    it('should allow execution by default', () => {
      expect(testCommand.canExecute(mockContext)).toBe(true);
    });

    it('should execute successfully', async () => {
      await testCommand.execute(mockContext);
      
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Test command executed',
        type: 'success'
      });
    });
  });

  describe('Error Handling', () => {
    let errorCommand: BaseCommand;

    beforeEach(() => {
      class ErrorCommand extends BaseCommand {
        async execute(): Promise<void> {
          await this.withErrorHandling(
            async () => {
              throw new Error('Test error');
            },
            'Operation failed'
          );
        }
      }

      errorCommand = new ErrorCommand(
        {
          id: 'error-command',
          label: 'Error Command',
          category: 'Test'
        },
        executor,
        mockDialogService
      );
    });

    it('should handle errors gracefully', async () => {
      await errorCommand.execute(mockContext);
      
      const notification = mockDialogService.getLastNotification();
      expect(notification?.type).toBe('error');
      expect(notification?.message).toContain('Operation failed: Test error');
    });
  });

  describe('Notification Methods', () => {
    it('should show success notifications', () => {
      testCommand['showSuccess']('Success message');
      
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Success message',
        type: 'success'
      });
    });

    it('should show error notifications', () => {
      testCommand['showError']('Error message');
      
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Error message',
        type: 'error'
      });
    });

    it('should show info notifications', () => {
      testCommand['showInfo']('Info message');
      
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Info message',
        type: 'info'
      });
    });

    it('should show warning notifications', () => {
      testCommand['showWarning']('Warning message');
      
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Warning message',
        type: 'warning'
      });
    });
  });
});