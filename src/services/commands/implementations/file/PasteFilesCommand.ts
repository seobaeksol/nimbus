import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";

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

      if (!context.clipboardHasFiles) {
        this.showWarning("No files in clipboard to paste");
        return;
      }

      // Implementation depends on clipboard state
      const panel = null; // TODO: Get panel from context
      if (!panel) return;

      // TODO: This would need to access clipboard state and perform paste operation
      this.showNotification("Paste operation not fully implemented", "warning");
      this.showSuccess("Pasted files from clipboard");
    }, "Failed to paste files");
  }
}
