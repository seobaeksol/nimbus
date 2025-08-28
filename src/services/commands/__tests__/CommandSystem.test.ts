import { CommandSystem } from '../CommandSystem';
import { CommandRegistry } from '../registry/CommandRegistry';
import { HybridCommandRegistry } from '../registry/LegacyCommandAdapter';
import { CommandExecutor } from '../../commandExecutor';
import { CommandContext } from '../../../types/commands';

// Mock dependencies
jest.mock('../registry/CommandRegistry');
jest.mock('../registry/LegacyCommandAdapter');
jest.mock('../../commandExecutor');

describe('CommandSystem', () => {
  let mockDispatch: jest.Mock;
  let mockContext: CommandContext;

  beforeEach(() => {
    mockDispatch = jest.fn();
    mockContext = {
      activePanelId: 'panel-1',
      currentPath: '/test/path',
      selectedFiles: [],
      dispatch: mockDispatch,
      panels: { 'panel-1': { id: 'panel-1', path: '/test/path' } },
      clipboardHasFiles: false
    };

    // Reset singleton
    (CommandSystem as any).instance = null;
    (CommandSystem as any).initialized = false;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize in hybrid mode by default', () => {
      const system = CommandSystem.initialize(mockDispatch);
      
      expect(CommandExecutor.initialize).toHaveBeenCalled();
      expect(CommandRegistry.initialize).toHaveBeenCalledWith(mockDispatch);
      expect(HybridCommandRegistry.initialize).toHaveBeenCalledWith(mockDispatch);
      expect(system.getMode()).toBe('hybrid');
    });

    it('should initialize in modern mode when specified', () => {
      const system = CommandSystem.initialize(mockDispatch, 'modern');
      
      expect(CommandRegistry.initialize).toHaveBeenCalledWith(mockDispatch);
      expect(system.getMode()).toBe('modern');
    });

    it('should return singleton instance on subsequent calls', () => {
      const system1 = CommandSystem.initialize(mockDispatch);
      const system2 = CommandSystem.initialize(mockDispatch);
      
      expect(system1).toBe(system2);
    });

    it('should track initialization state', () => {
      expect(CommandSystem.isInitialized()).toBe(false);
      
      CommandSystem.initialize(mockDispatch);
      
      expect(CommandSystem.isInitialized()).toBe(true);
    });
  });

  describe('Command Operations', () => {
    let system: CommandSystem;

    beforeEach(() => {
      system = CommandSystem.initialize(mockDispatch, 'hybrid');
    });

    it('should delegate command search to appropriate registry', () => {
      const mockCommands = [{ metadata: { id: 'test', category: 'Test' } }];
      (HybridCommandRegistry.searchCommands as jest.Mock).mockReturnValue(mockCommands);
      
      const result = system.searchCommands('test', mockContext);
      
      expect(HybridCommandRegistry.searchCommands).toHaveBeenCalledWith('test', mockContext);
      expect(result).toBe(mockCommands);
    });

    it('should delegate command execution to appropriate registry', async () => {
      (HybridCommandRegistry.executeCommand as jest.Mock).mockResolvedValue(true);
      
      const result = await system.executeCommand('test-command', mockContext);
      
      expect(HybridCommandRegistry.executeCommand).toHaveBeenCalledWith('test-command', mockContext);
      expect(result).toBe(true);
    });

    it('should organize commands by category', () => {
      const mockCommands = [
        { metadata: { id: 'file1', category: 'File' } },
        { metadata: { id: 'file2', category: 'File' } },
        { metadata: { id: 'nav1', category: 'Navigation' } }
      ];
      
      jest.spyOn(system, 'getAvailableCommands').mockReturnValue(mockCommands as any);
      
      const categorized = system.getCommandsByCategory(mockContext);
      
      expect(categorized.get('File')).toHaveLength(2);
      expect(categorized.get('Navigation')).toHaveLength(1);
    });
  });

  describe('Mode Switching', () => {
    let system: CommandSystem;

    beforeEach(() => {
      system = CommandSystem.initialize(mockDispatch, 'hybrid');
    });

    it('should switch modes and reinitialize', () => {
      expect(system.getMode()).toBe('hybrid');
      
      system.switchMode('modern');
      
      expect(system.getMode()).toBe('modern');
    });
  });

  describe('Factory Creation', () => {
    let system: CommandSystem;

    beforeEach(() => {
      system = CommandSystem.initialize(mockDispatch);
    });

    it('should create command factory with browser dialog service', () => {
      const factory = system.createCommandFactory(false);
      
      expect(factory).toBeDefined();
    });

    it('should create command factory with mock dialog service for testing', () => {
      const factory = system.createCommandFactory(true);
      
      expect(factory).toBeDefined();
    });
  });

  describe('Context Conversion', () => {
    it('should convert CommandContext to ExecutionContext', () => {
      const executionContext = CommandSystem.convertContext(mockContext);
      
      expect(executionContext).toEqual({
        panelId: 'panel-1',
        currentPath: '/test/path',
        selectedFiles: [],
        dispatch: mockDispatch,
        clipboardHasFiles: false,
        panels: { 'panel-1': { id: 'panel-1', path: '/test/path' } }
      });
    });
  });

  describe('Statistics', () => {
    let system: CommandSystem;

    beforeEach(() => {
      system = CommandSystem.initialize(mockDispatch);
    });

    it('should return system statistics', () => {
      const mockStats = {
        totalCommands: 18,
        categories: ['File', 'Navigation', 'View', 'Panel'],
        initialized: true
      };
      
      (CommandRegistry.getStats as jest.Mock).mockReturnValue(mockStats);
      
      const stats = system.getStats();
      
      expect(stats.mode).toBe('hybrid');
      expect(stats.initialized).toBe(true);
      expect(stats.subsystems.modern).toBe(mockStats);
    });
  });
});
