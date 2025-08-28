import { BaseCommand } from './BaseCommand';
import { ExecutionContext } from '../types';
import { FileInfo } from '../../../types';

/**
 * Specialized base class for file operations
 * Provides common functionality for file manipulation commands
 */
export abstract class FileOperationCommand extends BaseCommand {
  /**
   * Check if command requires file selection
   */
  protected requiresSelection(context: ExecutionContext, minCount = 1): boolean {
    return context.selectedFiles.length >= minCount;
  }

  /**
   * Check if command can execute based on selection requirements
   */
  canExecute(context: ExecutionContext): boolean {
    if (!super.canExecute(context)) return false;
    return this.getRequiredSelectionCount() <= context.selectedFiles.length;
  }

  /**
   * Get the minimum number of files that must be selected
   * Override in subclasses
   */
  protected getRequiredSelectionCount(): number {
    return 0; // Most commands don't require selection
  }

  /**
   * Get the currently selected files as FileInfo objects
   */
  protected getSelectedFiles(context: ExecutionContext): FileInfo[] {
    return context.selectedFiles;
  }

  /**
   * Generate confirmation message for file operations
   */
  protected generateConfirmationMessage(
    operation: string,
    files: FileInfo[]
  ): string {
    if (files.length === 1) {
      return `Are you sure you want to ${operation} "${files[0].name}"?`;
    }
    return `Are you sure you want to ${operation} ${files.length} selected items?`;
  }

  /**
   * Execute operation with progress indication for multiple files
   */
  protected async executeWithProgress<T>(
    files: FileInfo[],
    operation: (file: FileInfo, index: number) => Promise<T>,
    operationName: string
  ): Promise<T[]> {
    const results: T[] = [];
    
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      
      // Show progress for operations with multiple files
      if (files.length > 1) {
        this.showInfo(`${operationName} ${file.name} (${i + 1}/${files.length})`);
      }
      
      const result = await operation(file, i);
      results.push(result);
    }
    
    return results;
  }

  /**
   * Validate that panel exists and is accessible
   */
  protected validatePanel(context: ExecutionContext): void {
    const panel = context.panels[context.panelId];
    if (!panel) {
      throw new Error(`Panel ${context.panelId} not found`);
    }
  }
}