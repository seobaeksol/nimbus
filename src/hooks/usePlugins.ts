import { useCallback, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { listen } from '@tauri-apps/api/event';
import { PluginService } from '../services/pluginService';
import {
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
  selectAllPlugins,
  selectPluginById,
  selectSelectedPlugin,
  selectPluginStats,
  selectFilteredAndSortedPlugins,
  selectPluginSettings,
  selectPluginFilter,
  selectPluginSortBy,
  selectIsLoading,
  selectError,
  selectInstallProgress,
} from '../store/slices/pluginSlice';
import {
  PluginInstance,
  PluginAction,
  PluginDiscoverySettings,
  PluginFilter,
  PluginSortBy,
  PluginInstallRequest,
  PluginInstallProgress,
} from '../types/plugins';

interface PluginProgressEvent {
  pluginName: string;
  stage: 'downloading' | 'extracting' | 'validating' | 'installing' | 'loading';
  progress: number;
  message?: string;
  error?: string;
}

interface PluginStatusEvent {
  pluginId: string;
  status: PluginInstance['status'];
  error?: string;
}

export function usePlugins() {
  const dispatch = useDispatch();
  
  // Selectors
  const allPlugins = useSelector(selectAllPlugins);
  const filteredPlugins = useSelector(selectFilteredAndSortedPlugins);
  const selectedPlugin = useSelector(selectSelectedPlugin);
  const pluginStats = useSelector(selectPluginStats);
  const settings = useSelector(selectPluginSettings);
  const filter = useSelector(selectPluginFilter);
  const sortBy = useSelector(selectPluginSortBy);
  const isLoading = useSelector(selectIsLoading);
  const error = useSelector(selectError);
  const installProgress = useSelector(selectInstallProgress);

  // Initialize plugin system
  const initialize = useCallback(async () => {
    try {
      dispatch(setLoading(true));
      dispatch(setError(undefined));

      // Load current settings and plugins
      const [currentSettings, loadedPlugins] = await Promise.all([
        PluginService.getPluginSettings(),
        PluginService.getLoadedPlugins(),
      ]);

      dispatch(updateSettings(currentSettings));
      dispatch(setPlugins(loadedPlugins));
    } catch (err: any) {
      dispatch(setError(`Failed to initialize plugin system: ${err.message}`));
    } finally {
      dispatch(setLoading(false));
    }
  }, [dispatch]);

  // Discover plugins
  const discoverPlugins = useCallback(async () => {
    try {
      dispatch(setLoading(true));
      dispatch(setError(undefined));
      
      const manifests = await PluginService.discoverPlugins();
      // This would trigger loading of discovered plugins or update UI
      console.log(`Discovered ${manifests.length} plugins`);
    } catch (err: any) {
      dispatch(setError(`Failed to discover plugins: ${err.message}`));
    } finally {
      dispatch(setLoading(false));
    }
  }, [dispatch]);

  // Load plugin
  const loadPlugin = useCallback(async (pluginPath: string) => {
    try {
      dispatch(setError(undefined));
      
      const pluginId = await PluginService.loadPlugin(pluginPath);
      const pluginInfo = await PluginService.getPluginInfo(pluginId);
      
      if (pluginInfo) {
        dispatch(addPlugin(pluginInfo));
      }
      
      return pluginId;
    } catch (err: any) {
      dispatch(setError(`Failed to load plugin: ${err.message}`));
      throw err;
    }
  }, [dispatch]);

  // Unload plugin
  const unloadPlugin = useCallback(async (pluginId: string) => {
    try {
      dispatch(setError(undefined));
      await PluginService.unloadPlugin(pluginId);
      dispatch(removePlugin(pluginId));
    } catch (err: any) {
      dispatch(setError(`Failed to unload plugin: ${err.message}`));
      throw err;
    }
  }, [dispatch]);

  // Enable/disable plugin
  const setPluginEnabled = useCallback(async (pluginId: string, enabled: boolean) => {
    try {
      dispatch(setError(undefined));
      await PluginService.setPluginEnabled(pluginId, enabled);
      dispatch(setPluginStatus({
        pluginId,
        status: enabled ? 'Active' : 'Inactive'
      }));
    } catch (err: any) {
      dispatch(setError(`Failed to ${enabled ? 'enable' : 'disable'} plugin: ${err.message}`));
      throw err;
    }
  }, [dispatch]);

  // Install plugin
  const installPlugin = useCallback(async (request: PluginInstallRequest) => {
    try {
      dispatch(setError(undefined));
      const pluginId = await PluginService.installPlugin(request);
      
      // Refresh plugin list after installation
      const loadedPlugins = await PluginService.getLoadedPlugins();
      dispatch(setPlugins(loadedPlugins));
      
      return pluginId;
    } catch (err: any) {
      dispatch(setError(`Failed to install plugin: ${err.message}`));
      throw err;
    }
  }, [dispatch]);

  // Uninstall plugin
  const uninstallPlugin = useCallback(async (pluginId: string) => {
    try {
      dispatch(setError(undefined));
      await PluginService.uninstallPlugin(pluginId);
      dispatch(removePlugin(pluginId));
    } catch (err: any) {
      dispatch(setError(`Failed to uninstall plugin: ${err.message}`));
      throw err;
    }
  }, [dispatch]);

  // Update plugin settings
  const updatePluginSettings = useCallback(async (newSettings: Partial<PluginDiscoverySettings>) => {
    try {
      dispatch(setError(undefined));
      const updatedSettings = { ...settings, ...newSettings };
      await PluginService.updatePluginSettings(updatedSettings);
      dispatch(updateSettings(newSettings));
    } catch (err: any) {
      dispatch(setError(`Failed to update settings: ${err.message}`));
      throw err;
    }
  }, [dispatch, settings]);

  // Plugin action handler
  const executePluginAction = useCallback(async (action: PluginAction) => {
    try {
      dispatch(setError(undefined));
      
      switch (action.type) {
        case 'load':
          if (action.payload?.path) {
            await loadPlugin(action.payload.path);
          }
          break;
        case 'unload':
          if (action.pluginId) {
            await unloadPlugin(action.pluginId);
          }
          break;
        case 'enable':
          if (action.pluginId) {
            await setPluginEnabled(action.pluginId, true);
          }
          break;
        case 'disable':
          if (action.pluginId) {
            await setPluginEnabled(action.pluginId, false);
          }
          break;
        case 'update':
          if (action.pluginId) {
            await PluginService.updatePlugin(action.pluginId);
            // Refresh plugin info after update
            const updatedPlugin = await PluginService.getPluginInfo(action.pluginId);
            if (updatedPlugin) {
              dispatch(updatePlugin(updatedPlugin));
            }
          }
          break;
        case 'delete':
          if (action.pluginId) {
            await uninstallPlugin(action.pluginId);
          }
          break;
        default:
          throw new Error(`Unknown plugin action: ${action.type}`);
      }
    } catch (err: any) {
      dispatch(setError(`Failed to execute plugin action: ${err.message}`));
      throw err;
    }
  }, [dispatch, loadPlugin, unloadPlugin, setPluginEnabled, uninstallPlugin]);

  // Helper function to get plugin by ID
  const getPluginById = useCallback((pluginId: string) => {
    return useSelector((state: any) => selectPluginById(state, pluginId));
  }, []);

  // Filter and sorting controls
  const setPluginFilter = useCallback((newFilter: Partial<PluginFilter>) => {
    dispatch(setFilter(newFilter));
  }, [dispatch]);

  const clearPluginFilter = useCallback(() => {
    dispatch(clearFilter());
  }, [dispatch]);

  const setPluginSortBy = useCallback((sortConfig: PluginSortBy) => {
    dispatch(setSortBy(sortConfig));
  }, [dispatch]);

  const selectPlugin = useCallback((pluginId: string | undefined) => {
    dispatch(setSelectedPlugin(pluginId));
  }, [dispatch]);

  // Refresh plugins
  const refreshPlugins = useCallback(async () => {
    try {
      dispatch(setLoading(true));
      const loadedPlugins = await PluginService.getLoadedPlugins();
      dispatch(setPlugins(loadedPlugins));
    } catch (err: any) {
      dispatch(setError(`Failed to refresh plugins: ${err.message}`));
    } finally {
      dispatch(setLoading(false));
    }
  }, [dispatch]);

  // Event listeners setup
  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenStatus: (() => void) | undefined;

    const setupEventListeners = async () => {
      // Listen for plugin installation progress
      unlistenProgress = await listen<PluginProgressEvent>('plugin-install-progress', (event) => {
        dispatch(setInstallProgress({
          pluginName: event.payload.pluginName,
          stage: event.payload.stage,
          progress: event.payload.progress,
          message: event.payload.message,
          error: event.payload.error,
        }));
      });

      // Listen for plugin status changes
      unlistenStatus = await listen<PluginStatusEvent>('plugin-status-changed', (event) => {
        dispatch(setPluginStatus({
          pluginId: event.payload.pluginId,
          status: event.payload.status,
          error: event.payload.error,
        }));
      });
    };

    setupEventListeners().catch(console.error);

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenStatus) unlistenStatus();
    };
  }, [dispatch]);

  return {
    // State
    allPlugins,
    filteredPlugins,
    selectedPlugin,
    pluginStats,
    settings,
    filter,
    sortBy,
    isLoading,
    error,
    installProgress,

    // Actions
    initialize,
    discoverPlugins,
    loadPlugin,
    unloadPlugin,
    installPlugin,
    uninstallPlugin,
    setPluginEnabled,
    updatePluginSettings,
    executePluginAction,
    refreshPlugins,

    // Utilities
    getPluginById,
    setPluginFilter,
    clearPluginFilter,
    setPluginSortBy,
    selectPlugin,
  };
}