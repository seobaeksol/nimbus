import { FileOperationCommand } from "../../base/FileOperationCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { navigateToPath, setFiles } from "@/store/slices/panelSlice";
import { AppDispatch } from "@/store";
import { listDirectory, resolvePath } from "../../ipc/file";

export type LoadDirectoryCommandOptions = {
  panelId: string;
  path?: string;
};

export class LoadDirectoryCommand extends FileOperationCommand<LoadDirectoryCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "load-directory",
      label: "Load Directory",
      category: "File",
      description: "Load a directory and its contents",
      icon: "folder",
      shortcut: "Ctrl+R",
    };

    super(metadata, dispatch, dialogService);
  }

  canExecute(_context: ExecutionContext): boolean {
    return true;
  }

  protected getRequiredSelectionCount(): number {
    return 1;
  }

  async execute(
    context: ExecutionContext,
    _options?: LoadDirectoryCommandOptions
  ): Promise<void> {
    await this.withErrorHandling(async () => {
      this.validatePanel(context);
      if (!_options?.panelId) throw new Error("Panel ID is required");

      let path = _options?.path || context.currentPath;
      path = await resolvePath(path);

      const files = await listDirectory(path);
      this.dispatch(navigateToPath({ panelId: _options.panelId, path }));
      this.dispatch(setFiles({ panelId: _options.panelId, files }));
    }, "Failed to load directory");
  }
}
