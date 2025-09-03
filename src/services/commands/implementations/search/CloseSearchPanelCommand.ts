import { Command, CommandMetadata, ExecutionContext } from "../../types";
import { AppDispatch } from "@/store";
import { hideSearchPanel } from "@/store/slices/panelSlice";

export class CloseSearchPanelCommand implements Command<{}> {
  public readonly metadata: CommandMetadata = {
    id: "search.close-panel",
    label: "Close Search Panel",
    category: "Search",
    description: "Close the file search panel",
    shortcut: "Escape"
  };

  constructor(private dispatch: AppDispatch) {}

  canExecute(_context: ExecutionContext): boolean {
    return true;
  }

  async execute(_context: ExecutionContext): Promise<void> {
    this.dispatch(hideSearchPanel());
  }
}