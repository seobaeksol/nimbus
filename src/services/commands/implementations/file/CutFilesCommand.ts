import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { cutFilesToClipboard } from "@/store/slices/panelSlice";

export type CutFilesCommandOptions = {};

export class CutFilesCommand extends FileOperationCommand<CutFilesCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "cut-files",
      label: "Cut",
      category: "File",
      description: "Cut selected files to clipboard",
      icon: "scissors",
      shortcut: "Ctrl+X",
    };

    super(metadata, dispatch, dialogService);
  }

  protected getRequiredSelectionCount(): number {
    return 1;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);

      const selectedFiles = this.getSelectedFiles(context);
      if (selectedFiles.length === 0) {
        this.showWarning("No files selected for cutting");
        return;
      }

      this.dispatch(
        cutFilesToClipboard({ panelId: context.panelId, files: selectedFiles })
      );

      const fileWord = selectedFiles.length === 1 ? "item" : "items";
      this.showSuccess(`Cut ${selectedFiles.length} ${fileWord} to clipboard`);
    }, "Failed to cut files");
  }
}
