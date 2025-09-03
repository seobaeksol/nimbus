import { Command, CommandMetadata, ExecutionContext } from "../../types";
import { AppDispatch } from "@/store";
import { showSearchPanel } from "@/store/slices/panelSlice";

export class OpenSearchPanelCommand implements Command<{}> {
  public readonly metadata: CommandMetadata = {
    id: "search.open-panel",
    label: "Open Search Panel",
    category: "Search",
    description: "Open the file search panel",
    shortcut: "Ctrl+F"
  };

  constructor(private dispatch: AppDispatch) {}

  canExecute(_context: ExecutionContext): boolean {
    return true;
  }

  async execute(_context: ExecutionContext): Promise<void> {
    this.dispatch(showSearchPanel());
  }
}