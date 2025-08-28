import { AppDispatch } from '../../../store';
import { 
  selectFiles,
  navigateToPath,
  setActivePanel,
  setGridLayout,
  setViewMode,
  setSorting,
  setLoading,
  setError,
  setFiles,
  addNotification,
  copyFilesToClipboard,
  cutFilesToClipboard,
  clearClipboard,
  addProgressIndicator,
  updateProgressIndicator,
  removeProgressIndicator,
  startDrag,
  endDrag,
  setDragOperation,
  Panel
} from '../../../store/slices/panelSlice';
import { FileService } from '../../fileService';
import { PathAliasService } from '../../pathAliasService';
import { FileInfo } from '../../../types';
import { ExecutionContext, NotificationType } from '../types';

export interface CommandOptions {
  showNotifications?: boolean;
  navigateToTarget?: boolean;
  refreshPanels?: boolean;
}

/**
 * Modern CommandExecutor as dependency-injectable service
 * Replaces static CommandExecutor with proper DI pattern
 */
export class CommandExecutorService {
  constructor(private dispatch: AppDispatch) {}

  // =============================================================================
  // FILE OPERATIONS
  // =============================================================================

  async createFile(
    panelId: string, 
    input: string, 
    options: CommandOptions = {}
  ): Promise<void> {
    const { showNotifications = true, navigateToTarget = true } = options;
    
    try {
      this.setLoadingState(panelId, true);
      
      // Parse and resolve the path
      const { targetDir, fileName } = this.parseFileInput(input, '/'); // TODO: Get current path from context
      
      // Create the file
      await FileService.createFile(targetDir, fileName);
      
      // Navigate to target directory if different from current
      if (navigateToTarget) {
        this.dispatch(navigateToPath({ panelId, path: targetDir }));
      }
      
      // Show success notification
      if (showNotifications) {
        this.showNotification(`File "${fileName}" created successfully`, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      if (showNotifications) {
        this.showNotification(`Failed to create file: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  async createFolder(
    panelId: string, 
    input: string, 
    options: CommandOptions = {}
  ): Promise<void> {
    const { showNotifications = true, navigateToTarget = true } = options;
    
    try {
      this.setLoadingState(panelId, true);
      
      // Parse and resolve the path
      const { targetDir, fileName: folderName } = this.parseFileInput(input, '/'); // TODO: Get current path from context
      
      // Create the folder
      await FileService.createDirectory(targetDir, folderName);
      
      // Navigate to target directory if different from current
      if (navigateToTarget) {
        this.dispatch(navigateToPath({ panelId, path: targetDir }));
      }
      
      // Show success notification
      if (showNotifications) {
        this.showNotification(`Folder "${folderName}" created successfully`, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      if (showNotifications) {
        this.showNotification(`Failed to create folder: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  async deleteFiles(
    panelId: string, 
    files: FileInfo[], 
    options: CommandOptions = {}
  ): Promise<void> {
    const { showNotifications = true, refreshPanels = true } = options;
    
    try {
      this.setLoadingState(panelId, true);
      
      // Create progress indicator for multiple files
      const progressId = files.length > 1 ? `delete-${Date.now()}` : null;
      if (progressId) {
        this.dispatch(addProgressIndicator({
          id: progressId,
          operation: 'delete',
          fileName: files.length > 1 ? `${files.length} items` : files[0].name,
          progress: 0,
          totalFiles: files.length,
          currentFile: 0,
          isComplete: false
        }));
      }

      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        
        if (progressId) {
          this.dispatch(updateProgressIndicator({
            id: progressId,
            updates: {
              fileName: file.name,
              currentFile: i + 1,
              progress: ((i + 1) / files.length) * 100
            }
          }));
        }

        await FileService.deleteItem(file.path);
      }

      if (progressId) {
        this.dispatch(updateProgressIndicator({
          id: progressId,
          updates: { isComplete: true, progress: 100 }
        }));
        
        setTimeout(() => {
          this.dispatch(removeProgressIndicator(progressId));
        }, 3000);
      }

      // Clear selection and refresh
      this.dispatch(selectFiles({ panelId, fileNames: [] }));
      if (refreshPanels) {
        // TODO: Get current path from context and refresh
      }
      
      if (showNotifications) {
        const message = files.length === 1 
          ? `"${files[0].name}" deleted successfully`
          : `${files.length} items deleted successfully`;
        this.showNotification(message, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      if (showNotifications) {
        this.showNotification(`Failed to delete files: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  async renameFile(
    panelId: string, 
    file: FileInfo, 
    newName: string, 
    options: CommandOptions = {}
  ): Promise<void> {
    const { showNotifications = true, refreshPanels = true } = options;
    
    try {
      this.setLoadingState(panelId, true);
      
      await FileService.renameItem(file.path, newName);
      
      if (refreshPanels) {
        // TODO: Get current path from context and refresh
      }
      
      if (showNotifications) {
        this.showNotification(`"${file.name}" renamed to "${newName}"`, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      if (showNotifications) {
        this.showNotification(`Failed to rename file: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  // =============================================================================
  // CLIPBOARD OPERATIONS
  // =============================================================================

  copyFiles(panelId: string, files: FileInfo[]): void {
    this.dispatch(copyFilesToClipboard({ panelId, files }));
    this.showNotification(
      `${files.length} item${files.length > 1 ? 's' : ''} copied to clipboard`, 
      'info'
    );
  }

  cutFiles(panelId: string, files: FileInfo[]): void {
    this.dispatch(cutFilesToClipboard({ panelId, files }));
    this.showNotification(
      `${files.length} item${files.length > 1 ? 's' : ''} cut to clipboard`, 
      'info'
    );
  }

  // =============================================================================
  // NAVIGATION OPERATIONS
  // =============================================================================

  async navigateToPath(panelId: string, inputPath: string): Promise<void> {
    try {
      // Resolve the path using PathAliasService
      const resolvedPath = await PathAliasService.resolvePath(inputPath);
      
      this.setLoadingState(panelId, true);
      this.dispatch(navigateToPath({ panelId, path: resolvedPath }));
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      this.showNotification(`Cannot access path: ${errorMessage}`, 'error');
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  async navigateToHome(panelId: string): Promise<void> {
    const homePath = await PathAliasService.resolvePath('~');
    await this.navigateToPath(panelId, homePath);
  }

  async navigateToDocuments(panelId: string): Promise<void> {
    const documentsPath = await PathAliasService.resolvePath('Documents');
    await this.navigateToPath(panelId, documentsPath);
  }

  async navigateToDownloads(panelId: string): Promise<void> {
    const downloadsPath = await PathAliasService.resolvePath('Downloads');
    await this.navigateToPath(panelId, downloadsPath);
  }

  async navigateToDesktop(panelId: string): Promise<void> {
    const desktopPath = await PathAliasService.resolvePath('Desktop');
    await this.navigateToPath(panelId, desktopPath);
  }

  async navigateToApplications(panelId: string): Promise<void> {
    const applicationsPath = await PathAliasService.resolvePath('Applications');
    await this.navigateToPath(panelId, applicationsPath);
  }

  focusAddressBar(): void {
    // Dispatch a custom event that the AddressBar component can listen to
    const event = new CustomEvent('focusAddressBar');
    window.dispatchEvent(event);
  }

  // =============================================================================
  // PANEL OPERATIONS
  // =============================================================================

  switchToPanel(panelId: string): void {
    this.dispatch(setActivePanel(panelId));
  }

  changeGridLayout(rows: number, cols: number, name: string): void {
    this.dispatch(setGridLayout({ rows, cols, name }));
  }

  // =============================================================================
  // VIEW OPERATIONS
  // =============================================================================

  changeViewMode(panelId: string, viewMode: 'list' | 'grid' | 'details'): void {
    this.dispatch(setViewMode({ panelId, viewMode }));
  }

  changeSorting(panelId: string, sortBy: 'name' | 'size' | 'modified' | 'type'): void {
    this.dispatch(setSorting({ panelId, sortBy }));
  }

  // =============================================================================
  // SELECTION OPERATIONS
  // =============================================================================

  selectFiles(panelId: string, fileNames: string[], toggle: boolean = false): void {
    this.dispatch(selectFiles({ panelId, fileNames, toggle }));
  }

  selectAllFiles(panelId: string, files: FileInfo[]): void {
    const allFileNames = files.map((file: FileInfo) => file.name);
    this.dispatch(selectFiles({ panelId, fileNames: allFileNames }));
  }

  clearSelection(panelId: string): void {
    this.dispatch(selectFiles({ panelId, fileNames: [] }));
  }

  // =============================================================================
  // UTILITY METHODS
  // =============================================================================

  private parseFileInput(input: string, currentPath: string): { targetDir: string; fileName: string } {
    const trimmedInput = input.trim();
    
    // Check if input is an absolute path
    const isAbsolute = trimmedInput.startsWith('/') || /^[A-Za-z]:\\/.test(trimmedInput);
    
    let fullPath: string;
    if (isAbsolute) {
      fullPath = trimmedInput;
    } else {
      // Relative path - resolve against current panel path
      fullPath = currentPath.endsWith('/') 
        ? currentPath + trimmedInput 
        : currentPath + '/' + trimmedInput;
    }
    
    // Normalize path separators
    fullPath = fullPath.replace(/\\/g, '/');
    
    // Split into directory and filename
    const lastSlashIndex = fullPath.lastIndexOf('/');
    if (lastSlashIndex === -1) {
      return { targetDir: currentPath, fileName: fullPath };
    }
    
    const targetDir = lastSlashIndex === 0 ? '/' : fullPath.substring(0, lastSlashIndex);
    const fileName = fullPath.substring(lastSlashIndex + 1);
    
    if (!fileName) {
      throw new Error('Filename cannot be empty');
    }
    
    return { targetDir, fileName };
  }

  private setLoadingState(panelId: string, isLoading: boolean): void {
    this.dispatch(setLoading({ panelId, isLoading }));
  }

  // Navigation and directory operations
  async loadDirectory(panelId: string, path: string): Promise<void> {
    try {
      this.setLoadingState(panelId, true);
      const files = await FileService.listDirectory(path);
      this.dispatch(navigateToPath({ panelId, path }));
      this.dispatch(setFiles({ panelId, files }));
    } catch (error: any) {
      this.showNotification(`Failed to load directory: ${error.message}`, 'error');
      this.dispatch(setError({ panelId, error: error.message }));
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  async navigateToDirectory(panelId: string, file: FileInfo): Promise<void> {
    if (file.fileType === 'directory') {
      await this.loadDirectory(panelId, file.path);
    }
  }

  async navigateToParent(panelId: string): Promise<void> {
    const panel = this.getPanelFromState(panelId);
    if (panel) {
      const parentPath = panel.currentPath.split('/').slice(0, -1).join('/') || '/';
      await this.loadDirectory(panelId, parentPath);
    }
  }

  async pasteFiles(panelId: string): Promise<void> {
    // Implementation depends on clipboard state
    const panel = this.getPanelFromState(panelId);
    if (!panel) return;

    // This would need to access clipboard state and perform paste operation
    this.showNotification('Paste operation not fully implemented', 'warning');
  }

  // Drag and drop operations  
  startDrag(panelId: string, file: FileInfo, isMultiSelect: boolean, event: React.DragEvent): void {
    // Implementation for drag start
    this.dispatch(startDrag({ panelId, file, isMultiSelect }));
  }

  endDrag(): void {
    this.dispatch(endDrag());
  }

  updateDragOperation(isCtrlPressed: boolean): void {
    this.dispatch(setDragOperation(isCtrlPressed ? 'copy' : 'move'));
  }

  async handleDrop(panelId: string, dragState: any): Promise<void> {
    // Implementation for drop handling
    this.showNotification('Drop operation not fully implemented', 'warning');
  }

  handleError(panelId: string, error: any): void {
    this.showNotification(`Error: ${error.message || error}`, 'error');
  }

  private getPanelFromState(panelId: string) {
    // This would need access to current Redux state
    // For now, return null to prevent errors
    return null;
  }

  private showNotification(message: string, type: NotificationType): void {
    this.dispatch(addNotification({
      id: `cmd-${Date.now()}`,
      message,
      type,
      autoClose: true,
      timestamp: Date.now(),
      duration: type === 'success' ? 3000 : 5000
    }));
  }
}