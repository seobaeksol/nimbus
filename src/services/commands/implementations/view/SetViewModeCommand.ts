import { BaseCommand } from "../../base/BaseCommand";
import { CommandMetadata, ExecutionContext, ViewMode } from "../../types";
import { DialogService } from "../../services/DialogService";
import { setViewMode } from "@/store/slices/panelSlice";
import { AppDispatch } from "@/store";

export type SetViewModeCommandOptions = {
  panelId: string;
  viewMode?: ViewMode;
};

export class SetViewModeCommand extends BaseCommand<SetViewModeCommandOptions> {
  constructor(
    private readonly viewMode: ViewMode,
    dispatch: AppDispatch,
    dialogService: DialogService
  ) {
    const metadata: CommandMetadata = {
      id: `set-view-${viewMode}`,
      label: `${viewMode.charAt(0).toUpperCase() + viewMode.slice(1)} View`,
      category: "View",
      description: `Switch to ${viewMode} view mode`,
      icon:
        viewMode === "list" ? "list" : viewMode === "grid" ? "grid" : "table",
      shortcut:
        viewMode === "list"
          ? "Ctrl+1"
          : viewMode === "grid"
            ? "Ctrl+2"
            : "Ctrl+3",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(
    _context: ExecutionContext,
    options: SetViewModeCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      this.dispatch(
        setViewMode({ panelId: options.panelId, viewMode: options.viewMode || this.viewMode })
      );
      this.showSuccess(`Switched to ${options.viewMode || this.viewMode} view`);
    }, `Failed to set ${options.viewMode || this.viewMode} view`);
  }
}
