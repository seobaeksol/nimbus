import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { resolvePath } from "../../ipc/file";

export class GoToApplicationsCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "go-to-applications",
      label: "Go to Applications",
      category: "Navigation",
      description: "Navigate to the Applications folder",
      icon: "󰀋", // nf-md-application
      shortcut: "Alt+A",
    };
    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      const path = await resolvePath("Applications");

      await this.navigateToPath(context.panelId, path);
      this.showSuccess("Navigated to Applications");
    }, "Failed to go to Applications");
  }
}
