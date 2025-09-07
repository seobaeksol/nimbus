import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { copyItem, moveItem } from "../../ipc/file";
import { 
  clearClipboard, 
  setFilesLoading, 
  refreshPanel 
} from "@/store/slices/panelSlice";

export class PasteFilesCommand extends FileOperationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "paste-files",
      label: "Paste",
      category: "File",
      description: "Paste files from clipboard",
      icon: "󰆐", // nf-md-content_paste
      shortcut: "Ctrl+V",
    };

    super(metadata, dispatch, dialogService);
  }

  canExecute(
    context: ExecutionContext,
    options?: Record<string, never>
  ): boolean {
    return super.canExecute(context, options) && context.clipboardHasFiles;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);

      if (!context.clipboardHasFiles || !context.clipboardState) {
        this.showWarning("No files in clipboard to paste");
        return;
      }

      const panel = context.panels[context.panelId];
      const clipboardState = context.clipboardState;
      const destinationPath = panel?.currentPath;
      const operation = clipboardState.operation;
      
      if (!operation) {
        this.showWarning("Invalid clipboard operation");
        return;
      }

      // Set loading state
      this.dispatch(setFilesLoading({ panelId: context.panelId, isLoading: true }));

      let successCount = 0;
      let errorCount = 0;
      const errors: string[] = [];

      try {
        for (const file of clipboardState.files) {
          const fileName = file.name;
          const destinationFilePath = `${destinationPath}/${fileName}`;

          try {
            if (operation === "copy") {
              await copyItem(file.path, destinationFilePath);
            } else if (operation === "cut") {
              await moveItem(file.path, destinationFilePath);
            }
            successCount++;
          } catch (error) {
            errorCount++;
            const errorMessage = error instanceof Error ? error.message : String(error);
            errors.push(`${fileName}: ${errorMessage}`);
            console.error(`Failed to ${operation} ${fileName}:`, error);
          }
        }

        // Clear clipboard after successful cut operation
        if (operation === "cut" && successCount > 0) {
          this.dispatch(clearClipboard());
        }

        // Refresh the destination panel
        this.dispatch(refreshPanel({ panelId: context.panelId }));

        // Show results
        if (errorCount === 0) {
          const fileWord = successCount === 1 ? "item" : "items";
          const operationWord = operation === "copy" ? "copied" : "moved";
          this.showSuccess(`Successfully ${operationWord} ${successCount} ${fileWord}`);
        } else if (successCount > 0) {
          this.showWarning(
            `${operation === "copy" ? "Copied" : "Moved"} ${successCount} items, ${errorCount} failed`
          );
          // Show first few errors
          const errorSummary = errors.slice(0, 3).join("\n");
          this.showNotification(`Errors:\n${errorSummary}`, "error");
        } else {
          this.showNotification(`Failed to ${operation} files`, "error");
          const errorSummary = errors.slice(0, 5).join("\n");
          this.showNotification(`All operations failed:\n${errorSummary}`, "error");
        }

      } finally {
        this.dispatch(setFilesLoading({ panelId: context.panelId, isLoading: false }));
      }
    }, "Failed to paste files");
  }
}
