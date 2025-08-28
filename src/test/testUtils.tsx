import React, { PropsWithChildren } from 'react';
import { render } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { vi } from 'vitest';
import type { RenderOptions } from '@testing-library/react';
import type { PreloadedState } from '@reduxjs/toolkit';

import panelSlice from '../store/slices/panelSlice';
import type { RootState } from '../store';
import { FileInfo } from '../types';

// Mock FileService
export const mockFileService = {
  listDirectory: vi.fn(),
  getFileInfo: vi.fn(),
  createDirectory: vi.fn(),
  createFile: vi.fn(),
  deleteItem: vi.fn(),
  renameItem: vi.fn(),
  copyItem: vi.fn(),
  moveItem: vi.fn(),
  resolvePath: vi.fn(),
  getSystemPaths: vi.fn(),
  greet: vi.fn(),
};

vi.mock('../services/fileService', () => ({
  FileService: mockFileService,
}));

// Mock PathAliasService
export const mockPathAliasService = {
  resolvePath: vi.fn(),
};

vi.mock('../services/pathAliasService', () => ({
  PathAliasService: mockPathAliasService,
}));

// Default test state
export const createTestStore = (preloadedState?: PreloadedState<RootState>) => {
  return configureStore({
    reducer: {
      panels: panelSlice,
    },
    preloadedState,
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        serializableCheck: false, // Disable for testing
      }),
  });
};

// Default panel state for tests
export const mockPanelState: RootState['panels'] = {
  panels: {
    'panel-1': {
      id: 'panel-1',
      currentPath: '/test/path',
      files: [],
      selectedFiles: [],
      isLoading: false,
      error: null,
      sortBy: 'name',
      sortOrder: 'asc',
      viewMode: 'list',
    },
  },
  activePanelId: 'panel-1',
  gridLayout: { rows: 1, cols: 1, name: '1x1 (Single Panel)' },
  panelOrder: ['panel-1'],
  presetLayouts: [
    { rows: 1, cols: 1, name: '1x1 (Single Panel)' },
    { rows: 1, cols: 2, name: '1x2 (Classic Dual)' },
  ],
  clipboardState: {
    hasFiles: false,
    files: [],
    operation: 'copy',
    sourcePanelId: null,
    timestamp: Date.now(),
  },
  dragState: {
    isDragging: false,
    draggedFiles: [],
    sourcePanelId: null,
    dragOperation: 'move',
  },
  notifications: [],
  progressIndicators: [],
};

// Test wrapper with Redux Provider
interface ExtendedRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  preloadedState?: PreloadedState<RootState>;
  store?: ReturnType<typeof createTestStore>;
}

export function renderWithProviders(
  ui: React.ReactElement,
  {
    preloadedState = { panels: mockPanelState },
    store = createTestStore(preloadedState),
    ...renderOptions
  }: ExtendedRenderOptions = {}
) {
  function Wrapper({ children }: PropsWithChildren<{}>): JSX.Element {
    return <Provider store={store}>{children}</Provider>;
  }

  return { store, ...render(ui, { wrapper: Wrapper, ...renderOptions }) };
}

// Mock file data helpers
export const createMockFileInfo = (overrides: Partial<FileInfo> = {}): FileInfo => ({
  name: 'test-file.txt',
  path: '/test/path/test-file.txt',
  size: 1024,
  file_type: 'File',
  modified: '2023-01-01T00:00:00Z',
  extension: 'txt',
  ...overrides,
});

export const createMockDirectoryInfo = (overrides: Partial<FileInfo> = {}): FileInfo => ({
  name: 'test-directory',
  path: '/test/path/test-directory',
  size: 0,
  file_type: 'Directory',
  modified: '2023-01-01T00:00:00Z',
  extension: '',
  ...overrides,
});

// Mock CommandExecutor
export const mockCommandExecutor = {
  initialize: vi.fn(),
  createFile: vi.fn(),
  createFolder: vi.fn(),
  deleteFiles: vi.fn(),
  renameFile: vi.fn(),
  copyFiles: vi.fn(),
  cutFiles: vi.fn(),
  pasteFiles: vi.fn(),
  loadDirectory: vi.fn(),
  navigateToDirectory: vi.fn(),
  navigateToParent: vi.fn(),
  navigateToPath: vi.fn(),
  handleError: vi.fn(),
  goToHome: vi.fn(),
  goToDocuments: vi.fn(),
  goToDesktop: vi.fn(),
  goToDownloads: vi.fn(),
  goToApplications: vi.fn(),
  focusAddressBar: vi.fn(),
  promptGoToPath: vi.fn(),
  startDrag: vi.fn(),
  endDrag: vi.fn(),
  updateDragOperation: vi.fn(),
  handleDrop: vi.fn(),
};

// Helper to reset all mocks
export const resetAllMocks = () => {
  vi.clearAllMocks();
  Object.values(mockFileService).forEach((mock) => mock.mockReset?.());
  Object.values(mockPathAliasService).forEach((mock) => mock.mockReset?.());
  Object.values(mockCommandExecutor).forEach((mock) => mock.mockReset?.());
};

// Custom matchers for testing Redux actions
export const expectDispatchedActions = (store: ReturnType<typeof createTestStore>, expectedActions: string[]) => {
  const actions = store.getState();
  // This would be expanded based on your specific testing needs
  return expectedActions;
};