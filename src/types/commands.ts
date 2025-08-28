import { AppDispatch } from '../store';
import { FileInfo } from './index';
import { Panel } from '../store/slices/panelSlice';

export interface CommandContext {
  activePanelId: string | null;
  selectedFiles: FileInfo[];
  currentPath: string;
  dispatch: AppDispatch;
  panels: Record<string, Panel>;
  clipboardHasFiles: boolean;
}

export interface Command {
  id: string;
  label: string;
  description?: string;
  category: 'File' | 'Navigation' | 'Panel' | 'View' | 'System';
  icon?: string;
  shortcut?: string;
  action: (context: CommandContext) => void | Promise<void>;
  when?: (context: CommandContext) => boolean; // conditional availability
}

export interface CommandPaletteState {
  isOpen: boolean;
  searchTerm: string;
  selectedIndex: number;
  recentCommands: string[]; // command IDs
}