import { BaseCommand } from "../../base/BaseCommand";
import { CommandMetadata, ExecutionContext, SortBy } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { setSorting } from "@/store/slices/panelSlice";

export type SortByCommandOptions = {
  panelId: string;
  sortBy?: SortBy;
};

export class SortByCommand extends BaseCommand<SortByCommandOptions> {
  constructor(
    private readonly sortBy: SortBy,
    dispatch: AppDispatch,
    dialogService: DialogService
  ) {
    const metadata: CommandMetadata = {
      id: `sort-by-${sortBy}`,
      label: `Sort by ${sortBy.charAt(0).toUpperCase() + sortBy.slice(1)}`,
      category: "View",
      description: `Sort files by ${sortBy}`,
      icon:
        sortBy === "name"
          ? "nf-fa-font" // Nerd Font Font icon for name
          : sortBy === "size"
            ? "nf-oct-file_binary" // Nerd Font binary/file icon for size
            : sortBy === "modified"
              ? "nf-fa-calendar" // Nerd Font calendar icon for modified
              : "nf-oct-file_code", // Nerd Font code/file icon for type
      shortcut:
        sortBy === "name"
          ? "Ctrl+Shift+1"
          : sortBy === "size"
            ? "Ctrl+Shift+2"
            : sortBy === "modified"
              ? "Ctrl+Shift+3"
              : "Ctrl+Shift+4",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(
    _context: ExecutionContext,
    options: SortByCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(
      async () => {
        this.dispatch(
          setSorting({
            panelId: options.panelId,
            sortBy: options.sortBy || this.sortBy,
          })
        );
        this.showSuccess(`Sorted by ${options.sortBy || this.sortBy}`);
      },
      `Failed to sort by ${options.sortBy || this.sortBy}`
    );
  }
}
