import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { copyFilesToClipboard } from "@/store/slices/panelSlice";
import { AppDispatch } from "@/store";

export type CopyFilesCommandOptions = {
  includeSubdirectories?: boolean;
};

export class CopyFilesCommand extends FileOperationCommand<CopyFilesCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "copy-files",
      label: "Copy",
      category: "File",
      description: "Copy selected files to clipboard",
      icon: "󰆏", // nf-md-content_copy
      shortcut: "Ctrl+C",
    };

    super(metadata, dispatch, dialogService);
  }

  protected getRequiredSelectionCount(): number {
    return 1;
  }

  async execute(
    context: ExecutionContext,
    _options?: CopyFilesCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);

      const selectedFiles = this.getSelectedFiles(context);
      if (selectedFiles.length === 0) {
        this.dialogService.showNotification(
          "No files selected for copying",
          "warning"
        );
        return;
      }

      this.dispatch(
        copyFilesToClipboard({ panelId: context.panelId, files: selectedFiles })
      );
      this.showNotification(
        `${selectedFiles.length} item${selectedFiles.length > 1 ? "s" : ""} copied to clipboard`,
        "info"
      );

      const fileWord = selectedFiles.length === 1 ? "item" : "items";
      this.showSuccess(
        `Copied ${selectedFiles.length} ${fileWord} to clipboard`
      );
    }, "Failed to copy files");
  }
}
