import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { resolvePath } from "../../ipc/file";

export class GoToDownloadsCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "go-to-downloads",
      label: "Go to Downloads",
      category: "Navigation",
      description: "Navigate to the Downloads folder",
      icon: "download",
      shortcut: "Alt+Shift+L",
    };
    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      const path = await resolvePath("Downloads");

      await this.navigateToPath(context.panelId, path);
      this.showSuccess("Navigated to Downloads");
    }, "Failed to go to Downloads");
  }
}
