import { NavigationCommand } from "../../base/NavigationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { setAddressBarActive } from "@/store/slices/panelSlice";
import { AppDispatch } from "@/store";

export class FocusAddressBarCommand extends NavigationCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "focus-address-bar",
      label: "Focus Address Bar",
      category: "Navigation",
      description: "Focus the address bar for direct path input",
      icon: "navigation",
      shortcut: "Ctrl+L",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validateNavigationContext(context);

      this.dispatch(setAddressBarActive(context.panelId));
      this.showInfo("Address bar focused");
    }, "Failed to focus address bar");
  }
}
