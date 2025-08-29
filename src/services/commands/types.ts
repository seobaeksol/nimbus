import { R } from "vitest/dist/chunks/environment.d.cL3nLXbE.js";
import { AppDispatch } from "../../store";
import { FileInfo } from "./ipc/file";

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
  panels: Record<string, Panel>; // Panel state for complex operations
  clipboardState: {
    hasFiles: boolean;
    files: FileInfo[];
    operation: "copy" | "cut" | null;
    sourcePanelId: string | null;
  };
}

/**
 * Panel interface for better type safety
 */
export interface Panel {
  id: string;
  currentPath: string;
  files: FileInfo[];
  selectedFiles: string[];
  viewMode: ViewMode;
  sortBy: SortBy;
  sortOrder: "asc" | "desc";
  isLoading: boolean;
  error: string | null;
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
export interface Command<T extends Record<string, any> = {}> {
  readonly metadata: CommandMetadata;
  canExecute(context: ExecutionContext, options?: T): boolean;
  execute(context: ExecutionContext, options?: T): Promise<void>;
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
export type NotificationType = "success" | "error" | "warning" | "info";

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
export type ViewMode = "list" | "grid" | "details";

/**
 * Sort options
 */
export type SortBy = "name" | "size" | "modified" | "type";
