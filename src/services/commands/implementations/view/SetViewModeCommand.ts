import { BaseCommand } from '../../base/BaseCommand';
import { CommandMetadata, ExecutionContext, ViewMode } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class SetViewModeCommand extends BaseCommand {
  constructor(
    private viewMode: ViewMode,
    executor: CommandExecutor,
    dialogService: DialogService
  ) {
    const metadata: CommandMetadata = {
      id: `set-view-${viewMode}`,
      label: `${viewMode.charAt(0).toUpperCase() + viewMode.slice(1)} View`,
      category: 'View',
      description: `Switch to ${viewMode} view mode`,
      icon: viewMode === 'list' ? 'list' : viewMode === 'grid' ? 'grid' : 'table',
      shortcut: viewMode === 'list' ? 'Ctrl+1' : viewMode === 'grid' ? 'Ctrl+2' : 'Ctrl+3'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        CommandExecutor.changeViewMode(context.panelId, this.viewMode);
        this.showSuccess(`Switched to ${this.viewMode} view`);
      },
      `Failed to set ${this.viewMode} view`
    );
  }
}