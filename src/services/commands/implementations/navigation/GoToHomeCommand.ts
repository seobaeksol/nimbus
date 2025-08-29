import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { resolvePath } from "../../ipc/file";

export class GoToHomeCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "go-to-home",
      label: "Go to Home",
      category: "Navigation",
      description: "Navigate to the home directory",
      icon: "󰏵", // nf-md-home
      shortcut: "Alt+Home",
    };
    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      const path = await resolvePath("Home");

      await this.navigateToPath(context.panelId, path);
      this.showSuccess("Navigated to Home");
    }, "Failed to go to Home");
  }
}
