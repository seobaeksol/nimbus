import React, { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import panelSlice, { PanelState } from '@/store/slices/panelSlice';
import { CommandProvider } from '@/providers/CommandProvider';

// Custom render function that includes Redux Provider and CommandProvider
interface ExtendedRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  preloadedState?: {
    panels?: Partial<PanelState>;
  };
  store?: ReturnType<typeof configureStore>;
}

export function renderWithProviders(
  ui: ReactElement,
  {
    preloadedState = {},
    store = configureStore({
      reducer: {
        panels: panelSlice,
      },
      preloadedState,
    }),
    ...renderOptions
  }: ExtendedRenderOptions = {}
) {
  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <Provider store={store}>
        <CommandProvider>
          {children}
        </CommandProvider>
      </Provider>
    );
  }

  return { store, ...render(ui, { wrapper: Wrapper, ...renderOptions }) };
}

// Create mock panel state for testing
export const createMockPanelState = (overrides: Partial<PanelState> = {}): PanelState => ({
  panels: {
    'panel-1': {
      id: 'panel-1',
      currentPath: '/test/path',
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
      currentPath: '/test/path2',
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
  activePanelId: 'panel-1',
  gridLayout: { rows: 1, cols: 2, name: '1x2 (Classic Dual)' },
  panelOrder: ['panel-1', 'panel-2'],
  presetLayouts: [
    { rows: 1, cols: 1, name: '1x1 (Single Panel)' },
    { rows: 1, cols: 2, name: '1x2 (Classic Dual)' },
    { rows: 2, cols: 2, name: '2x2 (Quad)' },
  ],
  dragState: {
    isDragging: false,
    draggedFiles: [],
    sourcePanelId: null,
    dragOperation: null,
  },
  clipboardState: {
    hasFiles: false,
    files: [],
    sourcePanelId: null,
    operation: null,
    timestamp: 0,
  },
  progressIndicators: [],
  notifications: [],
  ...overrides,
});

// Helper to create a test panel
export const createTestPanel = (id: string, overrides: any = {}) => ({
  id,
  currentPath: `/test/${id}`,
  files: [],
  selectedFiles: [],
  isLoading: false,
  error: null,
  viewMode: 'list' as const,
  sortBy: 'name' as const,
  sortOrder: 'asc' as const,
  isAddressBarActive: false,
  ...overrides,
});

// Mock intersection observer for components that use it
export const mockIntersectionObserver = () => {
  const mockIntersectionObserver = vi.fn();
  mockIntersectionObserver.mockReturnValue({
    observe: () => null,
    unobserve: () => null,
    disconnect: () => null,
  });
  window.IntersectionObserver = mockIntersectionObserver;
  window.IntersectionObserverEntry = {} as any;
  window.IntersectionObserverInit = {} as any;
};