// Plugin system types for frontend
export interface PluginInfo {
  name: string;
  version: string;
  description: string;
  author: string;
  homepage?: string;
  repository?: string;
  license?: string;
  tags: string[];
  minVersion: string;
  maxVersion?: string;
}

export type PluginType = 'Content' | 'Protocol' | 'Viewer';

export type PluginStatus = 'Active' | 'Inactive' | 'Loading' | 'Error' | 'Unloaded';

export interface PluginManifest {
  info: PluginInfo;
  pluginType: PluginType;
  entryPoint: string;
  dependencies: string[];
  platforms: PlatformInfo[];
}

export interface PlatformInfo {
  os: 'windows' | 'macos' | 'linux';
  arch: 'x86_64' | 'aarch64' | 'x86';
  minVersion?: string;
}

export interface PluginInstance {
  id: string;
  manifest: PluginManifest;
  pluginPath: string;
  status: PluginStatus;
  loadedAt: string;
  error?: string;
}

export interface PluginStats {
  totalPlugins: number;
  activePlugins: number;
  inactivePlugins: number;
  errorPlugins: number;
  loadingPlugins: number;
  contentPlugins: number;
  protocolPlugins: number;
  viewerPlugins: number;
}

export interface ContentPluginCapabilities {
  supportedExtensions: string[];
  customColumns: ColumnDefinition[];
  canGenerateThumbnails: boolean;
}

export interface ProtocolPluginCapabilities {
  scheme: string;
  defaultPort: number;
  canCreateDirectories: boolean;
  canDelete: boolean;
  canRename: boolean;
  canSetPermissions: boolean;
  canPreserveTimestamps: boolean;
  canResumeTransfers: boolean;
  canConcurrentTransfers: boolean;
  maxPathLength?: number;
  maxFileSize?: number;
}

export interface ViewerPluginCapabilities {
  supportedExtensions: string[];
  supportedMimeTypes: string[];
  canView: boolean;
  canEdit: boolean;
  canSave: boolean;
  canSearch: boolean;
  canPrint: boolean;
  canCopy: boolean;
  canZoom: boolean;
  canFullscreen: boolean;
  maxFileSize?: number;
  preferredSize?: [number, number];
}

export interface ColumnDefinition {
  id: string;
  name: string;
  description?: string;
  width: number;
  sortable: boolean;
  visibleByDefault: boolean;
  alignment: 'Left' | 'Center' | 'Right';
}

export interface PluginDiscoverySettings {
  pluginDirectories: string[];
  recursiveScan: boolean;
  pluginExtensions: string[];
  autoLoad: boolean;
  loadTimeout: number;
  verifySignatures: boolean;
}

// Plugin management actions
export interface PluginAction {
  id: string;
  type: 'load' | 'unload' | 'enable' | 'disable' | 'configure' | 'update' | 'delete';
  pluginId?: string;
  payload?: any;
}

// UI component props
export interface PluginListProps {
  plugins: PluginInstance[];
  onPluginAction: (action: PluginAction) => void;
  filter?: PluginFilter;
  sortBy?: PluginSortBy;
}

export interface PluginCardProps {
  plugin: PluginInstance;
  onAction: (action: PluginAction) => void;
  isSelected?: boolean;
  showDetails?: boolean;
}

export interface PluginDetailsProps {
  plugin: PluginInstance;
  capabilities?: ContentPluginCapabilities | ProtocolPluginCapabilities | ViewerPluginCapabilities;
  onAction: (action: PluginAction) => void;
  onClose: () => void;
}

export interface PluginSettingsProps {
  settings: PluginDiscoverySettings;
  onSettingsChange: (settings: PluginDiscoverySettings) => void;
  onDiscoverPlugins: () => void;
}

export interface PluginFilter {
  type?: PluginType;
  status?: PluginStatus;
  searchTerm?: string;
  tags?: string[];
}

export interface PluginSortBy {
  field: 'name' | 'type' | 'status' | 'loadedAt' | 'author';
  direction: 'asc' | 'desc';
}

// Plugin installation and management
export interface PluginInstallRequest {
  source: 'file' | 'url' | 'registry';
  path?: string;
  url?: string;
  registryId?: string;
  overwrite?: boolean;
}

export interface PluginInstallProgress {
  pluginName: string;
  stage: 'downloading' | 'extracting' | 'validating' | 'installing' | 'loading';
  progress: number; // 0-100
  message?: string;
  error?: string;
}

// Store types
export interface PluginState {
  plugins: Record<string, PluginInstance>;
  stats: PluginStats;
  settings: PluginDiscoverySettings;
  filter: PluginFilter;
  sortBy: PluginSortBy;
  selectedPluginId?: string;
  isLoading: boolean;
  error?: string;
  installProgress?: PluginInstallProgress;
}