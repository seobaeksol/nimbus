import { Command, CommandMetadata, ExecutionContext } from "../../types";
import { AppDispatch } from "@/store";
import { toggleSearchPanel } from "@/store/slices/panelSlice";

export class ToggleSearchPanelCommand implements Command<{}> {
  public readonly metadata: CommandMetadata = {
    id: "search.toggle-panel",
    label: "Toggle Search Panel",
    category: "Search",
    description: "Show or hide the file search panel"
  };

  constructor(private dispatch: AppDispatch) {}

  canExecute(_context: ExecutionContext): boolean {
    return true;
  }

  async execute(_context: ExecutionContext): Promise<void> {
    this.dispatch(toggleSearchPanel());
  }
}