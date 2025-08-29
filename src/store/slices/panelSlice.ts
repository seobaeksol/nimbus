import { FileInfo } from "@/services/commands/ipc/file";
import { createSlice, PayloadAction } from "@reduxjs/toolkit";

export interface Panel {
  id: string;
  currentPath: string;
  files: FileInfo[];
  selectedFiles: string[];
  isLoading: boolean;
  error: string | null;
  viewMode: "list" | "grid" | "details";
  sortBy: "name" | "size" | "modified" | "type";
  sortOrder: "asc" | "desc";
  isAddressBarActive: boolean;
}

export interface GridLayout {
  rows: number;
  cols: number;
  name: string;
}

export interface DragState {
  isDragging: boolean;
  draggedFiles: string[];
  sourcePanelId: string | null;
  dragOperation: "move" | "copy" | null;
}

export interface ClipboardState {
  hasFiles: boolean;
  files: FileInfo[];
  sourcePanelId: string | null;
  operation: "copy" | "cut" | null;
  timestamp: number;
}

export interface ProgressInfo {
  id: string;
  operation: "copy" | "move" | "delete";
  fileName: string;
  progress: number; // 0-100
  totalFiles: number;
  currentFile: number;
  isComplete: boolean;
  error?: string;
}

export interface NotificationInfo {
  id: string;
  message: string;
  type: "error" | "warning" | "info" | "success";
  panelId?: string;
  timestamp: number;
  autoClose?: boolean;
  duration?: number;
  retryAction?: string;
  retryData?: any;
}

export interface PanelState {
  panels: { [key: string]: Panel };
  activePanelId: string | null;
  gridLayout: GridLayout;
  panelOrder: string[];
  presetLayouts: GridLayout[];
  dragState: DragState;
  clipboardState: ClipboardState;
  progressIndicators: ProgressInfo[];
  notifications: NotificationInfo[];
}

const defaultLayouts: GridLayout[] = [
  { rows: 1, cols: 1, name: "1x1 (Single Panel)" },
  { rows: 1, cols: 2, name: "1x2 (Classic Dual)" },
  { rows: 2, cols: 2, name: "2x2 (Quad)" },
  { rows: 2, cols: 3, name: "2x3 (Six Panel)" },
  { rows: 3, cols: 2, name: "3x2 (Vertical)" },
];

const createDefaultPanel = (id: string, path: string = "/"): Panel => ({
  id,
  currentPath: path,
  files: [],
  selectedFiles: [],
  isLoading: false,
  error: null,
  viewMode: "list",
  sortBy: "name",
  sortOrder: "asc",
  isAddressBarActive: false,
});

const initialState: PanelState = {
  panels: {
    "panel-1": createDefaultPanel("panel-1", "/"),
    "panel-2": createDefaultPanel("panel-2", "/"),
  },
  activePanelId: "panel-1",
  gridLayout: { rows: 1, cols: 2, name: "1x2 (Classic Dual)" },
  panelOrder: ["panel-1", "panel-2"],
  presetLayouts: defaultLayouts,
  dragState: {
    isDragging: false,
    draggedFiles: [],
    sourcePanelId: null,
    dragOperation: null,
  },
  clipboardState: {
    hasFiles: false,
    files: [],
    sourcePanelId: null,
    operation: null,
    timestamp: 0,
  },
  progressIndicators: [],
  notifications: [],
};

const panelSlice = createSlice({
  name: "panels",
  initialState,
  reducers: {
    setGridLayout: (state, action: PayloadAction<GridLayout>) => {
      const newLayout = action.payload;
      const requiredPanels = newLayout.rows * newLayout.cols;

      state.gridLayout = newLayout;

      // Add panels if needed
      while (state.panelOrder.length < requiredPanels) {
        const newPanelId = `panel-${state.panelOrder.length + 1}`;
        state.panels[newPanelId] = createDefaultPanel(newPanelId);
        state.panelOrder.push(newPanelId);
      }

      // Remove excess panels
      while (state.panelOrder.length > requiredPanels) {
        const removedId = state.panelOrder.pop()!;
        delete state.panels[removedId];
        if (state.activePanelId === removedId) {
          state.activePanelId = state.panelOrder[0] || null;
        }
      }
    },

    setActivePanel: (state, action: PayloadAction<string>) => {
      if (state.panels[action.payload]) {
        state.activePanelId = action.payload;
      }
    },

    navigateToPath: (
      state,
      action: PayloadAction<{ panelId: string; path: string }>
    ) => {
      const { panelId, path } = action.payload;
      if (state.panels[panelId]) {
        state.panels[panelId].currentPath = path;
        state.panels[panelId].selectedFiles = [];
      }
    },

    setFiles: (
      state,
      action: PayloadAction<{ panelId: string; files: FileInfo[] }>
    ) => {
      const { panelId, files } = action.payload;
      if (state.panels[panelId]) {
        state.panels[panelId].files = files;
        state.panels[panelId].isLoading = false;
        state.panels[panelId].error = null;
      }
    },

    setLoading: (
      state,
      action: PayloadAction<{ panelId: string; isLoading: boolean }>
    ) => {
      const { panelId, isLoading } = action.payload;
      if (state.panels[panelId]) {
        state.panels[panelId].isLoading = isLoading;
      }
    },

    setError: (
      state,
      action: PayloadAction<{ panelId: string; error: string | null }>
    ) => {
      const { panelId, error } = action.payload;
      if (state.panels[panelId]) {
        state.panels[panelId].error = error;
        state.panels[panelId].isLoading = false;
      }
    },

    selectFiles: (
      state,
      action: PayloadAction<{
        panelId: string;
        fileNames: string[];
        toggle?: boolean;
      }>
    ) => {
      const { panelId, fileNames, toggle = false } = action.payload;
      if (state.panels[panelId]) {
        if (toggle) {
          fileNames.forEach((fileName) => {
            const index = state.panels[panelId].selectedFiles.indexOf(fileName);
            if (index >= 0) {
              state.panels[panelId].selectedFiles.splice(index, 1);
            } else {
              state.panels[panelId].selectedFiles.push(fileName);
            }
          });
        } else {
          state.panels[panelId].selectedFiles = fileNames;
        }
      }
    },

    setViewMode: (
      state,
      action: PayloadAction<{ panelId: string; viewMode: Panel["viewMode"] }>
    ) => {
      const { panelId, viewMode } = action.payload;
      if (state.panels[panelId]) {
        state.panels[panelId].viewMode = viewMode;
      }
    },

    setSorting: (
      state,
      action: PayloadAction<{
        panelId: string;
        sortBy: Panel["sortBy"];
        sortOrder?: Panel["sortOrder"];
      }>
    ) => {
      const { panelId, sortBy, sortOrder } = action.payload;
      if (state.panels[panelId]) {
        state.panels[panelId].sortBy = sortBy;
        if (sortOrder) {
          state.panels[panelId].sortOrder = sortOrder;
        } else {
          // Toggle sort order if same field
          state.panels[panelId].sortOrder =
            state.panels[panelId].sortBy === sortBy &&
            state.panels[panelId].sortOrder === "asc"
              ? "desc"
              : "asc";
        }
      }
    },

    // Drag and drop actions
    startDrag: (
      state,
      action: PayloadAction<{
        panelId: string;
        fileNames: string[];
        operation: "move" | "copy";
      }>
    ) => {
      const { panelId, fileNames, operation } = action.payload;
      state.dragState = {
        isDragging: true,
        draggedFiles: fileNames,
        sourcePanelId: panelId,
        dragOperation: operation,
      };
    },

    endDrag: (state) => {
      state.dragState = {
        isDragging: false,
        draggedFiles: [],
        sourcePanelId: null,
        dragOperation: null,
      };
    },

    setDragOperation: (state, action: PayloadAction<"move" | "copy">) => {
      if (state.dragState.isDragging) {
        state.dragState.dragOperation = action.payload;
      }
    },

    // Progress indicator actions
    addProgressIndicator: (state, action: PayloadAction<ProgressInfo>) => {
      state.progressIndicators.push(action.payload);
    },

    updateProgressIndicator: (
      state,
      action: PayloadAction<{ id: string; updates: Partial<ProgressInfo> }>
    ) => {
      const { id, updates } = action.payload;
      const index = state.progressIndicators.findIndex((p) => p.id === id);
      if (index >= 0) {
        state.progressIndicators[index] = {
          ...state.progressIndicators[index],
          ...updates,
        };
      }
    },

    removeProgressIndicator: (state, action: PayloadAction<string>) => {
      const id = action.payload;
      state.progressIndicators = state.progressIndicators.filter(
        (p) => p.id !== id
      );
    },

    clearCompletedProgress: (state) => {
      state.progressIndicators = state.progressIndicators.filter(
        (p) => !p.isComplete && !p.error
      );
    },

    // Clipboard actions
    copyFilesToClipboard: (
      state,
      action: PayloadAction<{ panelId: string; files: FileInfo[] }>
    ) => {
      const { panelId, files } = action.payload;
      state.clipboardState = {
        hasFiles: true,
        files,
        sourcePanelId: panelId,
        operation: "copy",
        timestamp: Date.now(),
      };
    },

    cutFilesToClipboard: (
      state,
      action: PayloadAction<{ panelId: string; files: FileInfo[] }>
    ) => {
      const { panelId, files } = action.payload;
      state.clipboardState = {
        hasFiles: true,
        files,
        sourcePanelId: panelId,
        operation: "cut",
        timestamp: Date.now(),
      };
    },

    clearClipboard: (state) => {
      state.clipboardState = {
        hasFiles: false,
        files: [],
        sourcePanelId: null,
        operation: null,
        timestamp: 0,
      };
    },

    // Notification actions
    addNotification: (state, action: PayloadAction<NotificationInfo>) => {
      state.notifications.push(action.payload);
      // Keep only the latest 5 notifications to prevent memory issues
      if (state.notifications.length > 5) {
        state.notifications = state.notifications.slice(-5);
      }
    },

    removeNotification: (state, action: PayloadAction<string>) => {
      const id = action.payload;
      state.notifications = state.notifications.filter((n) => n.id !== id);
    },

    clearAllNotifications: (state) => {
      state.notifications = [];
    },

    clearPanelNotifications: (state, action: PayloadAction<string>) => {
      const panelId = action.payload;
      state.notifications = state.notifications.filter(
        (n) => n.panelId !== panelId
      );
    },

    setAddressBarActive: (state, action: PayloadAction<string>) => {
      state.panels[action.payload].isAddressBarActive = true;
      Object.keys(state.panels).forEach((panelId) => {
        if (panelId !== action.payload) {
          state.panels[panelId].isAddressBarActive = false;
        }
      });
    },
  },
});

export const {
  setGridLayout,
  setActivePanel,
  navigateToPath,
  setFiles,
  setLoading,
  setError,
  selectFiles,
  setViewMode,
  setSorting,
  startDrag,
  endDrag,
  setDragOperation,
  addProgressIndicator,
  updateProgressIndicator,
  removeProgressIndicator,
  clearCompletedProgress,
  copyFilesToClipboard,
  cutFilesToClipboard,
  clearClipboard,
  addNotification,
  removeNotification,
  clearAllNotifications,
  clearPanelNotifications,
  setAddressBarActive,
} = panelSlice.actions;

export default panelSlice.reducer;
