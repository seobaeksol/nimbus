import { invoke } from '@tauri-apps/api/tauri';
import {
  PluginInstance,
  PluginManifest,
  PluginStats,
  PluginDiscoverySettings,
  PluginInstallRequest,
  ContentPluginCapabilities,
  ProtocolPluginCapabilities,
  ViewerPluginCapabilities,
} from '../types/plugins';

export class PluginService {
  /**
   * Discover plugins in configured directories
   */
  static async discoverPlugins(): Promise<PluginManifest[]> {
    return await invoke<PluginManifest[]>('discover_plugins');
  }

  /**
   * Load a plugin from file path
   */
  static async loadPlugin(pluginPath: string): Promise<string> {
    return await invoke<string>('load_plugin', { pluginPath });
  }

  /**
   * Unload a plugin by ID
   */
  static async unloadPlugin(pluginId: string): Promise<void> {
    await invoke('unload_plugin', { pluginId });
  }

  /**
   * Enable or disable a plugin
   */
  static async setPluginEnabled(pluginId: string, enabled: boolean): Promise<void> {
    await invoke('set_plugin_enabled', { pluginId, enabled });
  }

  /**
   * Get list of all loaded plugins
   */
  static async getLoadedPlugins(): Promise<PluginInstance[]> {
    return await invoke<PluginInstance[]>('get_loaded_plugins');
  }

  /**
   * Get information about a specific plugin
   */
  static async getPluginInfo(pluginId: string): Promise<PluginInstance | null> {
    return await invoke<PluginInstance | null>('get_plugin_info', { pluginId });
  }

  /**
   * Get plugin statistics
   */
  static async getPluginStats(): Promise<PluginStats> {
    return await invoke<PluginStats>('get_plugin_stats');
  }

  /**
   * Update plugin discovery settings
   */
  static async updatePluginSettings(settings: PluginDiscoverySettings): Promise<void> {
    await invoke('update_plugin_settings', { settings });
  }

  /**
   * Get current plugin discovery settings
   */
  static async getPluginSettings(): Promise<PluginDiscoverySettings> {
    return await invoke<PluginDiscoverySettings>('get_plugin_settings');
  }

  /**
   * Install a plugin from various sources
   */
  static async installPlugin(request: PluginInstallRequest): Promise<string> {
    return await invoke<string>('install_plugin', { request });
  }

  /**
   * Uninstall a plugin
   */
  static async uninstallPlugin(pluginId: string): Promise<void> {
    await invoke('uninstall_plugin', { pluginId });
  }

  /**
   * Update a plugin to the latest version
   */
  static async updatePlugin(pluginId: string): Promise<void> {
    await invoke('update_plugin', { pluginId });
  }

  /**
   * Validate a plugin file before loading
   */
  static async validatePlugin(pluginPath: string): Promise<{ valid: boolean; errors: string[] }> {
    return await invoke<{ valid: boolean; errors: string[] }>('validate_plugin', { pluginPath });
  }

  /**
   * Get capabilities for a content plugin
   */
  static async getContentPluginCapabilities(pluginId: string): Promise<ContentPluginCapabilities> {
    return await invoke<ContentPluginCapabilities>('get_content_plugin_capabilities', { pluginId });
  }

  /**
   * Get capabilities for a protocol plugin
   */
  static async getProtocolPluginCapabilities(pluginId: string): Promise<ProtocolPluginCapabilities> {
    return await invoke<ProtocolPluginCapabilities>('get_protocol_plugin_capabilities', { pluginId });
  }

  /**
   * Get capabilities for a viewer plugin
   */
  static async getViewerPluginCapabilities(pluginId: string): Promise<ViewerPluginCapabilities> {
    return await invoke<ViewerPluginCapabilities>('get_viewer_plugin_capabilities', { pluginId });
  }

  /**
   * Get supported file extensions by all content plugins
   */
  static async getSupportedExtensions(): Promise<Record<string, string[]>> {
    return await invoke<Record<string, string[]>>('get_supported_extensions');
  }

  /**
   * Get supported protocols by all protocol plugins
   */
  static async getSupportedProtocols(): Promise<Record<string, { scheme: string; defaultPort: number }>> {
    return await invoke<Record<string, { scheme: string; defaultPort: number }>>('get_supported_protocols');
  }

  /**
   * Get available viewers for a file type
   */
  static async getViewersForFile(filePath: string): Promise<string[]> {
    return await invoke<string[]>('get_viewers_for_file', { filePath });
  }

  /**
   * Test plugin compatibility with current system
   */
  static async testPluginCompatibility(pluginPath: string): Promise<{ compatible: boolean; issues: string[] }> {
    return await invoke<{ compatible: boolean; issues: string[] }>('test_plugin_compatibility', { pluginPath });
  }

  /**
   * Get plugin dependency information
   */
  static async getPluginDependencies(pluginId: string): Promise<{ dependencies: string[]; dependents: string[] }> {
    return await invoke<{ dependencies: string[]; dependents: string[] }>('get_plugin_dependencies', { pluginId });
  }

  /**
   * Backup plugin configuration
   */
  static async backupPluginConfig(): Promise<string> {
    return await invoke<string>('backup_plugin_config');
  }

  /**
   * Restore plugin configuration from backup
   */
  static async restorePluginConfig(backupPath: string): Promise<void> {
    await invoke('restore_plugin_config', { backupPath });
  }

  /**
   * Clear plugin cache
   */
  static async clearPluginCache(): Promise<void> {
    await invoke('clear_plugin_cache');
  }

  /**
   * Get plugin logs
   */
  static async getPluginLogs(pluginId: string, lines?: number): Promise<string[]> {
    return await invoke<string[]>('get_plugin_logs', { pluginId, lines });
  }
}

// Error types for plugin operations
export interface PluginError {
  kind: 'NotFound' | 'LoadingFailed' | 'InvalidFormat' | 'DependencyMissing' | 
        'IncompatibleVersion' | 'AlreadyLoaded' | 'ConfigurationError';
  message: string;
  pluginPath?: string;
  details?: string;
}

// Plugin operation results
export interface PluginOperationResult {
  success: boolean;
  message?: string;
  errors?: string[];
  warnings?: string[];
}