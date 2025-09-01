import React, { useEffect, useState } from "react";
import { Panel, selectFiles } from "../../store/slices/panelSlice";
import { useAppDispatch, useAppSelector } from "../../store";
import { useCommands } from "../../hooks/useCommands";
import ContextMenu, { ContextMenuItem } from "../common/ContextMenu";
import ConfirmDialog from "../common/ConfirmDialog";
import PromptDialog from "../common/PromptDialog";
import AddressBar from "../common/AddressBar";
import NotificationContainer from "../common/NotificationContainer";
import FileIcon from "../common/FileIcon";
import ArchiveBrowser from "../common/ArchiveBrowser";
import "./FilePanel.css";
import { FileInfo } from "@/services/commands/ipc/file";
import { formatFileSize, formatFileDate, isArchiveFile } from "../../utils/fileIcons";

interface FilePanelProps {
  panel: Panel;
  isActive?: boolean;
  addressBarActive?: boolean;
  onAddressBarFocus?: () => void;
}

const FilePanel: React.FC<FilePanelProps> = ({
  panel,
  isActive = false,
  addressBarActive = false,
  onAddressBarFocus,
}) => {
  const dispatch = useAppDispatch();
  const { dragState, clipboardState } = useAppSelector((state) => state.panels);
  const { executeCommand } = useCommands();

  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    selectedFiles: FileInfo[];
  } | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    variant?: "default" | "danger";
    onConfirm: () => void;
  }>({
    isOpen: false,
    title: "",
    message: "",
    onConfirm: () => {},
  });
  const [promptDialog, setPromptDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    defaultValue?: string;
    placeholder?: string;
    onConfirm: (value: string) => void;
  }>({
    isOpen: false,
    title: "",
    message: "",
    defaultValue: "",
    onConfirm: () => {},
  });
  const [isDragOver, setIsDragOver] = useState(false);
  const [dragOverCounter, setDragOverCounter] = useState(0);
  const [archiveBrowser, setArchiveBrowser] = useState<{
    isOpen: boolean;
    archivePath: string;
  }>({
    isOpen: false,
    archivePath: "",
  });

  useEffect(() => {
    console.warn("Loading directory:", panel.currentPath);
    executeCommand("load-directory", {
      panelId: panel.id,
      path: panel.currentPath,
    });
  }, [panel.currentPath]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Only handle keyboard events if this panel is active
      if (!isActive) return;

      const selectedFileInfos = panel.files.filter((f) =>
        panel.selectedFiles.includes(f.name)
      );

      // Handle Ctrl+V (paste) even without selection
      if ((event.ctrlKey || event.metaKey) && event.key === "v") {
        event.preventDefault();
        executeCommand("paste-files");
        return;
      }

      // For other operations, require file selection
      if (selectedFileInfos.length === 0) return;

      switch (event.key) {
        case "Delete":
          event.preventDefault();
          const message =
            selectedFileInfos.length === 1
              ? `Are you sure you want to delete "${selectedFileInfos[0].name}"?`
              : `Are you sure you want to delete ${selectedFileInfos.length} selected items?`;

          showConfirmDialog(
            "Confirm Delete",
            message,
            () => executeCommand("delete-files"),
            "danger"
          );
          break;

        case "F2":
          event.preventDefault();
          if (selectedFileInfos.length === 1) {
            setPromptDialog({
              isOpen: true,
              title: "Rename File",
              message: "Enter new name:",
              defaultValue: selectedFileInfos[0].name,
              onConfirm: (newName: string) => {
                if (newName && newName !== selectedFileInfos[0].name) {
                  executeCommand("rename-file", { newName });
                }
                setPromptDialog({ ...promptDialog, isOpen: false });
              },
            });
          }
          break;

        case "c":
          if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            executeCommand("copy-files");
          }
          break;

        case "x":
          if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            executeCommand("cut-files");
          }
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [panel.files, panel.selectedFiles, panel.currentPath, isActive]);

  // NOTE: Command Palette events are now handled by CommandExecutor
  // All business logic has been moved to centralized command execution

  const handleFileClick = (file: FileInfo, event: React.MouseEvent) => {
    if (event.ctrlKey || event.metaKey) {
      // Multi-select with Ctrl/Cmd
      dispatch(
        selectFiles({
          panelId: panel.id,
          fileNames: [file.name],
          toggle: true,
        })
      );
    } else if (event.shiftKey && panel.selectedFiles.length > 0) {
      // Range select with Shift
      const fileNames = panel.files.map((f) => f.name);
      const lastSelected = panel.selectedFiles[panel.selectedFiles.length - 1];
      const lastIndex = fileNames.indexOf(lastSelected);
      const currentIndex = fileNames.indexOf(file.name);

      const start = Math.min(lastIndex, currentIndex);
      const end = Math.max(lastIndex, currentIndex);
      const rangeFiles = fileNames.slice(start, end + 1);

      dispatch(selectFiles({ panelId: panel.id, fileNames: rangeFiles }));
    } else {
      // Single select
      dispatch(selectFiles({ panelId: panel.id, fileNames: [file.name] }));
    }
  };

  const handleFileDoubleClick = (file: FileInfo) => {
    if (file.file_type === "Directory") {
      executeCommand("load-directory", {
        panelId: panel.id,
        path: file.path,
      });
    } else if (isArchiveFile(file.name)) {
      // Open archive browser for supported archive formats
      setArchiveBrowser({
        isOpen: true,
        archivePath: file.path,
      });
    }
  };

  const handleArchiveClose = () => {
    setArchiveBrowser({
      isOpen: false,
      archivePath: "",
    });
  };

  const handleArchiveExtract = async (entries: string[], _destination: string) => {
    // TODO: Implement extraction UI - for now, use current panel directory
    const extractPath = panel.currentPath;
    
    try {
      await executeCommand("extract-archive", {
        archivePath: archiveBrowser.archivePath,
        destination: extractPath,
        options: {
          preserve_paths: true,
          overwrite_policy: "ask",
          entries: entries.length > 0 ? entries : undefined,
        }
      });
    } catch (error) {
      console.error("Failed to extract archive:", error);
    }
  };

  const handleArchivePreview = async (entryPath: string) => {
    try {
      // Extract entry to memory for preview
      const data = await executeCommand("extract-entry-to-memory", {
        archivePath: archiveBrowser.archivePath,
        entryPath,
        maxSize: 1024 * 1024, // 1MB limit
      });
      
      // TODO: Show preview dialog with extracted data
      console.log("Preview data:", data);
    } catch (error) {
      console.error("Failed to preview archive entry:", error);
    }
  };

  const handleBackClick = () => {
    // TODO: Utility function for getting parent directory
    let parentPath = panel.currentPath.split("/").slice(0, -1).join("/") || "/";
    if (!parentPath.includes("/")) parentPath = "/";
    executeCommand("load-directory", {
      panelId: panel.id,
      path: parentPath,
    });
  };

  const handleAddressBarNavigate = async (inputPath: string) => {
    try {
      executeCommand("load-directory", {
        panelId: panel.id,
        path: inputPath,
      });
    } catch (error) {
      // Let the error bubble up to the AddressBar component for display
      throw error;
    }
  };

  const handleAddressBarError = (error: string) => {
    // TODO
    console.error("Address bar error:", error);
  };

  const handleAddressBarFocus = () => {
    onAddressBarFocus?.(); // Reset the active trigger
  };

  const handleRightClick = (e: React.MouseEvent, file: FileInfo) => {
    e.preventDefault();

    // Select the right-clicked file if it's not already selected
    if (!panel.selectedFiles.includes(file.name)) {
      dispatch(selectFiles({ panelId: panel.id, fileNames: [file.name] }));
    }

    const selectedFileInfos = panel.files.filter(
      (f) => panel.selectedFiles.includes(f.name) || f.name === file.name
    );

    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      selectedFiles: selectedFileInfos,
    });
  };

  const showConfirmDialog = (
    title: string,
    message: string,
    onConfirm: () => void,
    variant: "default" | "danger" = "default"
  ) => {
    setConfirmDialog({
      isOpen: true,
      title,
      message,
      variant,
      onConfirm,
    });
  };

  // Drag and Drop handlers
  const handleDragStart = (e: React.DragEvent, file: FileInfo) => {
    executeCommand("start-drag", { file, isCopy: e.ctrlKey, dragEvent: e });
  };

  const handleDragEnd = () => {
    executeCommand("end-drag");
    setIsDragOver(false);
    setDragOverCounter(0);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = e.ctrlKey ? "copy" : "move";

    // Update drag operation based on modifier keys
    if (dragState.isDragging && dragState.sourcePanelId !== panel.id) {
      executeCommand("update-drag-operation", { isCopy: e.ctrlKey });
    }
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOverCounter((prev) => prev + 1);

    if (dragState.isDragging && dragState.sourcePanelId !== panel.id) {
      setIsDragOver(true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOverCounter((prev) => prev - 1);

    if (dragOverCounter <= 1) {
      setIsDragOver(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    setDragOverCounter(0);

    if (!dragState.isDragging || dragState.sourcePanelId === panel.id) {
      return;
    }

    await executeCommand("handle-drop", { dragState });
  };

  const getContextMenuItems = (
    selectedFiles: FileInfo[]
  ): ContextMenuItem[] => {
    const items: ContextMenuItem[] = [];
    const hasSelection = selectedFiles.length > 0;
    const isSingleFile = selectedFiles.length === 1;

    if (hasSelection) {
      items.push({
        id: "rename",
        label: "Rename",
        icon: "✏️",
        shortcut: "F2",
        disabled: !isSingleFile,
        onClick: () => {
          if (isSingleFile) {
            setPromptDialog({
              isOpen: true,
              title: "Rename File",
              message: "Enter new name:",
              defaultValue: selectedFiles[0].name,
              onConfirm: (newName: string) => {
                if (newName && newName !== selectedFiles[0].name) {
                  executeCommand("rename-file", { newName });
                }
                setPromptDialog({ ...promptDialog, isOpen: false });
              },
            });
          }
        },
      });

      items.push({ separator: true } as ContextMenuItem);

      // Clipboard operations
      items.push({
        id: "copy",
        label: "Copy",
        icon: "📋",
        shortcut: "Ctrl+C",
        onClick: () => {
          executeCommand("copy-files");
        },
      });

      items.push({
        id: "cut",
        label: "Cut",
        icon: "✂️",
        shortcut: "Ctrl+X",
        onClick: () => {
          executeCommand("cut-files");
        },
      });

      items.push({ separator: true } as ContextMenuItem);

      items.push({
        id: "delete",
        label:
          selectedFiles.length === 1
            ? "Delete"
            : `Delete ${selectedFiles.length} items`,
        icon: "🗑️",
        shortcut: "Del",
        onClick: () => {
          const message =
            selectedFiles.length === 1
              ? `Are you sure you want to delete "${selectedFiles[0].name}"?`
              : `Are you sure you want to delete ${selectedFiles.length} selected items?`;

          showConfirmDialog(
            "Confirm Delete",
            message,
            () => executeCommand("delete-files"),
            "danger"
          );
        },
      });

      items.push({ separator: true } as ContextMenuItem);
    }

    // Add paste option if clipboard has files
    if (clipboardState.hasFiles) {
      items.push({
        id: "paste",
        label: `Paste ${clipboardState.files.length} item${clipboardState.files.length > 1 ? "s" : ""}`,
        icon: "📋",
        shortcut: "Ctrl+V",
        disabled:
          clipboardState.operation === "cut" &&
          clipboardState.sourcePanelId === panel.id,
        onClick: () => {
          executeCommand("paste-files");
        },
      });

      items.push({ separator: true } as ContextMenuItem);
    }

    // Add "New" options
    items.push({
      id: "new-file",
      label: "New File",
      icon: "📄",
      onClick: () => {
        setPromptDialog({
          isOpen: true,
          title: "Create File",
          message: "Enter file name:",
          placeholder: "filename.txt",
          onConfirm: (name: string) => {
            if (name) {
              executeCommand("create-file", { navigateToTarget: false });
            }
            setPromptDialog({ ...promptDialog, isOpen: false });
          },
        });
      },
    });

    items.push({
      id: "new-folder",
      label: "New Folder",
      icon: "📁",
      onClick: () => {
        setPromptDialog({
          isOpen: true,
          title: "Create Folder",
          message: "Enter folder name:",
          placeholder: "New Folder",
          onConfirm: (name: string) => {
            if (name) {
              executeCommand("create-folder", { navigateToTarget: false });
            }
            setPromptDialog({ ...promptDialog, isOpen: false });
          },
        });
      },
    });

    return items;
  };


  const isFileCut = (file: FileInfo): boolean => {
    return (
      clipboardState.operation === "cut" &&
      clipboardState.sourcePanelId === panel.id &&
      clipboardState.files.some((clipFile) => clipFile.path === file.path)
    );
  };

  const sortedFiles = React.useMemo(() => {
    const files = [...panel.files];
    return files.sort((a, b) => {
      let comparison = 0;

      switch (panel.sortBy) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "size":
          comparison = a.size - b.size;
          break;
        case "modified":
          comparison =
            new Date(a.modified).getTime() - new Date(b.modified).getTime();
          break;
        case "type":
          comparison = (
            a.file_type === "Directory" ? "folder" : a.extension || ""
          ).localeCompare(
            b.file_type === "Directory" ? "folder" : b.extension || ""
          );
          break;
      }

      return panel.sortOrder === "asc" ? comparison : -comparison;
    });
  }, [panel.files, panel.sortBy, panel.sortOrder]);

  if (panel.isLoading) {
    return (
      <div className="file-panel loading">
        <div className="panel-header">
          <AddressBar
            currentPath={panel.currentPath}
            isActive={addressBarActive}
            onNavigate={handleAddressBarNavigate}
            onError={handleAddressBarError}
            onFocus={handleAddressBarFocus}
            className="loading-state"
          />
        </div>
        <div className="loading-content" data-testid="loading-indicator">Loading...</div>
      </div>
    );
  }

  if (panel.error) {
    return (
      <div className="file-panel error">
        <div className="panel-header">
          <AddressBar
            currentPath={panel.currentPath}
            isActive={addressBarActive}
            onNavigate={handleAddressBarNavigate}
            onError={handleAddressBarError}
            onFocus={handleAddressBarFocus}
          />
        </div>
        <div className="error-content">
          <div className="error-message">{panel.error}</div>
          <button onClick={() => executeCommand("load-directory", { panelId: panel.id, path: panel.currentPath })}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  const isDragTarget =
    isDragOver && dragState.isDragging && dragState.sourcePanelId !== panel.id;
  const panelClassName = `file-panel ${isDragTarget ? "drag-target" : ""} ${
    isDragTarget
      ? dragState.dragOperation === "copy"
        ? "copy-mode"
        : "move-mode"
      : ""
  }`;

  return (
    <div
      className={panelClassName}
      onDragOver={handleDragOver}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <div className="panel-header">
        <button
          className="back-button"
          onClick={handleBackClick}
          disabled={panel.currentPath === "/"}
        >
          ←
        </button>
        <AddressBar
          currentPath={panel.currentPath}
          isActive={addressBarActive}
          onNavigate={handleAddressBarNavigate}
          onError={handleAddressBarError}
          onFocus={handleAddressBarFocus}
        />
      </div>

      <NotificationContainer
        panelId={panel.id}
        className="panel-notifications"
      />

      <div className="file-list">
        {sortedFiles.map((file) => (
          <div
            key={file.name}
            className={`file-item ${panel.selectedFiles.includes(file.name) ? "selected" : ""} ${
              dragState.isDragging && dragState.draggedFiles.includes(file.name)
                ? "dragging"
                : ""
            } ${isFileCut(file) ? "cut" : ""}`}
            draggable
            onClick={(e) => handleFileClick(file, e)}
            onDoubleClick={() => handleFileDoubleClick(file)}
            onContextMenu={(e) => handleRightClick(e, file)}
            onDragStart={(e) => handleDragStart(e, file)}
            onDragEnd={handleDragEnd}
          >
            <FileIcon 
              fileName={file.name} 
              fileType={file.file_type} 
              size="medium"
            />
            <span className="file-name">{file.name}</span>
            <span className="file-size">
              {file.file_type === "Directory" ? "" : formatFileSize(file.size)}
            </span>
            <span className="file-date">{formatFileDate(file.modified)}</span>
          </div>
        ))}
      </div>

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={getContextMenuItems(contextMenu.selectedFiles)}
          onClose={() => setContextMenu(null)}
          selectedFiles={contextMenu.selectedFiles}
        />
      )}

      <ConfirmDialog
        isOpen={confirmDialog.isOpen}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant={confirmDialog.variant}
        onConfirm={() => {
          confirmDialog.onConfirm();
          setConfirmDialog({ ...confirmDialog, isOpen: false });
        }}
        onCancel={() => setConfirmDialog({ ...confirmDialog, isOpen: false })}
      />

      <PromptDialog
        isOpen={promptDialog.isOpen}
        title={promptDialog.title}
        message={promptDialog.message}
        defaultValue={promptDialog.defaultValue}
        placeholder={promptDialog.placeholder}
        onConfirm={promptDialog.onConfirm}
        onCancel={() => setPromptDialog({ ...promptDialog, isOpen: false })}
      />

      {archiveBrowser.isOpen && (
        <div className="archive-browser-overlay">
          <ArchiveBrowser
            archivePath={archiveBrowser.archivePath}
            onClose={handleArchiveClose}
            onExtract={handleArchiveExtract}
            onPreview={handleArchivePreview}
          />
        </div>
      )}
    </div>
  );
};

export default FilePanel;
