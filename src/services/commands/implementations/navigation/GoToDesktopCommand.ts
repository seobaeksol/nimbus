import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { resolvePath } from "../../ipc/file";
import { AppDispatch } from "@/store";

export class GoToDesktopCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "go-to-desktop",
      label: "Go to Desktop",
      category: "Navigation",
      description: "Navigate to the Desktop folder",
      icon: "󰈷", // nf-md-monitor
      shortcut: "Alt+Shift+D",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      const path = await resolvePath("Desktop");

      await this.navigateToPath(context.panelId, path);
      this.showSuccess("Navigated to Desktop");
    }, "Failed to go to Desktop");
  }
}
