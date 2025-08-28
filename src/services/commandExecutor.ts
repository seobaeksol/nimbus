import { AppDispatch } from '../store';
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
  setDragOperation
} from '../store/slices/panelSlice';
import { FileService } from './fileService';
import { PathAliasService } from './pathAliasService';
import { FileInfo } from '../types';

export interface CommandContext {
  dispatch: AppDispatch;
  panels: Record<string, any>;
  activePanelId: string | null;
  clipboardState: any;
}

export interface CommandOptions {
  showNotifications?: boolean;
  navigateToTarget?: boolean;
  refreshPanels?: boolean;
}

/**
 * Centralized command executor for all application commands
 * Handles ALL business logic while keeping UI components pure
 */
export class CommandExecutor {
  private static dispatch: AppDispatch;
  private static context: CommandContext;

  /**
   * Initialize the command executor with application context
   */
  static initialize(context: CommandContext) {
    this.context = context;
    this.dispatch = context.dispatch;
  }

  /**
   * Update the current context (called when app state changes)
   */
  static updateContext(context: CommandContext) {
    this.context = context;
    this.dispatch = context.dispatch;
  }

  // =============================================================================
  // FILE OPERATIONS
  // =============================================================================

  /**
   * Create a new file with path resolution support
   */
  static async createFile(
    panelId: string, 
    input: string, 
    options: CommandOptions = {}
  ): Promise<void> {
    const { showNotifications = true, navigateToTarget = true } = options;
    
    try {
      const panel = this.context.panels[panelId];
      if (!panel) throw new Error(`Panel ${panelId} not found`);

      this.setLoadingState(panelId, true);
      
      // Parse and resolve the path
      const { targetDir, fileName } = this.parseFileInput(input, panel.currentPath);
      
      // Create the file
      await FileService.createFile(targetDir, fileName);
      
      // Navigate to target directory if different from current
      if (navigateToTarget) {
        const finalPath = targetDir === panel.currentPath ? panel.currentPath : targetDir;
        this.dispatch(navigateToPath({ panelId, path: finalPath }));
      }
      
      // Show success notification
      if (showNotifications) {
        this.showNotification(`File "${fileName}" created successfully`, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to create file:', error);
      
      if (showNotifications) {
        this.showNotification(`Failed to create file: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  /**
   * Create a new folder with path resolution support
   */
  static async createFolder(
    panelId: string, 
    input: string, 
    options: CommandOptions = {}
  ): Promise<void> {
    const { showNotifications = true, navigateToTarget = true } = options;
    
    try {
      const panel = this.context.panels[panelId];
      if (!panel) throw new Error(`Panel ${panelId} not found`);

      this.setLoadingState(panelId, true);
      
      // Parse and resolve the path
      const { targetDir, fileName: folderName } = this.parseFileInput(input, panel.currentPath);
      
      // Create the folder
      await FileService.createDirectory(targetDir, folderName);
      
      // Navigate to target directory if different from current
      if (navigateToTarget) {
        const finalPath = targetDir === panel.currentPath ? panel.currentPath : targetDir;
        this.dispatch(navigateToPath({ panelId, path: finalPath }));
      }
      
      // Show success notification
      if (showNotifications) {
        this.showNotification(`Folder "${folderName}" created successfully`, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to create folder:', error);
      
      if (showNotifications) {
        this.showNotification(`Failed to create folder: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  /**
   * Delete files with progress indication
   */
  static async deleteFiles(
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
        const panel = this.context.panels[panelId];
        if (panel) {
          this.dispatch(navigateToPath({ panelId, path: panel.currentPath }));
        }
      }
      
      if (showNotifications) {
        const message = files.length === 1 
          ? `"${files[0].name}" deleted successfully`
          : `${files.length} items deleted successfully`;
        this.showNotification(message, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to delete files:', error);
      
      if (showNotifications) {
        this.showNotification(`Failed to delete files: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  /**
   * Rename a file
   */
  static async renameFile(
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
        const panel = this.context.panels[panelId];
        if (panel) {
          this.dispatch(navigateToPath({ panelId, path: panel.currentPath }));
        }
      }
      
      if (showNotifications) {
        this.showNotification(`"${file.name}" renamed to "${newName}"`, 'success');
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to rename file:', error);
      
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

  /**
   * Copy files to clipboard
   */
  static copyFiles(panelId: string, files: FileInfo[]): void {
    this.dispatch(copyFilesToClipboard({ panelId, files }));
    this.showNotification(
      `${files.length} item${files.length > 1 ? 's' : ''} copied to clipboard`, 
      'info'
    );
  }

  /**
   * Cut files to clipboard
   */
  static cutFiles(panelId: string, files: FileInfo[]): void {
    this.dispatch(cutFilesToClipboard({ panelId, files }));
    this.showNotification(
      `${files.length} item${files.length > 1 ? 's' : ''} cut to clipboard`, 
      'info'
    );
  }

  /**
   * Paste files from clipboard with conflict resolution
   */
  static async pasteFiles(panelId: string, options: CommandOptions = {}): Promise<void> {
    const { showNotifications = true } = options;
    const { clipboardState } = this.context;
    
    if (!clipboardState.hasFiles || !clipboardState.files.length) {
      return;
    }

    // Don't paste to the same location for cut operations
    if (clipboardState.operation === 'cut' && clipboardState.sourcePanelId === panelId) {
      return;
    }

    try {
      const panel = this.context.panels[panelId];
      if (!panel) throw new Error(`Panel ${panelId} not found`);

      this.setLoadingState(panelId, true);
      
      const filesToPaste = clipboardState.files;
      const operation = clipboardState.operation === 'cut' ? 'move' : 'copy';
      
      // Create progress indicator
      const progressId = `paste-${Date.now()}`;
      const totalFiles = filesToPaste.length;
      
      this.dispatch(addProgressIndicator({
        id: progressId,
        operation,
        fileName: totalFiles > 1 ? `${totalFiles} items` : filesToPaste[0].name,
        progress: 0,
        totalFiles,
        currentFile: 0,
        isComplete: false
      }));

      for (let i = 0; i < filesToPaste.length; i++) {
        const file = filesToPaste[i];

        // Generate unique name to prevent conflicts
        const uniqueName = await this.generateUniqueFileName(panel.currentPath, file.name);
        
        // Update progress
        this.dispatch(updateProgressIndicator({
          id: progressId,
          updates: {
            fileName: uniqueName !== file.name ? `${file.name} → ${uniqueName}` : file.name,
            currentFile: i + 1,
            progress: ((i + 1) / totalFiles) * 100
          }
        }));

        const srcPath = file.path;
        const dstPath = panel.currentPath === '/' 
          ? `/${uniqueName}` 
          : `${panel.currentPath}/${uniqueName}`;

        if (operation === 'copy') {
          await FileService.copyItem(srcPath, dstPath);
        } else {
          await FileService.moveItem(srcPath, dstPath);
        }
      }

      // Mark progress as complete
      this.dispatch(updateProgressIndicator({
        id: progressId,
        updates: { isComplete: true, progress: 100 }
      }));

      setTimeout(() => {
        this.dispatch(removeProgressIndicator(progressId));
      }, 3000);

      // Refresh current panel
      this.dispatch(navigateToPath({ panelId, path: panel.currentPath }));
      
      // If it was a cut operation, refresh source panel and clear clipboard
      if (operation === 'move') {
        if (clipboardState.sourcePanelId && clipboardState.sourcePanelId !== panelId) {
          const sourcePanel = this.context.panels[clipboardState.sourcePanelId];
          if (sourcePanel) {
            this.dispatch(navigateToPath({ 
              panelId: clipboardState.sourcePanelId, 
              path: sourcePanel.currentPath 
            }));
          }
        }
        this.dispatch(clearClipboard());
      }
      
      if (showNotifications) {
        const action = operation === 'copy' ? 'copied' : 'moved';
        this.showNotification(
          `${totalFiles} item${totalFiles > 1 ? 's' : ''} ${action} successfully`, 
          'success'
        );
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to paste files:', error);
      
      if (showNotifications) {
        this.showNotification(`Failed to paste files: ${errorMessage}`, 'error');
      }
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  // =============================================================================
  // NAVIGATION OPERATIONS
  // =============================================================================

  /**
   * Navigate to a specific path
   */
  static async navigateToPath(panelId: string, inputPath: string): Promise<void> {
    try {
      // Resolve the path using PathAliasService
      const resolvedPath = await PathAliasService.resolvePath(inputPath);
      
      this.setLoadingState(panelId, true);
      this.dispatch(navigateToPath({ panelId, path: resolvedPath }));
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to navigate:', error);
      
      this.showNotification(`Cannot access path: ${errorMessage}`, 'error');
      throw error;
    } finally {
      this.setLoadingState(panelId, false);
    }
  }

  /**
   * Navigate to common directories
   */
  static async navigateToHome(panelId: string): Promise<void> {
    const homePath = await PathAliasService.resolvePath('~');
    await this.navigateToPath(panelId, homePath);
  }

  static async navigateToDocuments(panelId: string): Promise<void> {
    const documentsPath = await PathAliasService.resolvePath('Documents');
    await this.navigateToPath(panelId, documentsPath);
  }

  static async navigateToDownloads(panelId: string): Promise<void> {
    const downloadsPath = await PathAliasService.resolvePath('Downloads');
    await this.navigateToPath(panelId, downloadsPath);
  }

  static async navigateToDesktop(panelId: string): Promise<void> {
    const desktopPath = await PathAliasService.resolvePath('Desktop');
    await this.navigateToPath(panelId, desktopPath);
  }

  // =============================================================================
  // PANEL OPERATIONS
  // =============================================================================

  /**
   * Switch to a specific panel
   */
  static switchToPanel(panelId: string): void {
    this.dispatch(setActivePanel(panelId));
  }

  /**
   * Change grid layout
   */
  static changeGridLayout(rows: number, cols: number, name: string): void {
    this.dispatch(setGridLayout({ rows, cols, name }));
  }

  // =============================================================================
  // VIEW OPERATIONS
  // =============================================================================

  /**
   * Change view mode
   */
  static changeViewMode(panelId: string, viewMode: 'list' | 'grid' | 'details'): void {
    this.dispatch(setViewMode({ panelId, viewMode }));
  }

  /**
   * Change sorting
   */
  static changeSorting(panelId: string, sortBy: 'name' | 'size' | 'modified' | 'type'): void {
    this.dispatch(setSorting({ panelId, sortBy }));
  }

  // =============================================================================
  // SELECTION OPERATIONS
  // =============================================================================

  /**
   * Select files
   */
  static selectFiles(panelId: string, fileNames: string[], toggle: boolean = false): void {
    this.dispatch(selectFiles({ panelId, fileNames, toggle }));
  }

  /**
   * Select all files
   */
  static selectAllFiles(panelId: string): void {
    const panel = this.context.panels[panelId];
    if (panel) {
      const allFileNames = panel.files.map((file: FileInfo) => file.name);
      this.dispatch(selectFiles({ panelId, fileNames: allFileNames }));
    }
  }

  /**
   * Clear selection
   */
  static clearSelection(panelId: string): void {
    this.dispatch(selectFiles({ panelId, fileNames: [] }));
  }

  // =============================================================================
  // PRIVATE UTILITY METHODS
  // =============================================================================

  private static parseFileInput(input: string, currentPath: string): { targetDir: string; fileName: string } {
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

  private static async generateUniqueFileName(targetDir: string, originalName: string): Promise<string> {
    try {
      const existingFiles = await FileService.listDirectory(targetDir);
      const existingNames = new Set(existingFiles.map(f => f.name));
      
      if (!existingNames.has(originalName)) {
        return originalName;
      }
      
      // Parse file name and extension
      const lastDotIndex = originalName.lastIndexOf('.');
      let baseName: string;
      let extension: string;
      
      if (lastDotIndex > 0 && lastDotIndex < originalName.length - 1) {
        baseName = originalName.substring(0, lastDotIndex);
        extension = originalName.substring(lastDotIndex);
      } else {
        baseName = originalName;
        extension = '';
      }
      
      // Find unique name
      let counter = 2;
      let candidateName: string;
      
      do {
        candidateName = `${baseName} (${counter})${extension}`;
        counter++;
      } while (existingNames.has(candidateName) && counter < 1000);
      
      return candidateName;
    } catch (error) {
      console.warn('Could not check for file conflicts:', error);
      return originalName;
    }
  }

  private static setLoadingState(panelId: string, isLoading: boolean): void {
    this.dispatch(setLoading({ panelId, isLoading }));
  }

  private static showNotification(message: string, type: 'success' | 'error' | 'warning' | 'info'): void {
    this.dispatch(addNotification({
      id: `cmd-${Date.now()}`,
      message,
      type,
      autoClose: true,
      duration: type === 'success' ? 3000 : 5000
    }));
  }

  // === UI-Related Methods for FilePanel ===

  /**
   * Load directory contents for a panel
   */
  static async loadDirectory(panelId: string, path: string): Promise<void> {
    try {
      this.setLoadingState(panelId, true);
      const files = await FileService.listDirectory(path);
      this.dispatch(setFiles({ panelId, files }));
    } catch (error) {
      console.error('Failed to load directory:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to load directory';
      
      // Clear loading state first
      this.setLoadingState(panelId, false);
      
      // Show non-blocking notification
      this.dispatch(addNotification({
        id: `${panelId}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        message: `Cannot access directory "${path}": ${errorMessage}`,
        type: 'error',
        panelId,
        timestamp: Date.now(),
        autoClose: false,
        retryAction: 'loadDirectory',
        retryData: { path }
      }));
    }
  }

  /**
   * Navigate to directory when double-clicking folder
   */
  static navigateToDirectory(panelId: string, file: FileInfo): void {
    const panel = this.context.panels[panelId];
    if (!panel) return;

    const newPath = panel.currentPath === '/' 
      ? `/${file.name}` 
      : `${panel.currentPath}/${file.name}`;
    
    this.dispatch(navigateToPath({ panelId, path: newPath }));
  }

  /**
   * Navigate to parent directory
   */
  static navigateToParent(panelId: string): void {
    const panel = this.context.panels[panelId];
    if (!panel) return;

    const parentPath = panel.currentPath.split('/').slice(0, -1).join('/') || '/';
    this.dispatch(navigateToPath({ panelId, path: parentPath }));
  }

  /**
   * Handle address bar navigation
   */
  static async navigateToPath(panelId: string, inputPath: string): Promise<void> {
    try {
      // Resolve the path using the backend
      const resolvedPath = await FileService.resolvePath(inputPath);
      
      // Try to navigate to the resolved path
      this.setLoadingState(panelId, true);
      
      // If successful, update the panel
      this.dispatch(navigateToPath({ panelId, path: resolvedPath }));
      
    } catch (error) {
      // Let the error bubble up to the AddressBar component for display
      throw error;
    }
  }

  /**
   * Handle error display
   */
  static handleError(panelId: string, error: string): void {
    this.dispatch(setError({ panelId, error }));
  }

  // === Drag and Drop Methods ===

  /**
   * Start drag operation
   */
  static startDrag(panelId: string, file: FileInfo, isCtrlKey: boolean, event: React.DragEvent): void {
    const panel = this.context.panels[panelId];
    if (!panel) return;

    // If the dragged file is not selected, select only it
    const filesToDrag = panel.selectedFiles.includes(file.name) 
      ? panel.selectedFiles 
      : [file.name];

    if (!panel.selectedFiles.includes(file.name)) {
      this.dispatch(selectFiles({ panelId, fileNames: [file.name] }));
    }

    const operation = isCtrlKey ? 'copy' : 'move';
    this.dispatch(startDrag({ panelId, fileNames: filesToDrag, operation }));

    // Set drag data for compatibility
    event.dataTransfer.setData('text/plain', JSON.stringify({
      files: filesToDrag,
      sourcePanelId: panelId,
      operation,
    }));

    event.dataTransfer.effectAllowed = isCtrlKey ? 'copy' : 'move';
    
    // Create custom drag image
    const dragElement = event.currentTarget.cloneNode(true) as HTMLElement;
    dragElement.style.transform = 'rotate(5deg)';
    dragElement.style.opacity = '0.8';
    document.body.appendChild(dragElement);
    event.dataTransfer.setDragImage(dragElement, 10, 10);
    
    setTimeout(() => document.body.removeChild(dragElement), 0);
  }

  /**
   * End drag operation
   */
  static endDrag(): void {
    this.dispatch(endDrag());
  }

  /**
   * Update drag operation based on modifier keys
   */
  static updateDragOperation(isCtrlKey: boolean): void {
    const newOperation = isCtrlKey ? 'copy' : 'move';
    this.dispatch(setDragOperation(newOperation));
  }

  /**
   * Handle drop operation
   */
  static async handleDrop(targetPanelId: string, dragState: any): Promise<void> {
    let currentProgressId: string | null = null;

    try {
      this.setLoadingState(targetPanelId, true);
      
      const sourcePanelId = dragState.sourcePanelId!;
      const filesToTransfer = dragState.draggedFiles;
      const operation = dragState.dragOperation || 'move';

      // Get source and target panels
      const sourcePanel = this.context.panels[sourcePanelId];
      const targetPanel = this.context.panels[targetPanelId];
      const sourcePanelFiles = sourcePanel?.files || [];

      // Create progress indicator
      const progressId = `${operation}-${Date.now()}`;
      const totalFiles = filesToTransfer.length;
      currentProgressId = progressId;
      
      this.dispatch(addProgressIndicator({
        id: progressId,
        operation,
        fileName: totalFiles > 1 ? `${totalFiles} items` : filesToTransfer[0],
        progress: 0,
        totalFiles,
        currentFile: 0,
        isComplete: false
      }));

      for (let i = 0; i < filesToTransfer.length; i++) {
        const fileName = filesToTransfer[i];
        const sourceFile = sourcePanelFiles.find(f => f.name === fileName);
        if (!sourceFile) continue;

        // Generate unique name to prevent conflicts
        const uniqueName = await this.generateUniqueFileName(targetPanel.currentPath, fileName);

        // Update progress
        this.dispatch(updateProgressIndicator({
          id: progressId,
          updates: {
            fileName: uniqueName !== fileName ? `${fileName} → ${uniqueName}` : fileName,
            currentFile: i + 1,
            progress: ((i + 1) / totalFiles) * 100
          }
        }));

        const srcPath = sourceFile.path;
        // Construct destination path properly, handling root directory case
        const dstPath = targetPanel.currentPath === '/' 
          ? `/${uniqueName}` 
          : `${targetPanel.currentPath}/${uniqueName}`;

        if (operation === 'copy') {
          await FileService.copyItem(srcPath, dstPath);
        } else {
          await FileService.moveItem(srcPath, dstPath);
        }
      }

      // Mark progress as complete
      this.dispatch(updateProgressIndicator({
        id: progressId,
        updates: {
          isComplete: true,
          progress: 100
        }
      }));

      // Auto-remove completed progress after 3 seconds
      setTimeout(() => {
        this.dispatch(removeProgressIndicator(progressId));
      }, 3000);

      // Refresh destination panel
      await this.loadDirectory(targetPanelId, targetPanel.currentPath);
      
      // If it was a move operation, also refresh the source panel
      if (operation === 'move' && sourcePanelId !== targetPanelId) {
        // Trigger refresh of source panel by dispatching navigation action
        if (sourcePanel) {
          this.dispatch(navigateToPath({ panelId: sourcePanelId, path: sourcePanel.currentPath }));
        }
      }
      
      this.dispatch(endDrag());
      
    } catch (error) {
      console.error('Failed to transfer files:', error);
      
      // Update progress indicator with error
      if (currentProgressId) {
        this.dispatch(updateProgressIndicator({
          id: currentProgressId,
          updates: {
            error: error instanceof Error ? error.message : 'Unknown error',
            isComplete: false
          }
        }));
      }
      
      this.dispatch(addNotification({
        id: `${targetPanelId}-${Date.now()}`,
        message: `Failed to transfer files: ${error instanceof Error ? error.message : 'Unknown error'}`,
        type: 'error',
        panelId: targetPanelId,
        timestamp: Date.now(),
        autoClose: false
      }));
    } finally {
      this.setLoadingState(targetPanelId, false);
    }
  }

  // === Navigation to Common Directories ===

  /**
   * Navigate to common directories
   */
  static async goToHome(panelId: string): Promise<void> {
    const homePath = await PathAliasService.resolvePath('~');
    this.dispatch(navigateToPath({ panelId, path: homePath }));
  }

  static async goToDocuments(panelId: string): Promise<void> {
    const documentsPath = await PathAliasService.resolvePath('Documents');
    this.dispatch(navigateToPath({ panelId, path: documentsPath }));
  }

  static async goToDesktop(panelId: string): Promise<void> {
    const desktopPath = await PathAliasService.resolvePath('Desktop');
    this.dispatch(navigateToPath({ panelId, path: desktopPath }));
  }

  static async goToDownloads(panelId: string): Promise<void> {
    const downloadsPath = await PathAliasService.resolvePath('Downloads');
    this.dispatch(navigateToPath({ panelId, path: downloadsPath }));
  }

  static async goToApplications(panelId: string): Promise<void> {
    const applicationsPath = await PathAliasService.resolvePath('Applications');
    this.dispatch(navigateToPath({ panelId, path: applicationsPath }));
  }

  /**
   * Focus the address bar (delegates to UI)
   */
  static focusAddressBar(): void {
    // This still needs to be handled by the UI layer
    const event = new CustomEvent('command-palette-focus-address-bar');
    window.dispatchEvent(event);
  }

  /**
   * Go to path prompt (delegates to UI)
   */
  static promptGoToPath(): void {
    // This still needs to be handled by the UI layer for PromptDialog
    const event = new CustomEvent('command-palette-goto-path');
    window.dispatchEvent(event);
  }
}