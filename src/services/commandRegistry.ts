import { Command, CommandContext } from '../types/commands';
import { 
  navigateToPath, 
  setActivePanel, 
  setGridLayout, 
  setViewMode, 
  setSorting
} from '../store/slices/panelSlice';
import { PathAliasService } from './pathAliasService';

export class CommandRegistry {
  private static commands: Map<string, Command> = new Map();
  private static initialized = false;

  static initialize() {
    if (this.initialized) return;
    
    this.registerFileCommands();
    this.registerNavigationCommands();
    this.registerPanelCommands();
    this.registerViewCommands();
    
    this.initialized = true;
  }

  private static registerFileCommands() {
    // Create New File
    this.register({
      id: 'file.create.file',
      label: 'File: Create New File',
      description: 'Create a new file with absolute or relative path support',
      category: 'File',
      icon: '📄',
      shortcut: 'Ctrl+T',
      action: async (context) => {
        // This will need to trigger the PromptDialog - handled in component
        const event = new CustomEvent('command-palette-create-file', { 
          detail: { context } 
        });
        window.dispatchEvent(event);
      }
    });

    // Create New Folder
    this.register({
      id: 'file.create.folder',
      label: 'File: Create New Folder',
      description: 'Create a new folder with absolute or relative path support',
      category: 'File',
      icon: '📁',
      shortcut: 'Ctrl+N',
      action: async (context) => {
        const event = new CustomEvent('command-palette-create-folder', { 
          detail: { context } 
        });
        window.dispatchEvent(event);
      }
    });

    // Rename Selected
    this.register({
      id: 'file.rename',
      label: 'File: Rename',
      description: 'Rename selected file or folder',
      category: 'File',
      icon: '✏️',
      shortcut: 'F2',
      when: (context) => context.selectedFiles.length === 1,
      action: async (context) => {
        const event = new CustomEvent('command-palette-rename-file', { 
          detail: { context, file: context.selectedFiles[0] } 
        });
        window.dispatchEvent(event);
      }
    });

    // Delete Selected
    this.register({
      id: 'file.delete',
      label: 'File: Delete Selected',
      description: 'Delete selected files and folders',
      category: 'File',
      icon: '🗑️',
      shortcut: 'Delete',
      when: (context) => context.selectedFiles.length > 0,
      action: async (context) => {
        const event = new CustomEvent('command-palette-delete-files', { 
          detail: { context, files: context.selectedFiles } 
        });
        window.dispatchEvent(event);
      }
    });

    // Copy Selected
    this.register({
      id: 'file.copy',
      label: 'File: Copy Selected',
      description: 'Copy selected files to clipboard',
      category: 'File',
      icon: '📋',
      shortcut: 'Ctrl+C',
      when: (context) => context.selectedFiles.length > 0,
      action: async (context) => {
        const event = new CustomEvent('command-palette-copy-files', { 
          detail: { context, files: context.selectedFiles } 
        });
        window.dispatchEvent(event);
      }
    });

    // Cut Selected
    this.register({
      id: 'file.cut',
      label: 'File: Cut Selected',
      description: 'Cut selected files to clipboard',
      category: 'File',
      icon: '✂️',
      shortcut: 'Ctrl+X',
      when: (context) => context.selectedFiles.length > 0,
      action: async (context) => {
        const event = new CustomEvent('command-palette-cut-files', { 
          detail: { context, files: context.selectedFiles } 
        });
        window.dispatchEvent(event);
      }
    });

    // Paste
    this.register({
      id: 'file.paste',
      label: 'File: Paste',
      description: 'Paste files from clipboard',
      category: 'File',
      icon: '📄',
      shortcut: 'Ctrl+V',
      when: (context) => context.clipboardHasFiles,
      action: async (context) => {
        const event = new CustomEvent('command-palette-paste-files', { 
          detail: { context } 
        });
        window.dispatchEvent(event);
      }
    });
  }

  private static registerNavigationCommands() {
    // Focus Address Bar
    this.register({
      id: 'navigation.address-bar',
      label: 'Go: Address Bar',
      description: 'Focus the address bar for navigation',
      category: 'Navigation',
      icon: '🎯',
      shortcut: 'Ctrl+L',
      action: async (context) => {
        const event = new CustomEvent('command-palette-focus-address-bar');
        window.dispatchEvent(event);
      }
    });

    // Navigate to Home
    this.register({
      id: 'navigation.home',
      label: 'Go: Home Directory',
      description: 'Navigate to home directory',
      category: 'Navigation',
      icon: '🏠',
      action: async (context) => {
        if (!context.activePanelId) return;
        const homePath = await PathAliasService.resolvePath('~');
        context.dispatch(navigateToPath({ 
          panelId: context.activePanelId, 
          path: homePath 
        }));
      }
    });

    // Navigate to Documents
    this.register({
      id: 'navigation.documents',
      label: 'Go: Documents',
      description: 'Navigate to Documents folder',
      category: 'Navigation',
      icon: '📄',
      action: async (context) => {
        if (!context.activePanelId) return;
        const documentsPath = await PathAliasService.resolvePath('Documents');
        context.dispatch(navigateToPath({ 
          panelId: context.activePanelId, 
          path: documentsPath 
        }));
      }
    });

    // Navigate to Downloads
    this.register({
      id: 'navigation.downloads',
      label: 'Go: Downloads',
      description: 'Navigate to Downloads folder',
      category: 'Navigation',
      icon: '⬇️',
      action: async (context) => {
        if (!context.activePanelId) return;
        const downloadsPath = await PathAliasService.resolvePath('Downloads');
        context.dispatch(navigateToPath({ 
          panelId: context.activePanelId, 
          path: downloadsPath 
        }));
      }
    });

    // Navigate to Desktop
    this.register({
      id: 'navigation.desktop',
      label: 'Go: Desktop',
      description: 'Navigate to Desktop folder',
      category: 'Navigation',
      icon: '🖥️',
      action: async (context) => {
        if (!context.activePanelId) return;
        const desktopPath = await PathAliasService.resolvePath('Desktop');
        context.dispatch(navigateToPath({ 
          panelId: context.activePanelId, 
          path: desktopPath 
        }));
      }
    });

    // Navigate to Path
    this.register({
      id: 'navigation.goto-path',
      label: 'Go: To Path...',
      description: 'Navigate to a specific path',
      category: 'Navigation',
      icon: '📁',
      action: async (context) => {
        const event = new CustomEvent('command-palette-goto-path', { 
          detail: { context } 
        });
        window.dispatchEvent(event);
      }
    });
  }

  private static registerPanelCommands() {
    // Switch to Panel 1
    this.register({
      id: 'panel.switch.1',
      label: 'Panel: Switch to Panel 1',
      description: 'Activate the first panel',
      category: 'Panel',
      icon: '1️⃣',
      when: (context) => 'panel-1' in context.panels,
      action: async (context) => {
        context.dispatch(setActivePanel('panel-1'));
      }
    });

    // Switch to Panel 2
    this.register({
      id: 'panel.switch.2',
      label: 'Panel: Switch to Panel 2',
      description: 'Activate the second panel',
      category: 'Panel',
      icon: '2️⃣',
      when: (context) => 'panel-2' in context.panels,
      action: async (context) => {
        context.dispatch(setActivePanel('panel-2'));
      }
    });

    // Layout Single
    this.register({
      id: 'panel.layout.single',
      label: 'Panel: Single Layout (1x1)',
      description: 'Switch to single panel layout',
      category: 'Panel',
      icon: '⬜',
      action: async (context) => {
        context.dispatch(setGridLayout({ rows: 1, cols: 1, name: '1x1 (Single Panel)' }));
      }
    });

    // Layout Dual
    this.register({
      id: 'panel.layout.dual',
      label: 'Panel: Dual Layout (1x2)',
      description: 'Switch to dual panel layout',
      category: 'Panel',
      icon: '▤',
      action: async (context) => {
        context.dispatch(setGridLayout({ rows: 1, cols: 2, name: '1x2 (Classic Dual)' }));
      }
    });

    // Layout Quad
    this.register({
      id: 'panel.layout.quad',
      label: 'Panel: Quad Layout (2x2)',
      description: 'Switch to four panel layout',
      category: 'Panel',
      icon: '▦',
      action: async (context) => {
        context.dispatch(setGridLayout({ rows: 2, cols: 2, name: '2x2 (Quad)' }));
      }
    });
  }

  private static registerViewCommands() {
    // View List Mode
    this.register({
      id: 'view.mode.list',
      label: 'View: List Mode',
      description: 'Switch to list view',
      category: 'View',
      icon: '📋',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setViewMode({ 
          panelId: context.activePanelId, 
          viewMode: 'list' 
        }));
      }
    });

    // View Grid Mode
    this.register({
      id: 'view.mode.grid',
      label: 'View: Grid Mode',
      description: 'Switch to grid view',
      category: 'View',
      icon: '⊞',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setViewMode({ 
          panelId: context.activePanelId, 
          viewMode: 'grid' 
        }));
      }
    });

    // View Details Mode
    this.register({
      id: 'view.mode.details',
      label: 'View: Details Mode',
      description: 'Switch to details view',
      category: 'View',
      icon: '📊',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setViewMode({ 
          panelId: context.activePanelId, 
          viewMode: 'details' 
        }));
      }
    });

    // Sort by Name
    this.register({
      id: 'view.sort.name',
      label: 'View: Sort by Name',
      description: 'Sort files by name',
      category: 'View',
      icon: '🔤',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setSorting({ 
          panelId: context.activePanelId, 
          sortBy: 'name' 
        }));
      }
    });

    // Sort by Size
    this.register({
      id: 'view.sort.size',
      label: 'View: Sort by Size',
      description: 'Sort files by size',
      category: 'View',
      icon: '📏',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setSorting({ 
          panelId: context.activePanelId, 
          sortBy: 'size' 
        }));
      }
    });

    // Sort by Date
    this.register({
      id: 'view.sort.modified',
      label: 'View: Sort by Date Modified',
      description: 'Sort files by modification date',
      category: 'View',
      icon: '📅',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setSorting({ 
          panelId: context.activePanelId, 
          sortBy: 'modified' 
        }));
      }
    });

    // Sort by Type
    this.register({
      id: 'view.sort.type',
      label: 'View: Sort by Type',
      description: 'Sort files by type',
      category: 'View',
      icon: '🏷️',
      action: async (context) => {
        if (!context.activePanelId) return;
        context.dispatch(setSorting({ 
          panelId: context.activePanelId, 
          sortBy: 'type' 
        }));
      }
    });
  }

  static register(command: Command) {
    this.commands.set(command.id, command);
  }

  static getCommand(id: string): Command | undefined {
    return this.commands.get(id);
  }

  static getAllCommands(): Command[] {
    return Array.from(this.commands.values());
  }

  static getAvailableCommands(context: CommandContext): Command[] {
    return this.getAllCommands().filter(command => {
      // Check if command should be available in current context
      return !command.when || command.when(context);
    });
  }

  static searchCommands(searchTerm: string, context: CommandContext): Command[] {
    if (!searchTerm.trim()) {
      return this.getAvailableCommands(context);
    }

    const term = searchTerm.toLowerCase();
    const availableCommands = this.getAvailableCommands(context);

    return availableCommands
      .filter(command => {
        const searchableText = [
          command.label,
          command.description || '',
          command.category,
          command.shortcut || ''
        ].join(' ').toLowerCase();
        
        return searchableText.includes(term);
      })
      .sort((a, b) => {
        // Prioritize exact matches in label
        const aLabelMatch = a.label.toLowerCase().includes(term);
        const bLabelMatch = b.label.toLowerCase().includes(term);
        
        if (aLabelMatch && !bLabelMatch) return -1;
        if (!aLabelMatch && bLabelMatch) return 1;
        
        // Then by category
        return a.category.localeCompare(b.category);
      });
  }
}