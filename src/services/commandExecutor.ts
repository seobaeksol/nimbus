/**
 * Modern Command System - Primary Exports
 * 
 * This file provides the main exports for the Command system architecture.
 * All command operations use the modern Command pattern with dependency injection.
 */

// Primary Command Services
export { CommandService } from './commands/services/CommandService';

// Core Command System Components
export { CommandFactory } from './commands/factory/CommandFactory';

// Base Classes for Creating Custom Commands
export { BaseCommand } from './commands/base/BaseCommand';
export { FileOperationCommand } from './commands/base/FileOperationCommand';

// Dialog Services
export { BrowserDialogService } from './commands/services/DialogService';
export type { DialogService, NotificationType } from './commands/services/DialogService';

// Core Types and Interfaces
export type { 
  Command, 
  CommandMetadata, 
  ExecutionContext,
  ViewMode,
  SortBy,
  GridLayoutConfig,
  Panel
} from './commands/types';

// Command Implementations
export * from './commands/implementations/file';
export * from './commands/implementations/navigation';
export * from './commands/implementations/view';
export * from './commands/implementations/panel';

/**
 * Re-export types for compatibility with existing imports
 */
export interface CommandOptions {
  showNotifications?: boolean;
  navigateToTarget?: boolean;
  refreshPanels?: boolean;
}