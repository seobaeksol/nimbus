import { BaseCommand } from "../../base/BaseCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { 
  startDrag, 
  endDrag, 
  setDragOperation 
} from "@/store/slices/panelSlice";
import { FileInfo } from "../../ipc/file";
import { DragState } from "@/store/slices/panelSlice";

export type StartDragCommandOptions = {
  file: FileInfo;
  isCopy: boolean;
  dragEvent?: React.DragEvent;
};

export class StartDragCommand extends BaseCommand<StartDragCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "start-drag",
      label: "Start Drag",
      category: "File",
      description: "Start dragging files",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(
    context: ExecutionContext,
    options: {
      file: FileInfo;
      isCopy: boolean;
      dragEvent?: React.DragEvent;
    }
  ): Promise<void> {
    const { file, isCopy, dragEvent } = options;
    const panel = context.panels[context.panelId];
    
    // If the file isn't selected, select only it
    let draggedFiles = [file.name];
    if (panel.selectedFiles.includes(file.name)) {
      // If the file is already selected, drag all selected files
      draggedFiles = panel.selectedFiles;
    }

    // Set drag data for external drops (if needed)
    if (dragEvent?.dataTransfer) {
      dragEvent.dataTransfer.effectAllowed = isCopy ? "copy" : "move";
      dragEvent.dataTransfer.setData("text/plain", draggedFiles.join("\n"));
    }

    // Update Redux state
    this.dispatch(
      startDrag({
        panelId: context.panelId,
        fileNames: draggedFiles,
        operation: isCopy ? "copy" : "move",
      })
    );
  }
}

export class EndDragCommand extends BaseCommand {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "end-drag",
      label: "End Drag",
      category: "File",
      description: "End dragging operation",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(): Promise<void> {
    this.dispatch(endDrag());
  }
}

export type UpdateDragOperationCommandOptions = {
  isCopy: boolean;
};

export class UpdateDragOperationCommand extends BaseCommand<UpdateDragOperationCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "update-drag-operation",
      label: "Update Drag Operation",
      category: "File",
      description: "Update drag operation type (copy/move)",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(
    _context: ExecutionContext,
    options: { isCopy: boolean }
  ): Promise<void> {
    this.dispatch(setDragOperation(options.isCopy ? "copy" : "move"));
  }
}

export type HandleDropCommandOptions = {
  dragState: DragState;
};

export class HandleDropCommand extends BaseCommand<HandleDropCommandOptions> {
  constructor(dispatch: AppDispatch, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: "handle-drop",
      label: "Handle Drop",
      category: "File",
      description: "Handle dropping files",
    };

    super(metadata, dispatch, dialogService);
  }

  async execute(
    context: ExecutionContext,
    options: { dragState: DragState }
  ): Promise<void> {
    const { dragState } = options;

    if (!dragState.isDragging || !dragState.sourcePanelId) {
      return;
    }

    const sourcePanel = context.panels[dragState.sourcePanelId];
    const targetPanel = context.panels[context.panelId];

    if (!sourcePanel || !targetPanel) {
      this.showWarning("Invalid source or target panel");
      return;
    }

    // Get the actual file objects being dragged
    const filesToMove = sourcePanel.files.filter((file) =>
      dragState.draggedFiles.includes(file.name)
    );

    if (filesToMove.length === 0) {
      this.showWarning("No files to move");
      return;
    }

    // Execute the appropriate command based on the operation
    if (dragState.dragOperation === "copy") {
      // Import and execute copy command dynamically
      const { CopyFilesCommand } = await import('./CopyFilesCommand');
      const copyCommand = new CopyFilesCommand(this.dispatch, this.dialogService);
      await copyCommand.execute({
        ...context,
        selectedFiles: filesToMove,
      });
    } else {
      // For move operation, cut from source and paste to target
      const { CutFilesCommand } = await import('./CutFilesCommand');
      const { PasteFilesCommand } = await import('./PasteFilesCommand');
      
      const cutCommand = new CutFilesCommand(this.dispatch, this.dialogService);
      await cutCommand.execute({
        ...context,
        panelId: dragState.sourcePanelId!,
        selectedFiles: filesToMove,
      });

      const pasteCommand = new PasteFilesCommand(this.dispatch, this.dialogService);
      await pasteCommand.execute(context);
    }

    // End the drag operation
    this.dispatch(endDrag());
  }
}
