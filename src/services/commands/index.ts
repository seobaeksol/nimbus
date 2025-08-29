// Modern Command Architecture - Complete Command System

// Core System

// Main Services (Primary API)
export { CommandService } from './services/CommandService';

// Registry and Factory
export { CommandFactory } from './factory/CommandFactory';

// Base Classes for Extension
export * from './base';

// Dialog Services
export { BrowserDialogService } from './services/DialogService';
export type { DialogService, NotificationType } from './services/DialogService';

// Core Types
export type {
  Command,
  CommandMetadata,
  ExecutionContext,
  ViewMode,
  SortBy,
  GridLayoutConfig,
  Panel
} from './types';

// Command Implementations
export * from './implementations/file';
export * from './implementations/navigation';
export * from './implementations/view';
export * from './implementations/panel';

// Utility Exports - Note: useCommands hook is in src/hooks/useCommands.ts
// Import directly: import { useCommands } from '../../hooks/useCommands';
