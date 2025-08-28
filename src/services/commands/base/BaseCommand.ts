import { Command, CommandMetadata, DialogService, ExecutionContext } from '../types';
import { CommandExecutorService } from '../services/CommandExecutorService';

/**
 * Abstract base class for all commands
 * Provides common functionality and dependency injection
 */
export abstract class BaseCommand implements Command {
  constructor(
    public readonly metadata: CommandMetadata,
    protected executor: CommandExecutorService,
    protected dialogService: DialogService
  ) {}

  /**
   * Determines if command can be executed in current context
   * Override in subclasses for specific conditions
   */
  canExecute(context: ExecutionContext): boolean {
    return true;
  }

  /**
   * Execute the command - must be implemented by subclasses
   */
  abstract execute(context: ExecutionContext): Promise<void>;

  /**
   * Common error handling wrapper
   */
  protected async withErrorHandling<T>(
    operation: () => Promise<T>,
    errorPrefix?: string
  ): Promise<T | void> {
    try {
      return await operation();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      const fullMessage = errorPrefix ? `${errorPrefix}: ${message}` : message;
      
      this.dialogService.showNotification(fullMessage, 'error');
      throw error;
    }
  }

  /**
   * Show success notification
   */
  protected showSuccess(message: string): void {
    this.dialogService.showNotification(message, 'success');
  }

  /**
   * Show info notification
   */
  protected showInfo(message: string): void {
    this.dialogService.showNotification(message, 'info');
  }
}