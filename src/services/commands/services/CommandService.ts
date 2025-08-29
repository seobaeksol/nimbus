import { Command, ExecutionContext } from "../types";
import { AppDispatch } from "../../../store";
import { BrowserDialogService } from "./DialogService";
import { CommandFactory } from "../factory/CommandFactory";

/**
 * Central command service that replaces the static CommandExecutor approach
 * Provides a single entry point for all command operations
 */
export class CommandService {
  private commands: Map<string, Command<any>> = new Map();
  private commandFactory: CommandFactory | null = null;
  private static instance: CommandService | null = null;

  /**
   * Initialize the registry with dependency injection
   */
  static initialize(dispatch: AppDispatch): CommandService {
    if (this.instance) return this.instance;
    this.instance = new CommandService(dispatch);
    return this.instance;
  }

  private constructor(dispatch: AppDispatch) {
    // Create DialogService and CommandExecutorService
    const dialogService = new BrowserDialogService(dispatch);
    this.commandFactory = new CommandFactory(dispatch, dialogService);

    // Register all commands
    this.registerAllCommands();
    CommandService.instance = this;
  }

  /**
   * Register all commands from the factory
   */
  private registerAllCommands() {
    if (!this.commandFactory) {
      throw new Error("CommandFactory not initialized");
    }

    const allCommands = this.commandFactory.createAllCommands();
    allCommands.forEach((command) => {
      this.commands.set(command.metadata.id, command);
    });
  }

  /**
   * Get command by ID
   */
  getCommand(id: string): Command<any> | undefined {
    return this.commands.get(id);
  }
  /**
   * Get all registered commands
   */
  getAllCommands(): Command<any>[] {
    return Array.from(this.commands.values());
  }

  /**
   * Get commands organized by category
   */
  getCommandsByCategory(): Map<string, Command<any>[]> {
    if (!this.commandFactory) {
      throw new Error("CommandRegistry not initialized");
    }

    return this.commandFactory.createCommandsByCategory();
  }

  /**
   * Get available commands based on current context
   */
  getAvailableCommands(
    context: ExecutionContext,
    options?: Record<string, any>
  ): Command<any>[] {
    return this.getAllCommands().filter((command) => {
      return command.canExecute(context, options);
    });
  }

  /**
   * Search commands by term with context filtering
   */
  searchCommands(
    searchTerm: string,
    context: ExecutionContext,
    options?: Record<string, any>
  ): Command<any>[] {
    const availableCommands = this.getAvailableCommands(context, options);

    if (!searchTerm.trim()) {
      return availableCommands;
    }

    const term = searchTerm.toLowerCase();

    return availableCommands
      .filter((command) => {
        const metadata = command.metadata;
        const searchableText = [
          metadata.label,
          metadata.description || "",
          metadata.category,
          metadata.shortcut || "",
          metadata.id,
        ]
          .join(" ")
          .toLowerCase();

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
   * Execute a command by ID
   */
  async executeCommand(
    commandId: string,
    context: ExecutionContext,
    options?: Record<string, any>
  ): Promise<boolean> {
    const command = this.getCommand(commandId);
    if (!command) {
      console.error(`Command not found: ${commandId}`);
      return false;
    }

    if (!command.canExecute(context, options)) {
      console.warn(`Command cannot execute: ${commandId}`);
      return false;
    }

    try {
      await command.execute(context, options);
      return true;
    } catch (error) {
      console.error(`Command execution failed: ${commandId}`, error);
      return false;
    }
  }

  canExecuteCommand(
    commandId: string,
    context: ExecutionContext,
    options?: Record<string, any>
  ): boolean {
    const command = this.getCommand(commandId);
    if (!command) {
      console.error(`Command not found: ${commandId}`);
      return false;
    }
    return command.canExecute(context, options);
  }

  /**
   * Get command statistics
   */
  getStats() {
    return {
      totalCommands: this.commands.size,
      categories: [
        ...new Set(this.getAllCommands().map((cmd) => cmd.metadata.category)),
      ],
    };
  }
}
