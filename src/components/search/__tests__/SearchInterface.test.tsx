/**
 * SearchInterface Test
 * 
 * Basic smoke test for the search interface to ensure it renders without errors.
 */

import { render, screen } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { describe, it, expect } from 'vitest';
import { SearchInterface } from '../SearchInterface';
import searchSlice from '@/store/slices/searchSlice';
import panelSlice from '@/store/slices/panelSlice';

// Create test store
const createTestStore = () => configureStore({
  reducer: {
    search: searchSlice,
    panels: panelSlice
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST', 'persist/REHYDRATE'],
      },
    }),
});

describe('SearchInterface', () => {
  it('renders without crashing', () => {
    const store = createTestStore();
    
    render(
      <Provider store={store}>
        <SearchInterface />
      </Provider>
    );

    expect(screen.getByText('File Search')).toBeInTheDocument();
  });

  it('shows search form elements', () => {
    const store = createTestStore();
    
    render(
      <Provider store={store}>
        <SearchInterface />
      </Provider>
    );

    expect(screen.getByLabelText(/search path/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/file name pattern/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/content pattern/i)).toBeInTheDocument();
    expect(screen.getByText('Search')).toBeInTheDocument();
  });

  it('displays no results message initially', () => {
    const store = createTestStore();
    
    render(
      <Provider store={store}>
        <SearchInterface />
      </Provider>
    );

    expect(screen.getByText('No results found')).toBeInTheDocument();
  });
});