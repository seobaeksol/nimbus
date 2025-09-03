import CommandPalette from "@/components/common/CommandPalette";
import PromptDialog from "@/components/common/PromptDialog";
import { useCommands } from "@/hooks/useCommands";
import { useAppDispatch, useAppSelector } from "@/store";
import { setAddressBarActive, toggleSearchPanel, showSearchPanel } from "@/store/slices/panelSlice";
import { useEffect, useState } from "react";

interface CommandProviderProps {
  children: React.ReactNode;
}

export const CommandProvider: React.FC<CommandProviderProps> = ({
  children,
}) => {
  const [promptDialog, setPromptDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    placeholder?: string;
    onConfirm: (value: string) => void;
  }>({
    isOpen: false,
    title: "",
    message: "",
    onConfirm: () => {},
  });
  const { executeCommand } = useCommands();
  const { panels, activePanelId, searchPanelVisible } = useAppSelector(
    (state) => state.panels
  );
  const dispatch = useAppDispatch();
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  // Global keyboard shortcut handler - only affects the active panel
  useEffect(() => {
    const handleGlobalKeyDown = (event: KeyboardEvent) => {
      // Handle Ctrl+Shift+P for Command Palette (works globally)
      if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "P"
      ) {
        event.preventDefault();
        setCommandPaletteOpen(true);
        return;
      }

      // Only handle other shortcuts if we have an active panel
      if (!activePanelId || !panels[activePanelId]) return;

      // Handle Ctrl+L for address bar focus
      if ((event.ctrlKey || event.metaKey) && event.key === "l") {
        event.preventDefault();
        dispatch(setAddressBarActive(activePanelId));
        return;
      }

      // Handle Ctrl+N for new folder
      if ((event.ctrlKey || event.metaKey) && event.key === "n") {
        event.preventDefault();
        setPromptDialog({
          isOpen: true,
          title: "Create Folder",
          message:
            "Enter folder path (relative to current directory or absolute):",
          placeholder: "foldername or /path/to/folder or subdir/folder",
          onConfirm: (path: string) => {
            if (path) {
              executeCommand("create-folder", { navigateToTarget: true });
            }
            setPromptDialog({ ...promptDialog, isOpen: false });
          },
        });
        return;
      }

      // Handle Ctrl+T for new file
      if ((event.ctrlKey || event.metaKey) && event.key === "t") {
        event.preventDefault();
        setPromptDialog({
          isOpen: true,
          title: "Create File",
          message:
            "Enter file path (relative to current directory or absolute):",
          placeholder: "filename.txt or /path/to/file.txt or subdir/file.txt",
          onConfirm: (path: string) => {
            if (path) {
              executeCommand("create-file", { navigateToTarget: true });
            }
            setPromptDialog({ ...promptDialog, isOpen: false });
          },
        });
        return;
      }

      // Handle Ctrl+F for search panel
      if ((event.ctrlKey || event.metaKey) && event.key === "f") {
        event.preventDefault();
        dispatch(showSearchPanel());
        return;
      }

      // Handle Ctrl+Shift+F for global search (same as Ctrl+F for now)
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "F") {
        event.preventDefault();
        dispatch(showSearchPanel());
        return;
      }

      // Handle Escape to close search panel (global)
      if (event.key === "Escape" && searchPanelVisible) {
        event.preventDefault();
        dispatch(toggleSearchPanel());
        return;
      }

      // Future shortcuts can be added here
      // Examples: Ctrl+W (close tab), F3 (view), F4 (edit), etc.
    };

    document.addEventListener("keydown", handleGlobalKeyDown);
    return () => document.removeEventListener("keydown", handleGlobalKeyDown);
  }, [activePanelId, panels, searchPanelVisible, dispatch, executeCommand, promptDialog]);

  return (
    <>
      {children}
      <PromptDialog
        isOpen={promptDialog.isOpen}
        title={promptDialog.title}
        message={promptDialog.message}
        placeholder={promptDialog.placeholder}
        onConfirm={promptDialog.onConfirm}
        onCancel={() => setPromptDialog({ ...promptDialog, isOpen: false })}
      />

      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        dispatch={dispatch}
      />
    </>
  );
};
