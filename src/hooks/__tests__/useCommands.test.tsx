import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import React from 'react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import panelSlice from '@/store/slices/panelSlice';
import { useCommands } from '../useCommands';
import { CommandProvider } from '@/providers/CommandProvider';
import { createMockPanelState } from '@/test/utils/testUtils';
import { resetFileServiceMocks } from '@/test/mocks/fileService';
import { mockInvoke } from '@/test/setup';

// Create wrapper with providers
const createWrapper = (initialState: any = {}) => {
  const store = configureStore({
    reducer: { panels: panelSlice },
    preloadedState: initialState,
  });

  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <Provider store={store}>
        <CommandProvider>
          {children}
        </CommandProvider>
      </Provider>
    );
  };
};

describe('useCommands', () => {
  beforeEach(() => {
    resetFileServiceMocks();
    mockInvoke.mockClear();
  });

  it('provides command execution functionality', () => {
    const wrapper = createWrapper({
      panels: createMockPanelState(),
    });

    const { result } = renderHook(() => useCommands(), { wrapper });

    expect(result.current).toHaveProperty('executeCommand');
    expect(result.current).toHaveProperty('getAvailableCommands');
    expect(result.current).toHaveProperty('searchCommands');
    expect(result.current).toHaveProperty('canExecuteCommand');
    expect(result.current).toHaveProperty('getCommand');
    expect(result.current).toHaveProperty('createContext');
    expect(result.current).toHaveProperty('commandService');
  });

  it('creates execution context with current panel state', () => {
    const mockState = createMockPanelState({
      activePanelId: 'panel-1',
      panels: {
        'panel-1': {
          id: 'panel-1',
          currentPath: '/test/path',
          files: [],
          selectedFiles: ['test-file.txt'],
          isLoading: false,
          error: null,
          viewMode: 'list',
          sortBy: 'name',
          sortOrder: 'asc',
          isAddressBarActive: false,
        },
      },
    });

    const wrapper = createWrapper({ panels: mockState });
    const { result } = renderHook(() => useCommands(), { wrapper });

    const context = result.current.createContext();

    expect(context).toEqual(
      expect.objectContaining({
        panelId: 'panel-1',
        currentPath: '/test/path',
        clipboardHasFiles: false,
      })
    );
  });

  it('returns available commands for current context', () => {
    const wrapper = createWrapper({
      panels: createMockPanelState(),
    });

    const { result } = renderHook(() => useCommands(), { wrapper });
    const commands = result.current.getAvailableCommands();

    expect(Array.isArray(commands)).toBe(true);
    expect(commands.length).toBeGreaterThan(0);
    
    // Should include basic file operation commands
    const commandIds = commands.map(cmd => cmd.metadata.id);
    expect(commandIds).toContain('load-directory');
    expect(commandIds).toContain('create-file');
    expect(commandIds).toContain('create-folder');
  });

  it('can search for commands by term', () => {
    const wrapper = createWrapper({
      panels: createMockPanelState(),
    });

    const { result } = renderHook(() => useCommands(), { wrapper });
    const searchResults = result.current.searchCommands('create');

    expect(Array.isArray(searchResults)).toBe(true);
    
    // Should find commands related to "create"
    const commandLabels = searchResults.map(cmd => cmd.metadata.label.toLowerCase());
    expect(commandLabels.some(label => label.includes('create') || label.includes('new'))).toBe(true);
  });

  it('can check if command can execute', () => {
    const wrapper = createWrapper({
      panels: createMockPanelState(),
    });

    const { result } = renderHook(() => useCommands(), { wrapper });
    
    // Test a basic command that should be available
    const canExecute = result.current.canExecuteCommand('load-directory');
    expect(typeof canExecute).toBe('boolean');
  });

  it('handles command execution', async () => {
    const wrapper = createWrapper({
      panels: createMockPanelState(),
    });

    const { result } = renderHook(() => useCommands(), { wrapper });
    
    // Mock a successful command execution
    const success = await result.current.executeCommand('load-directory');
    expect(typeof success).toBe('boolean');
  });

  it('creates context for specific panel when panel ID provided', () => {
    const mockState = createMockPanelState({
      panels: {
        'panel-1': {
          id: 'panel-1',
          currentPath: '/path1',
          files: [],
          selectedFiles: [],
          isLoading: false,
          error: null,
          viewMode: 'list',
          sortBy: 'name',
          sortOrder: 'asc',
          isAddressBarActive: false,
        },
        'panel-2': {
          id: 'panel-2',
          currentPath: '/path2',
          files: [],
          selectedFiles: [],
          isLoading: false,
          error: null,
          viewMode: 'list',
          sortBy: 'name',
          sortOrder: 'asc',
          isAddressBarActive: false,
        },
      },
    });

    const wrapper = createWrapper({ panels: mockState });
    const { result } = renderHook(() => useCommands(), { wrapper });

    const contextPanel2 = result.current.createContext('panel-2');
    expect(contextPanel2.panelId).toBe('panel-2');
    expect(contextPanel2.currentPath).toBe('/path2');
  });
});