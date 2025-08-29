/**
 * Path Alias Service
 * Handles path resolution, aliases, and cross-platform path normalization
 */

import { getSystemPaths, resolvePath } from "./commands/ipc/file";

export interface SystemPaths {
  home: string;
  documents: string;
  downloads: string;
  desktop: string;
  music: string;
  pictures: string;
  videos: string;
}

export interface UserAlias {
  alias: string;
  path: string;
  description?: string;
}

export class PathAliasService {
  private static systemPaths: SystemPaths | null = null;
  private static userAliases: Map<string, UserAlias> = new Map();

  /**
   * Initialize with system paths from backend
   */
  static async initialize(): Promise<void> {
    try {
      const backendPaths = await getSystemPaths();

      this.systemPaths = {
        home: backendPaths.home || "/",
        documents: backendPaths.documents || `${backendPaths.home}/Documents`,
        downloads: backendPaths.downloads || `${backendPaths.home}/Downloads`,
        desktop: backendPaths.desktop || `${backendPaths.home}/Desktop`,
        music: backendPaths.music || `${backendPaths.home}/Music`,
        pictures: backendPaths.pictures || `${backendPaths.home}/Pictures`,
        videos: backendPaths.videos || `${backendPaths.home}/Videos`,
      };

      this.setupDefaultAliases();
    } catch (error) {
      console.error(
        "Failed to initialize system paths from backend, falling back to detection:",
        error
      );
      // Fallback to client-side detection if backend fails
      this.systemPaths = await this.detectSystemPaths();
      this.setupDefaultAliases();
    }
  }

  /**
   * Resolve an alias or path to a full system path
   */
  static async resolvePath(input: string): Promise<string> {
    if (!this.systemPaths) {
      await this.initialize();
    }

    try {
      // Use the backend's resolve_path command which handles all the complexity
      // including tilde expansion, system aliases, and path normalization
      const resolvedPath = await resolvePath(input);
      return resolvedPath;
    } catch (error) {
      console.error("Backend path resolution failed, using fallback:", error);

      // Fallback to client-side resolution if backend fails
      const normalizedInput = input.trim();

      // Handle empty input
      if (!normalizedInput) {
        return this.systemPaths!.home;
      }

      // Handle root path
      if (normalizedInput === "/") {
        return "/";
      }

      // Handle home directory aliases
      if (normalizedInput === "~" || normalizedInput === "~/") {
        return this.systemPaths!.home;
      }

      // Handle tilde expansion
      if (normalizedInput.startsWith("~/")) {
        return normalizedInput.replace("~", this.systemPaths!.home);
      }

      // Handle user-defined aliases
      const lowerInput = normalizedInput.toLowerCase();
      if (this.userAliases.has(lowerInput)) {
        const alias = this.userAliases.get(lowerInput)!;
        return alias.path;
      }

      // Handle common system directory aliases (case-insensitive)
      const systemAliases = this.getSystemAliases();
      if (systemAliases.has(lowerInput)) {
        return systemAliases.get(lowerInput)!;
      }

      // Return the path as-is if no alias matched
      return normalizedInput;
    }
  }

  /**
   * Get all available aliases for autocomplete/suggestions
   */
  static getAvailableAliases(): {
    alias: string;
    path: string;
    description: string;
  }[] {
    const aliases: { alias: string; path: string; description: string }[] = [];

    // System aliases
    aliases.push(
      {
        alias: "~",
        path: this.systemPaths?.home || "/",
        description: "Home directory",
      },
      { alias: "/", path: "/", description: "Root directory" },
      {
        alias: "Documents",
        path: this.systemPaths?.documents || "~/Documents",
        description: "Documents folder",
      },
      {
        alias: "Downloads",
        path: this.systemPaths?.downloads || "~/Downloads",
        description: "Downloads folder",
      },
      {
        alias: "Desktop",
        path: this.systemPaths?.desktop || "~/Desktop",
        description: "Desktop folder",
      },
      {
        alias: "Music",
        path: this.systemPaths?.music || "~/Music",
        description: "Music folder",
      },
      {
        alias: "Pictures",
        path: this.systemPaths?.pictures || "~/Pictures",
        description: "Pictures folder",
      },
      {
        alias: "Videos",
        path: this.systemPaths?.videos || "~/Videos",
        description: "Videos folder",
      }
    );

    // User-defined aliases
    this.userAliases.forEach((userAlias) => {
      aliases.push({
        alias: userAlias.alias,
        path: userAlias.path,
        description:
          userAlias.description || `Custom alias for ${userAlias.path}`,
      });
    });

    return aliases;
  }

  /**
   * Add a user-defined alias
   */
  static addUserAlias(alias: string, path: string, description?: string): void {
    this.userAliases.set(alias.toLowerCase(), {
      alias: alias.toLowerCase(),
      path,
      description,
    });
  }

  /**
   * Remove a user-defined alias
   */
  static removeUserAlias(alias: string): boolean {
    return this.userAliases.delete(alias.toLowerCase());
  }

  /**
   * Get user-defined aliases
   */
  static getUserAliases(): UserAlias[] {
    return Array.from(this.userAliases.values());
  }

  /**
   * Detect system paths based on platform
   */
  private static async detectSystemPaths(): Promise<SystemPaths> {
    const userAgent = navigator.userAgent;
    const platform = navigator.platform;

    // Detect OS
    const isMac = /Mac|iPhone|iPod|iPad/.test(platform);
    const isWindows = /Win/.test(platform);
    const isLinux = /Linux/.test(platform) && !/Android/.test(userAgent);

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
