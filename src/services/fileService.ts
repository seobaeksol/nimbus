// File service for IPC communication with Rust backend

import { invoke } from "@tauri-apps/api/core";
import { FileInfo, SystemInfo } from "../types";

export class FileService {
  /**
   * List contents of a directory
   */
  static async listDirectory(path: string): Promise<FileInfo[]> {
    try {
      const files = await invoke<FileInfo[]>("list_dir", { path });
      return files;
    } catch (error) {
      console.error("Failed to list directory:", error);
      throw error;
    }
  }

  /**
   * Get detailed information about a file or directory
   */
  static async getFileInfo(path: string): Promise<FileInfo> {
    try {
      const info = await invoke<FileInfo>("get_file_info", { path });
      return info;
    } catch (error) {
      console.error("Failed to get file info:", error);
      throw error;
    }
  }

  /**
   * Create a new directory
   */
  static async createDirectory(path: string, name: string): Promise<void> {
    try {
      await invoke("create_directory", { path, name });
    } catch (error) {
      console.error("Failed to create directory:", error);
      throw error;
    }
  }

  /**
   * Get system information
   */
  static async getSystemInfo(): Promise<SystemInfo> {
    try {
      const info = await invoke<SystemInfo>("get_system_info");
      return info;
    } catch (error) {
      console.error("Failed to get system info:", error);
      throw error;
    }
  }

  /**
   * Copy a file or directory
   */
  static async copyItem(srcPath: string, dstPath: string): Promise<void> {
    try {
      await invoke("copy_item", { srcPath, dstPath });
    } catch (error) {
      console.error("Failed to copy item:", error);
      throw error;
    }
  }

  /**
   * Move/rename a file or directory
   */
  static async moveItem(srcPath: string, dstPath: string): Promise<void> {
    try {
      await invoke("move_item", { srcPath, dstPath });
    } catch (error) {
      console.error("Failed to move item:", error);
      throw error;
    }
  }

  /**
   * Delete a file or directory
   */
  static async deleteItem(path: string): Promise<void> {
    try {
      await invoke("delete_item", { path });
    } catch (error) {
      console.error("Failed to delete item:", error);
      throw error;
    }
  }

  /**
   * Rename a file or directory
   */
  static async renameItem(oldPath: string, newName: string): Promise<void> {
    try {
      await invoke("rename_item", { oldPath, newName });
    } catch (error) {
      console.error("Failed to rename item:", error);
      throw error;
    }
  }

  /**
   * Create a new file
   */
  static async createFile(path: string, name: string): Promise<void> {
    try {
      await invoke("create_file", { path, name });
    } catch (error) {
      console.error("Failed to create file:", error);
      throw error;
    }
  }

  /**
   * Greet function for testing IPC
   */
  static async greet(name: string): Promise<string> {
    try {
      return await invoke<string>("greet", { name });
    } catch (error) {
      console.error("Failed to greet:", error);
      throw error;
    }
  }
}