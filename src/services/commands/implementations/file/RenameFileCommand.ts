import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { renameItem } from "../../ipc/file";
import { AppDispatch } from "@/store";

export type RenameFileCommandOptions = {
  newName?: string;
};

export class RenameFileCommand extends FileOperationCommand<RenameFileCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "rename-file",
      label: "Rename",
      category: "File",
      description: "Rename the selected file or folder",
      icon: "edit",
      shortcut: "F2",
    };

    super(metadata, dispatch, dialogService);
  }

  protected getRequiredSelectionCount(): number {
    return 1;
  }

  canExecute(context: ExecutionContext, options?: RenameFileCommandOptions): boolean {
    return super.canExecute(context, options) && context.selectedFiles.length === 1;
  }

  async execute(context: ExecutionContext, options?: RenameFileCommandOptions): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);

      const selectedFiles = this.getSelectedFiles(context);
      if (selectedFiles.length !== 1) {
        this.showWarning("Please select exactly one file to rename");
        return;
      }

      const file = selectedFiles[0];
      
      // Use provided newName or prompt for it
      const newName = options?.newName || await this.dialogService.prompt(
        "Enter new name:",
        file.name
      );

      if (!newName || newName === file.name) {
        this.showInfo("Rename operation cancelled");
        return;
      }

      await renameItem(file.path, newName);

      // TODO: Get current path from context and refresh

      this.showSuccess(`Renamed "${file.name}" to "${newName}"`);
    }, "Failed to rename file");
  }
}
