import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";

export class GoToPathCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "go-to-path",
      label: "Go to Path",
      category: "Navigation",
      description: "Navigate to a specific path",
      icon: "navigation",
      shortcut: "Ctrl+G",
    };
    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      const path = await this.dialogService.prompt(
        "Enter path:",
        context.currentPath
      );

      if (!path) {
        this.showInfo("Navigation cancelled");
        return;
      }

      await this.navigateWithValidation(context, path);
      this.showSuccess(`Navigated to: ${path}`);
    }, "Failed to navigate to path");
  }
}
