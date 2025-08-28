import { AppDispatch } from '../../store';
import { FileInfo } from '../../types';

/**
 * Execution context for commands - contains all necessary state
 * for command execution without UI concerns
 */
export interface ExecutionContext {
  panelId: string;
  currentPath: string;
  selectedFiles: FileInfo[];
  dispatch: AppDispatch;
  clipboardHasFiles: boolean;
  panels: Record<string, any>; // Panel state for complex operations
}

/**
 * Command metadata for display and organization
 */
export interface CommandMetadata {
  id: string;
  label: string;
  description?: string;
  category: string;
  icon?: string;
  shortcut?: string;
}

/**
 * Core command interface - encapsulates metadata and execution logic
 */
export interface Command {
  readonly metadata: CommandMetadata;
  canExecute(context: ExecutionContext): boolean;
  execute(context: ExecutionContext): Promise<void>;
}

/**
 * Dialog service for user interactions - separates UI concerns from business logic
 */
export interface DialogService {
  prompt(message: string, defaultValue?: string): Promise<string | null>;
  confirm(message: string): Promise<boolean>;
  showNotification(message: string, type: NotificationType): void;
}

/**
 * Notification types
 */
export type NotificationType = 'success' | 'error' | 'warning' | 'info';

/**
 * Grid layout configuration
 */
export interface GridLayoutConfig {
  rows: number;
  cols: number;
  name: string;
}

/**
 * View mode options
 */
export type ViewMode = 'list' | 'grid' | 'details';

/**
 * Sort options
 */
export type SortBy = 'name' | 'size' | 'modified' | 'type';