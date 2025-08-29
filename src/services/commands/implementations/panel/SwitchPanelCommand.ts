import { BaseCommand } from "../../base/BaseCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { setGridLayout } from "@/store/slices/panelSlice";

export type SwitchPanelCommandOptions = {
  rows: number;
  cols: number;
  name: string;
};

export class SwitchPanelCommand extends BaseCommand<SwitchPanelCommandOptions> {
  constructor(
    private panelId: string,
    dispatch: AppDispatch,
    dialogService: DialogService
  ) {
    const panelNumber = panelId.replace("panel-", "");
    const metadata: CommandMetadata = {
      id: `switch-to-${panelId}`,
      label: `Switch to Panel ${panelNumber}`,
      category: "Panel",
      description: `Switch focus to panel ${panelNumber}`,
      icon: "󰍯", // nf-md-tab
      shortcut: `Ctrl+${panelNumber}`,
    };

    super(metadata, dispatch, dialogService);
  }

  canExecute(context: ExecutionContext): boolean {
    if (!super.canExecute(context)) return false;

    // Only allow switching if panel exists and is different from current
    const panel = context.panels[this.panelId];
    return panel && this.panelId !== context.panelId;
  }

  async execute(
    context: ExecutionContext,
    options: SwitchPanelCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      const panel = context.panels[this.panelId];
      if (!panel) {
        this.showWarning(`Panel ${this.panelId} does not exist`);
        return;
      }

      if (this.panelId === context.panelId) {
        this.showInfo(`Already on panel ${this.panelId}`);
        return;
      }
      this.dispatch(setGridLayout(options));
      const panelNumber = this.panelId.replace("panel-", "");
      this.showSuccess(`Switched to panel ${panelNumber}`);
    }, `Failed to switch to panel ${this.panelId}`);
  }
}
