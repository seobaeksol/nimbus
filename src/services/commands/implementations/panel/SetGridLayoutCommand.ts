import { BaseCommand } from "../../base/BaseCommand";
import {
  CommandMetadata,
  ExecutionContext,
  GridLayoutConfig,
} from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { setActivePanel } from "@/store/slices/panelSlice";

export class SetGridLayoutCommand extends BaseCommand {
  constructor(
    private layout: GridLayoutConfig,
    dispatch: AppDispatch,
    dialogService: DialogService
  ) {
    const metadata: CommandMetadata = {
      id: `set-layout-${layout.rows}x${layout.cols}`,
      label: layout.name,
      category: "Panel",
      description: `Switch to ${layout.name} layout`,
      icon: "layout-grid",
      shortcut:
        layout.rows === 1 && layout.cols === 1
          ? "Alt+1"
          : layout.rows === 1 && layout.cols === 2
            ? "Alt+2"
            : layout.rows === 2 && layout.cols === 2
              ? "Alt+3"
              : undefined,
    };
    super(metadata, dispatch, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(async () => {
      this.dispatch(setActivePanel(context.panelId));
      this.showSuccess(`Switched to ${this.layout.name} layout`);
    }, `Failed to set ${this.layout.name} layout`);
  }
}
