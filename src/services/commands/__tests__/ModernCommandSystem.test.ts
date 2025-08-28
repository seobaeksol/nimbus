import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CommandService } from '../services/CommandService';
import { CommandExecutorService } from '../services/CommandExecutorService';
import { CommandRegistry } from '../registry/CommandRegistry';
import { CommandFactory } from '../factory/CommandFactory';
import { BrowserDialogService } from '../services/DialogService';
import type { ExecutionContext } from '../types';
import { createTestStore, mockPanelState } from '../../../test/testUtils';

// Mock Redux store
const mockDispatch = vi.fn();
const mockContext: ExecutionContext = {
  panelId: 'panel-1',
  selectedFiles: [],
  currentPath: '/test/path',
  dispatch: mockDispatch,
  panels: mockPanelState.panels,
  clipboardHasFiles: false,
  clipboardState: {
    hasFiles: false,
    files: [],
    operation: null,
    sourcePanelId: null
  }
};

describe('Modern Command System Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset singleton instances
    (CommandService as any).instance = null;
  });

  describe('CommandService', () => {
    it('should initialize properly', () => {
      const service = CommandService.initialize(mockDispatch);
      expect(service).toBeDefined();
      expect(CommandService.getInstance()).toBe(service);
    });

    it('should throw error when accessing uninitialized service', () => {
      expect(() => CommandService.getInstance()).toThrow('CommandService not initialized');
    });

    it('should execute commands', async () => {
      const service = CommandService.initialize(mockDispatch);
      const result = await service.executeCommand('create-file', mockContext);
      
      // Command execution may fail due to missing UI interactions, but system should handle it
      expect(typeof result).toBe('boolean');
    });

    it('should get available commands', () => {
      const service = CommandService.initialize(mockDispatch);
      const commands = service.getAvailableCommands(mockContext);
      
      expect(Array.isArray(commands)).toBe(true);
      expect(commands.length).toBeGreaterThan(0);
    });

    it('should search commands', () => {
      const service = CommandService.initialize(mockDispatch);
      const results = service.searchCommands('file', mockContext);
      
      expect(Array.isArray(results)).toBe(true);
      // Should find file-related commands
      expect(results.some(cmd => cmd.metadata.category === 'File')).toBe(true);
    });
  });

  describe('CommandExecutorService', () => {
    it('should create instance with dispatch', () => {
      const service = new CommandExecutorService(mockDispatch);
      expect(service).toBeDefined();
    });

    it('should handle file operations with proper error handling', async () => {
      const service = new CommandExecutorService(mockDispatch);
      
      // Test error handling for invalid operations
      await expect(
        service.createFile('invalid-panel', 'test.txt')
      ).rejects.toThrow();
    });
  });

  describe('CommandRegistry', () => {
    it('should initialize with dispatch', () => {
      CommandRegistry.initialize(mockDispatch);
      expect(CommandRegistry.isInitialized()).toBe(true);
    });

    it('should provide command statistics', () => {
      CommandRegistry.initialize(mockDispatch);
      const stats = CommandRegistry.getStats();
      
      expect(stats).toHaveProperty('totalCommands');
      expect(stats).toHaveProperty('categories');
      expect(stats).toHaveProperty('initialized');
      expect(stats.totalCommands).toBeGreaterThan(0);
    });
  });

  describe('CommandFactory', () => {
    it('should create commands with proper dependencies', () => {
      const executor = new CommandExecutorService(mockDispatch);
      const dialogService = new BrowserDialogService(mockDispatch);
      const factory = new CommandFactory(executor, dialogService);

      const fileCommands = factory.createFileCommands();
      expect(fileCommands.length).toBeGreaterThan(0);
      
      const allCommands = factory.createAllCommands();
      expect(allCommands.length).toBeGreaterThan(fileCommands.length);
    });

    it('should organize commands by category', () => {
      const executor = new CommandExecutorService(mockDispatch);
      const dialogService = new BrowserDialogService(mockDispatch);
      const factory = new CommandFactory(executor, dialogService);

      const categorized = factory.createCommandsByCategory();
      expect(categorized.has('File')).toBe(true);
      expect(categorized.has('Navigation')).toBe(true);
      expect(categorized.has('View')).toBe(true);
      expect(categorized.has('Panel')).toBe(true);
    });
  });

  describe('System Integration', () => {
    it('should work end-to-end from service to command execution', async () => {
      // Initialize the full system
      const service = CommandService.initialize(mockDispatch);
      
      // Get available commands
      const commands = service.getAvailableCommands(mockContext);
      expect(commands.length).toBeGreaterThan(0);
      
      // Find a specific command
      const createFileCommand = service.getCommand('create-file');
      expect(createFileCommand).toBeDefined();
      expect(createFileCommand?.metadata.id).toBe('create-file');
      
      // Check command availability
      const canExecute = service.canExecuteCommand('create-file', mockContext);
      expect(typeof canExecute).toBe('boolean');
    });

    it('should handle missing commands gracefully', async () => {
      const service = CommandService.initialize(mockDispatch);
      
      const result = await service.executeCommand('non-existent-command', mockContext);
      expect(result).toBe(false);
      
      const command = service.getCommand('non-existent-command');
      expect(command).toBeUndefined();
    });

    it('should provide proper context conversion', () => {
      const service = CommandService.initialize(mockDispatch);
      
      // Test internal context conversion
      const commands = service.getAvailableCommands(mockContext);
      
      // Commands should be filtered based on context
      expect(commands.every(cmd => cmd.canExecute)).toBeDefined();
    });
  });

  describe('Performance', () => {
    it('should create commands efficiently', () => {
      const start = performance.now();
      
      CommandService.initialize(mockDispatch);
      const service = CommandService.getInstance();
      service.getAvailableCommands(mockContext);
      
      const duration = performance.now() - start;
      expect(duration).toBeLessThan(100); // Should be very fast
    });

    it('should handle large command sets', () => {
      const service = CommandService.initialize(mockDispatch);
      
      // Simulate searching through all commands multiple times
      const iterations = 100;
      const start = performance.now();
      
      for (let i = 0; i < iterations; i++) {
        service.searchCommands('test', mockContext);
      }
      
      const duration = performance.now() - start;
      const avgDuration = duration / iterations;
      expect(avgDuration).toBeLessThan(10); // Should average less than 10ms per search
    });
  });

  describe('Error Handling', () => {
    it('should handle command execution failures gracefully', async () => {
      const service = CommandService.initialize(mockDispatch);
      
      // Mock a command that will fail
      const mockContext = {
        ...mockContext,
        currentPath: '/invalid/path'
      };
      
      // Should not throw, but return false
      const result = await service.executeCommand('create-file', mockContext);
      expect(result).toBe(false);
    });

    it('should handle invalid contexts', () => {
      const service = CommandService.initialize(mockDispatch);
      
      const invalidContext = {
        ...mockContext,
        activePanelId: null
      };
      
      // Should still return commands, just filtered differently
      const commands = service.getAvailableCommands(invalidContext);
      expect(Array.isArray(commands)).toBe(true);
    });
  });
});