// Core types matching the Rust backend

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  size_formatted: string;
  modified: string; // ISO timestamp
  created?: string;
  accessed?: string;
  file_type: "File" | "Directory" | "Symlink";
  extension?: string;
  permissions: FilePermissions;
  is_hidden: boolean;
  is_readonly: boolean;
}

export interface FilePermissions {
  read: boolean;
  write: boolean;
  execute: boolean;
}

export interface SystemInfo {
  platform: string;
  arch: string;
  hostname: string;
  username: string;
  home_dir?: string;
  temp_dir: string;
  app_name: string;
  app_version: string;
}

// Panel and layout types from architecture
export interface PanelState {
  id: string;
  tabs: TabState[];
  activeTabIndex: number;
  selection: Set<string>;
  viewMode: "list" | "grid" | "details";
  sortBy: string;
  sortOrder: "asc" | "desc";
  gridPosition?: GridPosition;
  isActive: boolean;
}

export interface TabState {
  id: string;
  title: string;
  path: string;
  files: FileInfo[];
  loading: boolean;
  error?: string;
  selection: Set<string>;
  sortBy: string;
  sortOrder: "asc" | "desc";
  viewMode: "list" | "grid" | "details";
  filter?: string;
  history: string[];
  historyIndex: number;
  scrollPosition: number;
  lastAccessed: Date;
  isPinned: boolean;
}

export interface PanelLayoutConfig {
  type: "single" | "dual" | "triple" | "grid2x2" | "grid2x3" | "grid3x2";
  splitterPositions: number[];
  gridDimensions?: {
    rows: number;
    cols: number;
    cellSpacing: number;
    uniformSizing: boolean;
  };
}

export interface GridPosition {
  row: number;
  col: number;
  rowSpan?: number;
  colSpan?: number;
}

export interface AppState {
  panels: PanelState[];
  activePanelId: string;
  layout: PanelLayoutConfig;
  globalSettings: any; // TODO: define settings structure
  connections: any[]; // TODO: define connection structure
  searchResults?: any; // TODO: define search results structure
}