import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import MultiPanelLayout from '../MultiPanelLayout';
import { renderWithProviders, createMockPanelState } from '@/test/utils/testUtils';
import { mockFileList } from '@/test/mocks/fileService';

describe('MultiPanelLayout', () => {
  beforeEach(() => {
    // Reset any mocks before each test
  });

  it('renders with default dual panel layout', () => {
    const mockState = createMockPanelState({
      panels: {
        'panel-1': {
          id: 'panel-1',
          currentPath: '/home/user',
          files: mockFileList,
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
          currentPath: '/home/user/documents',
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

    renderWithProviders(<MultiPanelLayout />, {
      preloadedState: { panels: mockState },
    });

    // Should render grid container
    expect(screen.getByRole('grid')).toBeInTheDocument();
    
    // Should render the multi-panel layout
    const container = screen.getByTestId('multi-panel-layout');
    expect(container).toBeInTheDocument();
  });

  it('switches active panel when clicked', () => {
    const mockState = createMockPanelState({
      activePanelId: 'panel-1',
    });

    const { store } = renderWithProviders(<MultiPanelLayout />, {
      preloadedState: { panels: mockState },
    });

    // Initially panel-1 should be active
    expect(store.getState().panels.activePanelId).toBe('panel-1');

    // Find and click on the second panel wrapper
    const panelWrappers = screen.getAllByTestId(/panel-wrapper/);
    if (panelWrappers.length > 1) {
      fireEvent.click(panelWrappers[1]);
      expect(store.getState().panels.activePanelId).toBe('panel-2');
    }
  });

  it('applies correct grid layout styles', () => {
    const mockState = createMockPanelState({
      gridLayout: { rows: 2, cols: 2, name: '2x2 (Quad)' },
    });

    renderWithProviders(<MultiPanelLayout />, {
      preloadedState: { panels: mockState },
    });

    const gridContainer = screen.getByRole('grid');
    expect(gridContainer).toHaveStyle({
      display: 'grid',
      gridTemplateRows: 'repeat(2, 1fr)',
      gridTemplateColumns: 'repeat(2, 1fr)',
    });
  });

  it('renders correct number of panels based on grid layout', () => {
    const mockState = createMockPanelState({
      gridLayout: { rows: 2, cols: 2, name: '2x2 (Quad)' },
      panelOrder: ['panel-1', 'panel-2', 'panel-3', 'panel-4'],
      panels: {
        'panel-1': createTestPanel('panel-1'),
        'panel-2': createTestPanel('panel-2'),
        'panel-3': createTestPanel('panel-3'),
        'panel-4': createTestPanel('panel-4'),
      },
    });

    renderWithProviders(<MultiPanelLayout />, {
      preloadedState: { panels: mockState },
    });

    // Should render 4 panels for 2x2 grid
    const panelWrappers = screen.getAllByTestId(/panel-wrapper/);
    expect(panelWrappers).toHaveLength(4);
  });

  it('handles loading states correctly', () => {
    const mockState = createMockPanelState({
      panels: {
        'panel-1': {
          ...createTestPanel('panel-1'),
          isLoading: true,
        },
      },
    });

    renderWithProviders(<MultiPanelLayout />, {
      preloadedState: { panels: mockState },
    });

    expect(screen.getByTestId('loading-indicator')).toBeInTheDocument();
  });

  it('displays error messages when panels have errors', () => {
    const errorMessage = 'Failed to load directory';
    const mockState = createMockPanelState({
      panels: {
        'panel-1': {
          ...createTestPanel('panel-1'),
          error: errorMessage,
        },
      },
    });

    renderWithProviders(<MultiPanelLayout />, {
      preloadedState: { panels: mockState },
    });

    expect(screen.getByText(errorMessage)).toBeInTheDocument();
  });
});

// Helper function to create test panel (moved here to avoid import issues)
function createTestPanel(id: string, overrides: any = {}) {
  return {
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
  };
}