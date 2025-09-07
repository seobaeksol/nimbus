import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { createFile } from "../../ipc/file";
import { navigateToPath, refreshPanel } from "@/store/slices/panelSlice";
import { AppDispatch } from "@/store";

export type CreateFileCommandOptions = {
  navigateToTarget?: boolean;
};

export class CreateFileCommand extends FileOperationCommand<CreateFileCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "create-file",
      label: "New File",
      category: "File",
      description: "Create a new file in the current directory",
      icon: "󰂰", // nf-md-file_plus
      shortcut: "Ctrl+N",
    };

    super(metadata, dispatch, dialogService);
  }

  canExecute(
    context: ExecutionContext,
    options?: CreateFileCommandOptions
  ): boolean {
    return super.canExecute(context, options) && !!context.currentPath;
  }

  async execute(
    context: ExecutionContext,
    options: CreateFileCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);

      const input = await this.dialogService.prompt(
        "Enter file name:",
        "new-file.txt"
      );

      if (!input) {
        this.showInfo("File creation cancelled");
        return;
      }

      // Parse and resolve the path
      const { targetDir, fileName } = this.parseFileInput(
        input,
        context.currentPath
      );

      // Create the file
      await createFile(targetDir, fileName);

      // Navigate to target directory if different from current
      if (options?.navigateToTarget) {
        this.dispatch(
          navigateToPath({ panelId: context.panelId, path: targetDir })
        );
      }

      // Refresh the panel to show the new file
      this.dispatch(refreshPanel({ panelId: context.panelId }));

      this.showSuccess(`Created file: ${fileName}`);
    }, "Failed to create file");
  }
}
