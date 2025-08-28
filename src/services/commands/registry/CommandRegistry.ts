import { Command, ExecutionContext } from '../types';
import { CommandContext } from '../../types/commands';
import { CommandFactory } from '../factory/CommandFactory';
import { CommandExecutor } from '../../commandExecutor';
import { BrowserDialogService } from '../services/DialogService';
import { AppDispatch } from '../../../store';

/**
 * Modern command registry using the new command architecture
 * Provides discovery, filtering, and execution of commands with dependency injection
 */
export class ModernCommandRegistry {
  private static commands: Map<string, Command> = new Map();
  private static commandFactory: CommandFactory | null = null;
  private static initialized = false;

  /**
   * Initialize the registry with dependency injection
   */
  static initialize(dispatch: AppDispatch) {
    if (this.initialized) return;

    // Initialize CommandExecutor with required context
    CommandExecutor.initialize({
      dispatch,
      panels: {},
      activePanelId: null,
      clipboardState: null
    });

    // Create DialogService and CommandFactory
    const dialogService = new BrowserDialogService(dispatch);
    const executor = new CommandExecutor(); // Instance for factory
    this.commandFactory = new CommandFactory(executor, dialogService);

    // Register all commands
    this.registerAllCommands();
    this.initialized = true;
  }

  /**
   * Register all commands from the factory
   */
  private static registerAllCommands() {
    if (!this.commandFactory) {
      throw new Error('CommandFactory not initialized');
    }

    const allCommands = this.commandFactory.createAllCommands();
    allCommands.forEach(command => {
      this.commands.set(command.metadata.id, command);
    });
  }

  /**
   * Get command by ID
   */
  static getCommand(id: string): Command | undefined {
    return this.commands.get(id);
  }

  /**
   * Get all registered commands
   */
  static getAllCommands(): Command[] {
    return Array.from(this.commands.values());
  }

  /**
   * Get commands organized by category
   */
  static getCommandsByCategory(): Map<string, Command[]> {
    if (!this.commandFactory) {
      throw new Error('CommandRegistry not initialized');
    }

    return this.commandFactory.createCommandsByCategory();
  }

  /**
   * Get available commands based on current context
   */
  static getAvailableCommands(context: CommandContext): Command[] {
    const executionContext = this.convertContext(context);
    
    return this.getAllCommands().filter(command => {
      return command.canExecute(executionContext);
    });
  }

  /**
   * Search commands by term with context filtering
   */
  static searchCommands(searchTerm: string, context: CommandContext): Command[] {
    const availableCommands = this.getAvailableCommands(context);
    
    if (!searchTerm.trim()) {
      return availableCommands;
    }

    const term = searchTerm.toLowerCase();
    
    return availableCommands
      .filter(command => {
        const metadata = command.metadata;
        const searchableText = [
          metadata.label,
          metadata.description || '',
          metadata.category,
          metadata.shortcut || '',
          metadata.id
        ].join(' ').toLowerCase();
        
        return searchableText.includes(term);
      })
      .sort((a, b) => {
        // Prioritize exact matches in label
        const aLabelMatch = a.metadata.label.toLowerCase().includes(term);
        const bLabelMatch = b.metadata.label.toLowerCase().includes(term);
        
        if (aLabelMatch && !bLabelMatch) return -1;
        if (!aLabelMatch && bLabelMatch) return 1;
        
        // Then by category
        return a.metadata.category.localeCompare(b.metadata.category);
      });
  }

  /**
   * Execute a command by ID with proper context conversion
   */
  static async executeCommand(commandId: string, context: CommandContext): Promise<boolean> {
    const command = this.getCommand(commandId);
    if (!command) {
      console.error(`Command not found: ${commandId}`);
      return false;
    }

    const executionContext = this.convertContext(context);
    
    if (!command.canExecute(executionContext)) {
      console.warn(`Command cannot execute: ${commandId}`);
      return false;
    }

    try {
      await command.execute(executionContext);
      return true;
    } catch (error) {
      console.error(`Command execution failed: ${commandId}`, error);
      return false;
    }
  }

  /**
   * Convert old CommandContext to new ExecutionContext
   */
  private static convertContext(context: CommandContext): ExecutionContext {
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
   * Check if registry is initialized
   */
  static isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Get command statistics
   */
  static getStats() {
    return {
      totalCommands: this.commands.size,
      categories: [...new Set(this.getAllCommands().map(cmd => cmd.metadata.category))],
      initialized: this.initialized
    };
  }
}