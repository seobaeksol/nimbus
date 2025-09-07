import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import {
  addProgressIndicator,
  removeProgressIndicator,
  selectFiles,
  updateProgressIndicator,
  refreshPanel,
} from "@/store/slices/panelSlice";
import { deleteItem } from "../../ipc/file";

export class DeleteFilesCommand extends FileOperationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "delete-files",
      label: "Delete",
      category: "File",
      description: "Delete selected files and folders",
      icon: "󰆩", // nf-md-delete
      shortcut: "Delete",
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
        this.showWarning("No files selected for deletion");
        return;
      }

      const confirmMessage = this.generateConfirmationMessage(
        "delete",
        selectedFiles
      );
      const confirmed = await this.dialogService.confirm(confirmMessage);

      if (!confirmed) {
        this.showInfo("Delete operation cancelled");
        return;
      }

      // Create progress indicator for multiple files
      const progressId =
        selectedFiles.length > 1 ? `delete-${Date.now()}` : null;
      if (progressId) {
        this.dispatch(
          addProgressIndicator({
            id: progressId,
            operation: "delete",
            fileName:
              selectedFiles.length > 1
                ? `${selectedFiles.length} items`
                : selectedFiles[0].name,
            progress: 0,
            totalFiles: selectedFiles.length,
            currentFile: 0,
            isComplete: false,
          })
        );
      }

      for (let i = 0; i < selectedFiles.length; i++) {
        const file = selectedFiles[i];

        if (progressId) {
          this.dispatch(
            updateProgressIndicator({
              id: progressId,
              updates: {
                fileName: file.name,
                currentFile: i + 1,
                progress: ((i + 1) / selectedFiles.length) * 100,
              },
            })
          );
        }

        await deleteItem(file.path);
      }

      if (progressId) {
        this.dispatch(
          updateProgressIndicator({
            id: progressId,
            updates: { isComplete: true, progress: 100 },
          })
        );

        setTimeout(() => {
          this.dispatch(removeProgressIndicator(progressId));
        }, 3000);
      }

      // Clear selection and refresh panel
      this.dispatch(selectFiles({ panelId: context.panelId, fileNames: [] }));
      this.dispatch(refreshPanel({ panelId: context.panelId }));

      const fileWord = selectedFiles.length === 1 ? "item" : "items";
      this.showSuccess(`Deleted ${selectedFiles.length} ${fileWord}`);
    }, "Failed to delete files");
  }
}
