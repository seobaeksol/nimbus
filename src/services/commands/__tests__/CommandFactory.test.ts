import { CommandFactory } from '../factory/CommandFactory';
import { CommandExecutor } from '../../commandExecutor';
import { MockDialogService } from '../services/DialogService';
import { ExecutionContext } from '../types';

describe('CommandFactory', () => {
  let factory: CommandFactory;
  let mockDialogService: MockDialogService;
  let executor: CommandExecutor;
  let mockContext: ExecutionContext;

  beforeEach(() => {
    mockDialogService = new MockDialogService();
    executor = new CommandExecutor();
    factory = new CommandFactory(executor, mockDialogService);
    
    mockContext = {
      panelId: 'panel-1',
      currentPath: '/test/path',
      selectedFiles: [],
      dispatch: jest.fn(),
      clipboardHasFiles: false,
      panels: { 'panel-1': { id: 'panel-1', path: '/test/path' } }
    };
  });

  describe('File Commands', () => {
    it('should create all file operation commands', () => {
      const fileCommands = factory.createFileCommands();
      
      expect(fileCommands).toHaveLength(7);
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('create-file');
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('create-folder');
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('delete-files');
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('rename-file');
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('copy-files');
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('cut-files');
      expect(fileCommands.map(cmd => cmd.metadata.id)).toContain('paste-files');
    });

    it('should create commands with proper metadata', () => {
      const fileCommands = factory.createFileCommands();
      const createFileCommand = fileCommands.find(cmd => cmd.metadata.id === 'create-file');
      
      expect(createFileCommand).toBeDefined();
      expect(createFileCommand!.metadata).toEqual({
        id: 'create-file',
        label: 'New File',
        category: 'File',
        description: 'Create a new file in the current directory',
        icon: 'file-plus',
        shortcut: 'Ctrl+N'
      });
    });
  });

  describe('Navigation Commands', () => {
    it('should create all navigation commands', () => {
      const navCommands = factory.createNavigationCommands();
      
      expect(navCommands).toHaveLength(7);
      expect(navCommands.map(cmd => cmd.metadata.id)).toContain('focus-address-bar');
      expect(navCommands.map(cmd => cmd.metadata.id)).toContain('go-to-home');
      expect(navCommands.map(cmd => cmd.metadata.id)).toContain('go-to-documents');
    });
  });

  describe('View Commands', () => {
    it('should create view mode and sort commands', () => {
      const viewCommands = factory.createViewCommands();
      
      // Should include 3 view modes + 4 sort options = 7 commands
      expect(viewCommands.length).toBeGreaterThanOrEqual(7);
      
      const viewModeIds = viewCommands
        .filter(cmd => cmd.metadata.id.startsWith('set-view-'))
        .map(cmd => cmd.metadata.id);
      
      expect(viewModeIds).toContain('set-view-list');
      expect(viewModeIds).toContain('set-view-grid');
      expect(viewModeIds).toContain('set-view-details');
    });
  });

  describe('Panel Commands', () => {
    it('should create panel switching and layout commands', () => {
      const panelCommands = factory.createPanelCommands();
      
      // Should include 4 panel switches + 5 layouts = 9 commands
      expect(panelCommands.length).toBeGreaterThanOrEqual(9);
      
      const layoutIds = panelCommands
        .filter(cmd => cmd.metadata.id.startsWith('set-layout-'))
        .map(cmd => cmd.metadata.id);
      
      expect(layoutIds).toContain('set-layout-1x1');
      expect(layoutIds).toContain('set-layout-1x2');
      expect(layoutIds).toContain('set-layout-2x2');
    });
  });

  describe('All Commands', () => {
    it('should create all commands without duplicates', () => {
      const allCommands = factory.createAllCommands();
      const commandIds = allCommands.map(cmd => cmd.metadata.id);
      const uniqueIds = [...new Set(commandIds)];
      
      expect(commandIds).toHaveLength(uniqueIds.length);
      expect(allCommands.length).toBeGreaterThanOrEqual(18);
    });

    it('should create commands by category map', () => {
      const commandsByCategory = factory.createCommandsByCategory();
      
      expect(commandsByCategory.has('File')).toBe(true);
      expect(commandsByCategory.has('Navigation')).toBe(true);
      expect(commandsByCategory.has('View')).toBe(true);
      expect(commandsByCategory.has('Panel')).toBe(true);
      
      expect(commandsByCategory.get('File')).toHaveLength(7);
      expect(commandsByCategory.get('Navigation')).toHaveLength(7);
    });
  });

  describe('Dependency Injection', () => {
    it('should inject dependencies correctly', () => {
      const fileCommands = factory.createFileCommands();
      const createFileCommand = fileCommands[0];
      
      // Commands should be able to execute (dependencies injected)
      expect(createFileCommand.canExecute(mockContext)).toBe(true);
    });

    it('should use provided dialog service', async () => {
      mockDialogService.queuePromptResponse('test-file.txt');
      
      const fileCommands = factory.createFileCommands();
      const createFileCommand = fileCommands.find(cmd => cmd.metadata.id === 'create-file');
      
      // Mock CommandExecutor.createFile
      const createFileSpy = jest.spyOn(CommandExecutor, 'createFile').mockResolvedValue();
      
      await createFileCommand!.execute(mockContext);
      
      expect(createFileSpy).toHaveBeenCalledWith('panel-1', 'test-file.txt');
      createFileSpy.mockRestore();
    });
  });
});