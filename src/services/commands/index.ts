// Modern Command Architecture - Main Entry Point

// Core System
export { CommandSystem } from './CommandSystem';

// Registry
export { CommandRegistry as ModernCommandRegistry } from './registry/CommandRegistry';

// Factory
export { CommandFactory } from './factory/CommandFactory';

// Base Classes
export * from './base';

// Services
export { BrowserDialogService, MockDialogService } from './services/DialogService';
export type { DialogService, NotificationType } from './services/DialogService';

// Types
export type {
  Command,
  CommandMetadata,
  ExecutionContext,
  ViewMode,
  SortBy,
  GridLayoutConfig
} from './types';

// Command Implementations
export * from './implementations/file';
export * from './implementations/navigation';
export * from './implementations/view';
export * from './implementations/panel';
