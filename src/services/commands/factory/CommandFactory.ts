import { Command, DialogService, GridLayoutConfig, ViewMode, SortBy } from '../types';
import { CommandExecutor } from '../../commandExecutor';

// Import concrete command classes
import {
  CreateFileCommand,
  CreateFolderCommand,
  DeleteFilesCommand,
  RenameFileCommand,
  CopyFilesCommand,
  CutFilesCommand,
  PasteFilesCommand
} from '../implementations/file';

import {
  FocusAddressBarCommand,
  GoToHomeCommand,
  GoToDocumentsCommand,
  GoToDesktopCommand,
  GoToDownloadsCommand,
  GoToApplicationsCommand,
  GoToPathCommand
} from '../implementations/navigation';

import {
  SetViewModeCommand,
  SortByCommand
} from '../implementations/view';

import {
  SwitchPanelCommand,
  SetGridLayoutCommand
} from '../implementations/panel';

/**
 * Factory for creating command instances with proper dependency injection
 */
export class CommandFactory {
  constructor(
    private executor: CommandExecutor,
    private dialogService: DialogService
  ) {}

  /**
   * Create all file operation commands
   */
  createFileCommands(): Command[] {
    return [
      new CreateFileCommand(this.executor, this.dialogService),
      new CreateFolderCommand(this.executor, this.dialogService),
      new DeleteFilesCommand(this.executor, this.dialogService),
      new RenameFileCommand(this.executor, this.dialogService),
      new CopyFilesCommand(this.executor, this.dialogService),
      new CutFilesCommand(this.executor, this.dialogService),
      new PasteFilesCommand(this.executor, this.dialogService),
    ];
  }

  /**
   * Create all navigation commands
   */
  createNavigationCommands(): Command[] {
    return [
      new FocusAddressBarCommand(this.executor, this.dialogService),
      new GoToHomeCommand(this.executor, this.dialogService),
      new GoToDocumentsCommand(this.executor, this.dialogService),
      new GoToDesktopCommand(this.executor, this.dialogService),
      new GoToDownloadsCommand(this.executor, this.dialogService),
      new GoToApplicationsCommand(this.executor, this.dialogService),
      new GoToPathCommand(this.executor, this.dialogService),
    ];
  }

  /**
   * Create all view commands
   */
  createViewCommands(): Command[] {
    const viewModes: ViewMode[] = ['list', 'grid', 'details'];
    const sortOptions: SortBy[] = ['name', 'size', 'modified', 'type'];

    return [
      // View mode commands
      ...viewModes.map(mode => 
        new SetViewModeCommand(mode, this.executor, this.dialogService)
      ),
      // Sort commands
      ...sortOptions.map(sortBy =>
        new SortByCommand(sortBy, this.executor, this.dialogService)
      ),
    ];
  }

  /**
   * Create all panel management commands
   */
  createPanelCommands(): Command[] {
    const panelIds = ['panel-1', 'panel-2', 'panel-3', 'panel-4'];
    const layouts: GridLayoutConfig[] = [
      { rows: 1, cols: 1, name: '1x1 (Single Panel)' },
      { rows: 1, cols: 2, name: '1x2 (Classic Dual)' },
      { rows: 2, cols: 2, name: '2x2 (Quad)' },
      { rows: 2, cols: 3, name: '2x3 (Six Panel)' },
      { rows: 3, cols: 2, name: '3x2 (Vertical)' },
    ];

    return [
      // Panel switching commands (only for existing panels)
      ...panelIds.map(panelId =>
        new SwitchPanelCommand(panelId, this.executor, this.dialogService)
      ),
      // Layout commands
      ...layouts.map(layout =>
        new SetGridLayoutCommand(layout, this.executor, this.dialogService)
      ),
    ];
  }

  /**
   * Create all commands at once
   */
  createAllCommands(): Command[] {
    return [
      ...this.createFileCommands(),
      ...this.createNavigationCommands(),
      ...this.createViewCommands(),
      ...this.createPanelCommands(),
    ];
  }

  /**
   * Create commands by category
   */
  createCommandsByCategory(): Map<string, Command[]> {
    const commandsByCategory = new Map<string, Command[]>();
    
    commandsByCategory.set('File', this.createFileCommands());
    commandsByCategory.set('Navigation', this.createNavigationCommands());
    commandsByCategory.set('View', this.createViewCommands());
    commandsByCategory.set('Panel', this.createPanelCommands());
    
    return commandsByCategory;
  }
}