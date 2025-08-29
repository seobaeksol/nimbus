import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { createDirectory } from "../../ipc/file";
import { navigateToPath } from "@/store/slices/panelSlice";
import { AppDispatch } from "@/store";

export type CreateFolderCommandOptions = {
  navigateToTarget?: boolean;
  showNotifications?: boolean;
};

export class CreateFolderCommand extends FileOperationCommand<CreateFolderCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "create-folder",
      label: "New Folder",
      category: "File",
      description: "Create a new folder in the current directory",
      icon: "folder-plus",
      shortcut: "Ctrl+Shift+N",
    };

    super(metadata, dispatch, dialogService);
  }

  canExecute(
    context: ExecutionContext,
    options?: CreateFolderCommandOptions
  ): boolean {
    return super.canExecute(context, options) && !!context.currentPath;
  }

  async execute(
    context: ExecutionContext,
    options: CreateFolderCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);

      const input = await this.dialogService.prompt(
        "Enter folder name:",
        "New Folder"
      );

      if (!input) {
        this.showInfo("Folder creation cancelled");
        return;
      }

      // Parse and resolve the path
      const { targetDir, fileName: folderName } = this.parseFileInput(
        input,
        context.currentPath
      );

      // Create the folder
      await createDirectory(targetDir, folderName);

      // Navigate to target directory if different from current
      if (options?.navigateToTarget) {
        this.dispatch(
          navigateToPath({ panelId: context.panelId, path: targetDir })
        );
      }

      // Show success notification
      if (options?.showNotifications) {
        this.showNotification(
          `Folder "${folderName}" created successfully`,
          "success"
        );
      }
    }, "Failed to create folder");
  }
}
