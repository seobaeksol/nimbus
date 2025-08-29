import { BaseCommand } from "../../base/BaseCommand";
import { CommandMetadata, ExecutionContext } from "../../types";
import { DialogService } from "../../services/DialogService";
import { AppDispatch } from "@/store";
import { startDrag, endDrag } from "@/store/slices/panelSlice";
import { FileInfo } from "../../ipc/file";
import { DragState } from "@/store/slices/panelSlice";

export class StartDragCommand extends BaseCommand {
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
    throw new Error("Method not implemented.");
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

export class UpdateDragOperationCommand extends BaseCommand {
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
    throw new Error("Method not implemented.");
  }
}

export class HandleDropCommand extends BaseCommand {
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

    // End the drag operation
    this.dispatch(endDrag());
  }
}
