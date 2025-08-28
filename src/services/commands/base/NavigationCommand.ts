import { BaseCommand } from './BaseCommand';
import { ExecutionContext } from '../types';
import { CommandExecutorService } from '../services/CommandExecutorService';

/**
 * Specialized base class for navigation operations
 * Provides common functionality for directory navigation commands
 */
export abstract class NavigationCommand extends BaseCommand {
  
  protected constructor(
    metadata: any,
    protected executor: CommandExecutorService,
    protected dialogService: any
  ) {
    super(metadata);
  }
  /**
   * Navigate to path with error handling and validation
   */
  protected async navigateWithValidation(
    context: ExecutionContext,
    path: string
  ): Promise<void> {
    await this.withErrorHandling(
      async () => {
        await this.executor.navigateToPath(context.panelId, path);
      },
      'Navigation failed'
    );
  }

  /**
   * Navigate using CommandExecutor methods with error handling
   */
  protected async navigateUsingMethod(
    context: ExecutionContext,
    method: (panelId: string) => Promise<void>,
    methodName: string
  ): Promise<void> {
    await this.withErrorHandling(
      async () => {
        await method(context.panelId);
      },
      `${methodName} failed`
    );
  }

  /**
   * Validate that the current panel exists before navigation
   */
  protected validateNavigationContext(context: ExecutionContext): void {
    if (!context.panelId) {
      throw new Error('No active panel for navigation');
    }

    const panel = context.panels[context.panelId];
    if (!panel) {
      throw new Error(`Panel ${context.panelId} not found`);
    }
  }

  /**
   * Check if command can execute - navigation requires an active panel
   */
  canExecute(context: ExecutionContext): boolean {
    if (!super.canExecute(context)) return false;
    
    try {
      this.validateNavigationContext(context);
      return true;
    } catch {
      return false;
    }
  }
}