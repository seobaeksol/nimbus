import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { resolvePath } from "../../ipc/file";

export class GoToDocumentsCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "go-to-documents",
      label: "Go to Documents",
      category: "Navigation",
      description: "Navigate to the Documents folder",
      icon: "folder",
      shortcut: "Alt+D",
    };
    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      const path = await resolvePath("Documents");

      await this.navigateToPath(context.panelId, path);
      this.showSuccess("Navigated to Documents");
    }, "Failed to go to Documents");
  }
}
