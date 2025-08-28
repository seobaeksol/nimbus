import { Command, ExecutionContext } from '../types';
import { AppDispatch } from '../../../store';
import { CommandRegistry } from '../registry/CommandRegistry';

/**
 * Central command service that replaces the static CommandExecutor approach
 * Provides a single entry point for all command operations
 */
export class CommandService {
  private static instance: CommandService | null = null;
  private dispatch: AppDispatch;

  private constructor(dispatch: AppDispatch) {
    this.dispatch = dispatch;
  }

  /**
   * Initialize the command service
   */
  static initialize(dispatch: AppDispatch): CommandService {
    if (!this.instance) {
      this.instance = new CommandService(dispatch);
      CommandRegistry.initialize(dispatch);
    }
    return this.instance;
  }

  /**
   * Get singleton instance
   */
  static getInstance(): CommandService {
    if (!this.instance) {
      throw new Error('CommandService not initialized. Call initialize() first.');
    }
    return this.instance;
  }

  /**
   * Execute a command by ID with context
   */
  async executeCommand(commandId: string, context: ExecutionContext): Promise<boolean> {
    return CommandRegistry.executeCommand(commandId, context);
  }

  /**
   * Get available commands for current context
   */
  getAvailableCommands(context: ExecutionContext): Command[] {
    return CommandRegistry.getAvailableCommands(context);
  }

  /**
   * Search commands with context filtering
   */
  searchCommands(searchTerm: string, context: ExecutionContext): Command[] {
    return CommandRegistry.searchCommands(searchTerm, context);
  }

  /**
   * Get commands organized by category
   */
  getCommandsByCategory(): Map<string, Command[]> {
    return CommandRegistry.getCommandsByCategory();
  }

  /**
   * Get command by ID
   */
  getCommand(id: string): Command | undefined {
    return CommandRegistry.getCommand(id);
  }

  /**
   * Check if a specific command can execute in current context
   */
  canExecuteCommand(commandId: string, context: ExecutionContext): boolean {
    const command = this.getCommand(commandId);
    if (!command) return false;

    return command.canExecute(context);
  }

  /**
   * Get system stats
   */
  getStats() {
    return CommandRegistry.getStats();
  }
}