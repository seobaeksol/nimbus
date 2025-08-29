// File service for IPC communication with Rust backend

import { invoke } from "@tauri-apps/api/core";

// Core types matching the Rust backend
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  size_formatted: string;
  modified: string; // ISO timestamp
  created?: string;
  accessed?: string;
  file_type: "File" | "Directory" | "Symlink";
  extension?: string;
  permissions: FilePermissions;
  is_hidden: boolean;
  is_readonly: boolean;
}

export interface FilePermissions {
  read: boolean;
  write: boolean;
  execute: boolean;
}

export interface SystemInfo {
  platform: string;
  arch: string;
  hostname: string;
  username: string;
  home_dir?: string;
  temp_dir: string;
  app_name: string;
  app_version: string;
}

export const listDirectory = async (path: string): Promise<FileInfo[]> => {
  try {
    const files = await invoke<FileInfo[]>("list_dir", { path });
    // Normalize paths in FileInfo objects to remove Windows long path prefixes
    return files.map((file) => ({
      ...file,
      path: normalizeWindowsPath(file.path),
    }));
  } catch (error) {
    console.error("Failed to list directory:", error);
    throw error;
  }
};

/**
 * Get detailed information about a file or directory
 */
export const getFileInfo = async (path: string): Promise<FileInfo> => {
  try {
    const info = await invoke<FileInfo>("get_file_info", { path });
    // Normalize path in FileInfo object to remove Windows long path prefixes
    return {
      ...info,
      path: normalizeWindowsPath(info.path),
    };
  } catch (error) {
    console.error("Failed to get file info:", error);
    throw error;
  }
};

/**
 * Create a new directory
 */
export const createDirectory = async (
  path: string,
  name: string
): Promise<void> => {
  try {
    await invoke("create_directory", { path, name });
  } catch (error) {
    console.error("Failed to create directory:", error);
    throw error;
  }
};

/**
 * Get system information
 */
export const getSystemInfo = async (): Promise<SystemInfo> => {
  try {
    const info = await invoke<SystemInfo>("get_system_info");
    return info;
  } catch (error) {
    console.error("Failed to get system info:", error);
    throw error;
  }
};

/**
 * Copy a file or directory
 */
export const copyItem = async (
  srcPath: string,
  dstPath: string
): Promise<void> => {
  try {
    await invoke("copy_item", { srcPath, dstPath });
  } catch (error) {
    console.error("Failed to copy item:", error);
    throw error;
  }
};

/**
 * Move/rename a file or directory
 */
export const moveItem = async (
  srcPath: string,
  dstPath: string
): Promise<void> => {
  try {
    await invoke("move_item", { srcPath, dstPath });
  } catch (error) {
    console.error("Failed to move item:", error);
    throw error;
  }
};

/**
 * Delete a file or directory
 */
export const deleteItem = async (path: string): Promise<void> => {
  try {
    await invoke("delete_item", { path });
  } catch (error) {
    console.error("Failed to delete item:", error);
    throw error;
  }
};

/**
 * Rename a file or directory
 */
export const renameItem = async (
  oldPath: string,
  newName: string
): Promise<void> => {
  try {
    await invoke("rename_item", { oldPath, newName });
  } catch (error) {
    console.error("Failed to rename item:", error);
    throw error;
  }
};

/**
 * Create a new file
 */
export const createFile = async (path: string, name: string): Promise<void> => {
  try {
    await invoke("create_file", { path, name });
  } catch (error) {
    // Don't log here - let the caller handle logging to avoid duplicates
    throw error;
  }
};

/**
 * Get system paths for common directories
 */
export const getSystemPaths = async (): Promise<Record<string, string>> => {
  try {
    const paths = await invoke<Record<string, string>>("get_system_paths");
    // Normalize all system paths to remove Windows long path prefixes
    const normalizedPaths: Record<string, string> = {};
    for (const [key, path] of Object.entries(paths)) {
      normalizedPaths[key] = normalizeWindowsPath(path);
    }
    return normalizedPaths;
  } catch (error) {
    console.error("Failed to get system paths:", error);
    throw error;
  }
};

/**
 * Normalize Windows paths by removing long path prefixes and fixing separators
 */
function normalizeWindowsPath(path: string): string {
  if (!path) return path;

  let normalizedPath = path;

  // Remove Windows long path prefixes
  if (normalizedPath.startsWith("\\\\?\\UNC\\")) {
    // UNC path: \\?\UNC\server\share -> \\server\share
    normalizedPath = "\\\\" + normalizedPath.substring(8);
  } else if (normalizedPath.startsWith("\\\\?\\")) {
    // Regular long path: \\?\C:\path -> C:\path
    normalizedPath = normalizedPath.substring(4);
  }

  // Convert backslashes to forward slashes for consistency
  normalizedPath = normalizedPath.replace(/\\/g, "/");

  return normalizedPath;
}

/**
 * Resolve a path with alias support
 */
export const resolvePath = async (inputPath: string): Promise<string> => {
  try {
    const resolvedPath = await invoke<string>("resolve_path", { inputPath });
    // Normalize Windows paths to remove long path prefixes and fix separators
    return normalizeWindowsPath(resolvedPath);
  } catch (error) {
    console.error("Failed to resolve path:", error);
    throw error;
  }
};
