import {
  Command,
  CommandMetadata,
  DialogService,
  ExecutionContext,
  NotificationType,
} from "../types";
import { AppDispatch } from "@/store";
import { addNotification } from "@/store/slices/panelSlice";

/**
 * Abstract base class for all commands
 * Provides common functionality and dependency injection
 */
export abstract class BaseCommand<
  T extends Record<string, unknown> = Record<string, never>,
> implements Command<T>
{
  constructor(
    public readonly metadata: CommandMetadata,
    protected dispatch: AppDispatch,
    protected dialogService: DialogService
  ) {}

  /**
   * Determines if command can be executed in current context
   * Override in subclasses for specific conditions
   */
  canExecute(_context: ExecutionContext, _options?: T): boolean {
    return true;
  }

  /**
   * Execute the command - must be implemented by subclasses
   */
  abstract execute(context: ExecutionContext, options?: T): Promise<void>;

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
      const message = error instanceof Error ? error.message : "Unknown error";
      const fullMessage = errorPrefix ? `${errorPrefix}: ${message}` : message;

      this.dialogService.showNotification(fullMessage, "error");
      throw error;
    }
  }

  /**
   * Show success notification
   */
  protected showSuccess(message: string): void {
    this.dialogService.showNotification(message, "success");
  }

  /**
   * Show info notification
   */
  protected showInfo(message: string): void {
    this.dialogService.showNotification(message, "info");
  }

  /**
   * Show warning notification
   */
  protected showWarning(message: string): void {
    this.dialogService.showNotification(message, "warning");
  }

  protected showNotification(message: string, type: NotificationType): void {
    this.dispatch(
      addNotification({
        id: `cmd-${Date.now()}`,
        message,
        type,
        autoClose: true,
        timestamp: Date.now(),
        duration: type === "success" ? 3000 : 5000,
      })
    );
  }
}
