import { BaseCommand } from "./BaseCommand";
import { ExecutionContext } from "../types";
import { resolvePath } from "../ipc/file";
import { navigateToPath } from "@/store/slices/panelSlice";
/**
 * Specialized base class for navigation operations
 * Provides common functionality for directory navigation commands
 */
export abstract class NavigationCommand extends BaseCommand {
  /**
   * Navigate to path with error handling and validation
   */
  protected async navigateWithValidation(
    context: ExecutionContext,
    path: string
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      await this.navigateToPath(context.panelId, path);
    }, "Navigation failed");
  }

  async navigateToPath(panelId: string, inputPath: string): Promise<void> {
    const resolvedPath = await resolvePath(inputPath);

    this.dispatch(navigateToPath({ panelId, path: resolvedPath }));
  }

  /**
   * Validate that the current panel exists before navigation
   */
  protected validateNavigationContext(context: ExecutionContext): void {
    if (!context.panelId) {
      throw new Error("No active panel for navigation");
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
