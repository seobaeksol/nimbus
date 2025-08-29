/**
 * Path Alias Service for Nimbus File Manager
 * 
 * Provides aliases for common directories and paths to enhance user experience
 * and reduce typing. Supports system directories like Documents, Downloads, Desktop, etc.
 */

export interface SystemPaths {
  home: string;
  documents: string;
  downloads: string;
  desktop: string;
  music: string;
  pictures: string;
  videos: string;
}

export interface PathAlias {
  alias: string;
  path: string;
  description?: string;
  systemAlias: boolean;
}

export class PathAliasService {
  private static instance: PathAliasService | null = null;
  private static systemPaths: SystemPaths | null = null;
  private static userAliases: Map<string, PathAlias> = new Map();
  private static initialized = false;

  private constructor() {}

  /**
   * Get the singleton instance
   */
  static getInstance(): PathAliasService {
    if (!this.instance) {
      this.instance = new PathAliasService();
    }
    return this.instance;
  }

  /**
   * Initialize the service with system paths
   */
  static async initialize(): Promise<void> {
    if (this.initialized) return;

    try {
      this.systemPaths = await this.detectSystemPaths();
      this.setupDefaultAliases();
      this.initialized = true;
    } catch (error) {
      console.error('Failed to initialize PathAliasService:', error);
      // Continue with minimal functionality
      this.initialized = true;
    }
  }

  /**
   * Resolve a path that might contain aliases
   * 
   * @param inputPath - The input path that might contain aliases
   * @param currentPath - The current working directory (for relative paths)
   * @returns Resolved absolute path
   */
  static async resolvePath(inputPath: string, currentPath: string = '/'): Promise<string> {
    if (!this.initialized) {
      await this.initialize();
    }

    // Trim whitespace
    let path = inputPath.trim();

    // Handle empty input
    if (!path) return currentPath;

    // Handle home directory alias (~)
    if (path === '~') {
      return this.systemPaths?.home || '/';
    }

    if (path.startsWith('~/')) {
      const homePath = this.systemPaths?.home || '/';
      return this.normalizePath(`${homePath}/${path.slice(2)}`);
    }

    // Check system aliases (e.g., documents, downloads, desktop)
    if (!path.includes('/') && !path.includes('\\')) {
      const systemAliases = this.getSystemAliases();
      if (systemAliases.has(path.toLowerCase())) {
        return systemAliases.get(path.toLowerCase())!;
      }

      // Check user-defined aliases
      const alias = this.userAliases.get(path.toLowerCase());
      if (alias) {
        return alias.path;
      }
    }

    // Handle absolute paths
    if (this.isAbsolutePath(path)) {
      return this.normalizePath(path);
    }

    // Handle relative paths - resolve against current directory
    if (path.startsWith('./')) {
      path = path.slice(2);
    }

    const resolvedPath = currentPath.endsWith('/') 
      ? `${currentPath}${path}`
      : `${currentPath}/${path}`;

    return this.normalizePath(resolvedPath);
  }

  /**
   * Add a user-defined alias
   */
  static addAlias(alias: string, path: string, description?: string): boolean {
    if (!this.isValidAliasName(alias)) {
      throw new Error(`Invalid alias name: ${alias}`);
    }

    // Don't allow overriding system aliases
    const systemAliases = this.getSystemAliases();
    if (systemAliases.has(alias.toLowerCase())) {
      throw new Error(`Cannot override system alias: ${alias}`);
    }

    const pathAlias: PathAlias = {
      alias: alias.toLowerCase(),
      path: this.normalizePath(path),
      description,
      systemAlias: false
    };

    this.userAliases.set(alias.toLowerCase(), pathAlias);
    return true;
  }

  /**
   * Remove a user-defined alias
   */
  static removeAlias(alias: string): boolean {
    return this.userAliases.delete(alias.toLowerCase());
  }

  /**
   * Get all available aliases
   */
  static getAllAliases(): PathAlias[] {
    const systemAliases = this.getSystemAliases();
    const aliases: PathAlias[] = [];

    // Add system aliases
    for (const [alias, path] of systemAliases) {
      aliases.push({
        alias,
        path,
        systemAlias: true,
        description: `System directory: ${path}`
      });
    }

    // Add user aliases
    aliases.push(...Array.from(this.userAliases.values()));

    return aliases.sort((a, b) => a.alias.localeCompare(b.alias));
  }

  /**
   * Search aliases by name or path
   */
  static searchAliases(query: string): PathAlias[] {
    const allAliases = this.getAllAliases();
    const lowerQuery = query.toLowerCase();

    return allAliases.filter(alias => 
      alias.alias.toLowerCase().includes(lowerQuery) ||
      alias.path.toLowerCase().includes(lowerQuery) ||
      (alias.description && alias.description.toLowerCase().includes(lowerQuery))
    );
  }

  /**
   * Get system paths
   */
  static getSystemPaths(): SystemPaths | null {
    return this.systemPaths;
  }

  /**
   * Clear all user-defined aliases
   */
  static clearUserAliases(): void {
    this.userAliases.clear();
  }

  /**
   * Export aliases to JSON
   */
  static exportAliases(): string {
    const userAliases = Array.from(this.userAliases.values());
    return JSON.stringify(userAliases, null, 2);
  }

  /**
   * Import aliases from JSON
   */
  static importAliases(jsonData: string): number {
    try {
      const aliases: PathAlias[] = JSON.parse(jsonData);
      let imported = 0;

      for (const alias of aliases) {
        if (this.isValidAliasName(alias.alias)) {
          this.userAliases.set(alias.alias.toLowerCase(), {
            ...alias,
            systemAlias: false // Force user alias
          });
          imported++;
        }
      }

      return imported;
    } catch (error) {
      throw new Error(`Failed to import aliases: ${error}`);
    }
  }

  /**
   * Detect system paths based on platform
   */
  private static async detectSystemPaths(): Promise<SystemPaths> {
    // const userAgent = navigator.userAgent;
    const platform = navigator.platform;

    // Detect OS
    const isMac = /Mac|iPhone|iPod|iPad/.test(platform);
    const isWindows = /Win/.test(platform);
    // const isLinux = /Linux/.test(platform) && !/Android/.test(userAgent);

    // Get username (simplified - in real implementation this would come from backend)
    const username = "user"; // Placeholder

    if (isMac) {
      return {
        home: `/Users/${username}`,
        documents: `/Users/${username}/Documents`,
        downloads: `/Users/${username}/Downloads`,
        desktop: `/Users/${username}/Desktop`,
        music: `/Users/${username}/Music`,
        pictures: `/Users/${username}/Pictures`,
        videos: `/Users/${username}/Movies`,
      };
    } else if (isWindows) {
      return {
        home: `C:\\Users\\${username}`,
        documents: `C:\\Users\\${username}\\Documents`,
        downloads: `C:\\Users\\${username}\\Downloads`,
        desktop: `C:\\Users\\${username}\\Desktop`,
        music: `C:\\Users\\${username}\\Music`,
        pictures: `C:\\Users\\${username}\\Pictures`,
        videos: `C:\\Users\\${username}\\Videos`,
      };
    } else {
      // Linux/Unix
      return {
        home: `/home/${username}`,
        documents: `/home/${username}/Documents`,
        downloads: `/home/${username}/Downloads`,
        desktop: `/home/${username}/Desktop`,
        music: `/home/${username}/Music`,
        pictures: `/home/${username}/Pictures`,
        videos: `/home/${username}/Videos`,
      };
    }
  }

  /**
   * Setup default system aliases
   */
  private static setupDefaultAliases(): void {
    if (!this.systemPaths) return;

    // Add common aliases (these are handled in resolvePath, but documented here)
    // documents, downloads, desktop, music, pictures, videos
  }

  /**
   * Get system directory aliases map
   */
  private static getSystemAliases(): Map<string, string> {
    const aliases = new Map<string, string>();

    if (this.systemPaths) {
      aliases.set("documents", this.systemPaths.documents);
      aliases.set("downloads", this.systemPaths.downloads);
      aliases.set("desktop", this.systemPaths.desktop);
      aliases.set("music", this.systemPaths.music);
      aliases.set("pictures", this.systemPaths.pictures);
      aliases.set("videos", this.systemPaths.videos);

      // Common variations
      aliases.set("docs", this.systemPaths.documents);
      aliases.set("dl", this.systemPaths.downloads);
      aliases.set("pics", this.systemPaths.pictures);
      aliases.set("movies", this.systemPaths.videos);
    }

    return aliases;
  }

  /**
   * Check if a path is absolute
   */
  private static isAbsolutePath(path: string): boolean {
    // Unix/Linux/Mac absolute paths start with /
    if (path.startsWith("/")) return true;

    // Windows absolute paths (C:\ or \\server\)
    if (/^[A-Za-z]:[\\/]/.test(path)) return true;
    if (path.startsWith("\\\\")) return true;

    return false;
  }

  /**
   * Normalize path separators for current platform
   */
  static normalizePath(path: string): string {
    // For web-based file manager, we'll primarily use forward slashes
    // In a real implementation with platform-specific backend, this would be more sophisticated
    return path.replace(/\\/g, "/");
  }

  /**
   * Format path for display (replace home with ~)
   */
  static formatForDisplay(path: string): string {
    if (!this.systemPaths) return path;

    if (path === this.systemPaths.home) {
      return "~";
    }

    if (path.startsWith(this.systemPaths.home + "/")) {
      return path.replace(this.systemPaths.home, "~");
    }

    return path;
  }

  /**
   * Validate if an alias name is valid
   */
  static isValidAliasName(alias: string): boolean {
    // Alias names should be simple strings without special path characters
    const invalidChars = /[\/\\:*?"<>|]/;
    return !invalidChars.test(alias) && alias.trim().length > 0;
  }
}