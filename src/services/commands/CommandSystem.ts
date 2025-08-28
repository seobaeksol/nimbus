import { AppDispatch } from '../../store';
import { CommandContext } from '../../types/commands';
import { Command, ExecutionContext } from './types';
import { CommandRegistry } from './registry/CommandRegistry';
import { CommandFactory } from './factory/CommandFactory';
import { CommandExecutor } from '../commandExecutor';
import { BrowserDialogService, MockDialogService } from './services/DialogService';

// Modern Command System - no legacy support

/**
 * Central orchestrator for the command system
 * Manages initialization, execution, and coordination between command subsystems
 */
export class CommandSystem {
  private static instance: CommandSystem | null = null;
  private static initialized = false;
  
  private constructor(
    private dispatch: AppDispatch
  ) {}

  /**
   * Initialize the modern command system
   */
  static initialize(dispatch: AppDispatch): CommandSystem {
    if (!this.instance) {
      this.instance = new CommandSystem(dispatch);
      this.instance.initializeSubsystems();
      this.initialized = true;
    }
    return this.instance;
  }

  /**
   * Get the singleton instance
   */
  static getInstance(): CommandSystem | null {
    return this.instance;
  }

  /**
   * Check if system is initialized
   */
  static isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Initialize all command subsystems
   */
  private initializeSubsystems() {
    // Initialize CommandExecutor with context
    CommandExecutor.initialize({
      dispatch: this.dispatch,
      panels: {},
      activePanelId: null,
      clipboardState: null
    });

    // Initialize modern command registry
    CommandRegistry.initialize(this.dispatch);
  }

  /**
   * Get all available commands for the current context
   */
  getAvailableCommands(context: CommandContext): Command[] {
    return CommandRegistry.getAvailableCommands(context);
  }

  /**
   * Search commands across the system
   */
  searchCommands(searchTerm: string, context: CommandContext): Command[] {
    return CommandRegistry.searchCommands(searchTerm, context);
  }

  /**
   * Execute a command by ID
   */
  async executeCommand(commandId: string, context: CommandContext): Promise<boolean> {
    return CommandRegistry.executeCommand(commandId, context);
  }

  /**
   * Get commands organized by category
   */
  getCommandsByCategory(context: CommandContext): Map<string, Command[]> {
    const commands = this.getAvailableCommands(context);
    const categorized = new Map<string, Command[]>();
    
    commands.forEach(command => {
      const category = command.metadata.category;
      if (!categorized.has(category)) {
        categorized.set(category, []);
      }
      categorized.get(category)!.push(command);
    });
    
    return categorized;
  }

  /**
   * Get system statistics
   */
  getStats() {
    const modernStats = CommandRegistry.getStats();
    
    return {
      mode: 'modern',
      initialized: this.initialized,
      totalCommands: modernStats.totalCommands,
      categories: modernStats.categories,
      modernStats
    };
  }

  /**
   * Create a command factory for testing or custom command creation
   */
  createCommandFactory(useMockDialog = false): CommandFactory {
    const dialogService = useMockDialog 
      ? new MockDialogService()
      : new BrowserDialogService(this.dispatch);
    
    const executor = new CommandExecutor(); // Instance for factory
    return new CommandFactory(executor, dialogService);
  }

  /**
   * Convert between context formats
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
   * Get the current mode (always 'modern')
   */
  getMode(): string {
    return 'modern';
  }
}
