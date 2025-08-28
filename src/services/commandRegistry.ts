import { Command as LegacyCommand, CommandContext } from '../types/commands';
import { Command as ModernCommand, ExecutionContext } from './commands/types';
import { ModernCommandRegistry } from './commands/registry/CommandRegistry';
import { AppDispatch } from '../store';

/**
 * Modern CommandRegistry - uses new command architecture exclusively
 * Replaces the legacy command system with dependency injection and better separation of concerns
 */
export class CommandRegistry {
  private static initialized = false;

  /**
   * Initialize the modern command system
   */
  static initialize(dispatch: AppDispatch) {
    if (this.initialized) return;
    
    ModernCommandRegistry.initialize(dispatch);
    this.initialized = true;
  }

  /**
   * Get command by ID
   */
  static getCommand(id: string): ModernCommand | undefined {
    return ModernCommandRegistry.getCommand(id);
  }

  /**
   * Get all registered commands
   */
  static getAllCommands(): ModernCommand[] {
    return ModernCommandRegistry.getAllCommands();
  }

  /**
   * Get available commands based on current context
   */
  static getAvailableCommands(context: CommandContext): ModernCommand[] {
    return ModernCommandRegistry.getAvailableCommands(context);
  }

  /**
   * Search commands by term with context filtering
   */
  static searchCommands(searchTerm: string, context: CommandContext): ModernCommand[] {
    return ModernCommandRegistry.searchCommands(searchTerm, context);
  }

  /**
   * Execute a command by ID
   */
  static async executeCommand(commandId: string, context: CommandContext): Promise<boolean> {
    return ModernCommandRegistry.executeCommand(commandId, context);
  }

  /**
   * Get commands organized by category
   */
  static getCommandsByCategory(): Map<string, ModernCommand[]> {
    return ModernCommandRegistry.getCommandsByCategory();
  }

  /**
   * Convert CommandContext to ExecutionContext for modern commands
   */
  static convertContext(context: CommandContext): ExecutionContext {
    return {
      panelId: context.activePanelId || '',
      currentPath: context.currentPath,
      selectedFiles: context.selectedFiles,
      dispatch: context.dispatch,
      clipboardHasFiles: context.clipboardHasFiles,
      panels: context.panels
    };
  }

  /**
   * Check if system is initialized
   */
  static isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Get system statistics
   */
  static getStats() {
    return ModernCommandRegistry.getStats();
  }

  // === LEGACY COMPATIBILITY METHODS ===
  // These methods maintain the same interface as the old CommandRegistry
  // but delegate to the modern system

  /**
   * Execute command using unified system (for backward compatibility)
   */
  static async executeCommandUnified(commandId: string, context: CommandContext): Promise<boolean> {
    return this.executeCommand(commandId, context);
  }

  /**
   * Get all commands unified (for backward compatibility)
   */
  static getAllCommandsUnified(context: CommandContext) {
    const modernCommands = this.getAvailableCommands(context);
    return {
      legacy: [], // No legacy commands
      modern: modernCommands,
      total: modernCommands.length
    };
  }
}