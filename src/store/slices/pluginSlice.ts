import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import {
  PluginState,
  PluginInstance,
  PluginStats,
  PluginDiscoverySettings,
  PluginFilter,
  PluginSortBy,
  PluginInstallProgress,
  PluginAction,
} from '../../types/plugins';

const initialState: PluginState = {
  plugins: {},
  stats: {
    totalPlugins: 0,
    activePlugins: 0,
    inactivePlugins: 0,
    errorPlugins: 0,
    loadingPlugins: 0,
    contentPlugins: 0,
    protocolPlugins: 0,
    viewerPlugins: 0,
  },
  settings: {
    pluginDirectories: [
      'plugins',
      '/usr/local/lib/nimbus/plugins',
      '~/.local/lib/nimbus/plugins',
    ],
    recursiveScan: true,
    pluginExtensions: ['dll', 'so', 'dylib'],
    autoLoad: false,
    loadTimeout: 30,
    verifySignatures: true,
  },
  filter: {},
  sortBy: {
    field: 'name',
    direction: 'asc',
  },
  isLoading: false,
};

const pluginSlice = createSlice({
  name: 'plugins',
  initialState,
  reducers: {
    // Plugin management
    setPlugins: (state, action: PayloadAction<PluginInstance[]>) => {
      state.plugins = action.payload.reduce((acc, plugin) => {
        acc[plugin.id] = plugin;
        return acc;
      }, {} as Record<string, PluginInstance>);
      state.stats = calculateStats(action.payload);
    },

    addPlugin: (state, action: PayloadAction<PluginInstance>) => {
      state.plugins[action.payload.id] = action.payload;
      state.stats = calculateStats(Object.values(state.plugins));
    },

    updatePlugin: (state, action: PayloadAction<Partial<PluginInstance> & { id: string }>) => {
      const { id, ...updates } = action.payload;
      if (state.plugins[id]) {
        state.plugins[id] = { ...state.plugins[id], ...updates };
        state.stats = calculateStats(Object.values(state.plugins));
      }
    },

    removePlugin: (state, action: PayloadAction<string>) => {
      delete state.plugins[action.payload];
      if (state.selectedPluginId === action.payload) {
        state.selectedPluginId = undefined;
      }
      state.stats = calculateStats(Object.values(state.plugins));
    },

    // Plugin selection
    setSelectedPlugin: (state, action: PayloadAction<string | undefined>) => {
      state.selectedPluginId = action.payload;
    },

    // Settings management
    updateSettings: (state, action: PayloadAction<Partial<PluginDiscoverySettings>>) => {
      state.settings = { ...state.settings, ...action.payload };
    },

    // Filter and sorting
    setFilter: (state, action: PayloadAction<Partial<PluginFilter>>) => {
      state.filter = { ...state.filter, ...action.payload };
    },

    clearFilter: (state) => {
      state.filter = {};
    },

    setSortBy: (state, action: PayloadAction<PluginSortBy>) => {
      state.sortBy = action.payload;
    },

    // Loading states
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },

    setError: (state, action: PayloadAction<string | undefined>) => {
      state.error = action.payload;
    },

    // Installation progress
    setInstallProgress: (state, action: PayloadAction<PluginInstallProgress | undefined>) => {
      state.installProgress = action.payload;
    },

    // Plugin status updates
    setPluginStatus: (state, action: PayloadAction<{ pluginId: string; status: PluginInstance['status']; error?: string }>) => {
      const { pluginId, status, error } = action.payload;
      if (state.plugins[pluginId]) {
        state.plugins[pluginId].status = status;
        if (error) {
          state.plugins[pluginId].error = error;
        } else {
          delete state.plugins[pluginId].error;
        }
        state.stats = calculateStats(Object.values(state.plugins));
      }
    },

    // Bulk operations
    setMultiplePluginStatus: (state, action: PayloadAction<{ pluginIds: string[]; status: PluginInstance['status'] }>) => {
      const { pluginIds, status } = action.payload;
      pluginIds.forEach(pluginId => {
        if (state.plugins[pluginId]) {
          state.plugins[pluginId].status = status;
        }
      });
      state.stats = calculateStats(Object.values(state.plugins));
    },

    // Reset state
    resetPluginState: () => initialState,
  },
});

// Helper function to calculate plugin statistics
function calculateStats(plugins: PluginInstance[]): PluginStats {
  const stats: PluginStats = {
    totalPlugins: plugins.length,
    activePlugins: 0,
    inactivePlugins: 0,
    errorPlugins: 0,
    loadingPlugins: 0,
    contentPlugins: 0,
    protocolPlugins: 0,
    viewerPlugins: 0,
  };

  plugins.forEach(plugin => {
    // Count by status
    switch (plugin.status) {
      case 'Active':
        stats.activePlugins++;
        break;
      case 'Inactive':
        stats.inactivePlugins++;
        break;
      case 'Error':
        stats.errorPlugins++;
        break;
      case 'Loading':
        stats.loadingPlugins++;
        break;
    }

    // Count by type
    switch (plugin.manifest.pluginType) {
      case 'Content':
        stats.contentPlugins++;
        break;
      case 'Protocol':
        stats.protocolPlugins++;
        break;
      case 'Viewer':
        stats.viewerPlugins++;
        break;
    }
  });

  return stats;
}

export const {
  setPlugins,
  addPlugin,
  updatePlugin,
  removePlugin,
  setSelectedPlugin,
  updateSettings,
  setFilter,
  clearFilter,
  setSortBy,
  setLoading,
  setError,
  setInstallProgress,
  setPluginStatus,
  setMultiplePluginStatus,
  resetPluginState,
} = pluginSlice.actions;

export default pluginSlice.reducer;

// Selectors
export const selectAllPlugins = (state: { plugins: PluginState }) => 
  Object.values(state.plugins.plugins);

export const selectPluginById = (state: { plugins: PluginState }, pluginId: string) =>
  state.plugins.plugins[pluginId];

export const selectSelectedPlugin = (state: { plugins: PluginState }) =>
  state.plugins.selectedPluginId ? state.plugins.plugins[state.plugins.selectedPluginId] : undefined;

export const selectPluginStats = (state: { plugins: PluginState }) =>
  state.plugins.stats;

export const selectFilteredAndSortedPlugins = (state: { plugins: PluginState }) => {
  let plugins = Object.values(state.plugins.plugins);
  const { filter, sortBy } = state.plugins;

  // Apply filters
  if (filter.type) {
    plugins = plugins.filter(plugin => plugin.manifest.pluginType === filter.type);
  }
  if (filter.status) {
    plugins = plugins.filter(plugin => plugin.status === filter.status);
  }
  if (filter.searchTerm) {
    const searchTerm = filter.searchTerm.toLowerCase();
    plugins = plugins.filter(plugin =>
      plugin.manifest.info.name.toLowerCase().includes(searchTerm) ||
      plugin.manifest.info.description.toLowerCase().includes(searchTerm) ||
      plugin.manifest.info.author.toLowerCase().includes(searchTerm) ||
      plugin.manifest.info.tags.some(tag => tag.toLowerCase().includes(searchTerm))
    );
  }
  if (filter.tags && filter.tags.length > 0) {
    plugins = plugins.filter(plugin =>
      filter.tags!.every(tag => plugin.manifest.info.tags.includes(tag))
    );
  }

  // Apply sorting
  plugins.sort((a, b) => {
    let aValue: any;
    let bValue: any;

    switch (sortBy.field) {
      case 'name':
        aValue = a.manifest.info.name.toLowerCase();
        bValue = b.manifest.info.name.toLowerCase();
        break;
      case 'type':
        aValue = a.manifest.pluginType;
        bValue = b.manifest.pluginType;
        break;
      case 'status':
        aValue = a.status;
        bValue = b.status;
        break;
      case 'loadedAt':
        aValue = new Date(a.loadedAt);
        bValue = new Date(b.loadedAt);
        break;
      case 'author':
        aValue = a.manifest.info.author.toLowerCase();
        bValue = b.manifest.info.author.toLowerCase();
        break;
      default:
        return 0;
    }

    if (aValue < bValue) {
      return sortBy.direction === 'asc' ? -1 : 1;
    }
    if (aValue > bValue) {
      return sortBy.direction === 'asc' ? 1 : -1;
    }
    return 0;
  });

  return plugins;
};

export const selectPluginSettings = (state: { plugins: PluginState }) =>
  state.plugins.settings;

export const selectPluginFilter = (state: { plugins: PluginState }) =>
  state.plugins.filter;

export const selectPluginSortBy = (state: { plugins: PluginState }) =>
  state.plugins.sortBy;

export const selectIsLoading = (state: { plugins: PluginState }) =>
  state.plugins.isLoading;

export const selectError = (state: { plugins: PluginState }) =>
  state.plugins.error;

export const selectInstallProgress = (state: { plugins: PluginState }) =>
  state.plugins.installProgress;