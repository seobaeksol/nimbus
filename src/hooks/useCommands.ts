import { useCallback } from "react";
import { useAppSelector, useAppDispatch } from "../store";
import { CommandService } from "../services/commands/services/CommandService";
import { ExecutionContext } from "../services/commands/types";

/**
 * Hook for accessing the modern command system
 * Replaces direct calls to static CommandExecutor methods
 */
export function useCommands() {
  const dispatch = useAppDispatch();
  const { panels, activePanelId, clipboardState } = useAppSelector(
    (state) => state.panels
  );

  // Initialize command service
  const commandService = CommandService.initialize(dispatch);

  // Create execution context from current state
  const createContext = useCallback(
    (panelId?: string): ExecutionContext => {
      const targetPanelId = panelId || activePanelId;
      const activePanel = targetPanelId ? panels[targetPanelId] : null;

      const selectedFiles =
        activePanel?.selectedFiles
          .map((fileName) =>
            activePanel.files.find((file) => file.name === fileName)
          )
          .filter(Boolean) || [];

      return {
        panelId: targetPanelId || "",
        currentPath: activePanel?.currentPath || "/",
        selectedFiles: selectedFiles as any[], // Type assertion for compatibility
        dispatch,
        clipboardHasFiles: clipboardState.hasFiles,
        panels,
        clipboardState: {
          hasFiles: clipboardState.hasFiles,
          files: clipboardState.files || [],
          operation: clipboardState.operation || null,
          sourcePanelId: clipboardState.sourcePanelId || null,
        },
      };
    },
    [activePanelId, panels, clipboardState, dispatch]
  );

  // Execute command by ID
  const executeCommand = useCallback(
    async (
      commandId: string,
      options?: Record<string, any>
    ): Promise<boolean> => {
      const context = createContext();
      return commandService.executeCommand(commandId, context, options);
    },
    [commandService, createContext]
  );

  // Get available commands
  const getAvailableCommands = useCallback(
    (panelId?: string) => {
      const context = createContext(panelId);
      return commandService.getAvailableCommands(context);
    },
    [commandService, createContext]
  );

  // Search commands
  const searchCommands = useCallback(
    (searchTerm: string, panelId?: string) => {
      const context = createContext(panelId);
      return commandService.searchCommands(searchTerm, context);
    },
    [commandService, createContext]
  );

  // Check if command can execute
  const canExecuteCommand = useCallback(
    (commandId: string, panelId?: string): boolean => {
      const context = createContext(panelId);
      return commandService.canExecuteCommand(commandId, context);
    },
    [commandService, createContext]
  );

  // Get command by ID
  const getCommand = useCallback(
    (commandId: string) => {
      return commandService.getCommand(commandId);
    },
    [commandService]
  );

  return {
    executeCommand,
    getAvailableCommands,
    searchCommands,
    canExecuteCommand,
    getCommand,
    createContext,
    commandService,
  };
}
