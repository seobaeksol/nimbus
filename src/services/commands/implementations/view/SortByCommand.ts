import { BaseCommand } from '../../base/BaseCommand';
import { CommandMetadata, ExecutionContext, SortBy } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class SortByCommand extends BaseCommand {
  constructor(
    private sortBy: SortBy,
    executor: CommandExecutor,
    dialogService: DialogService
  ) {
    const metadata: CommandMetadata = {
      id: `sort-by-${sortBy}`,
      label: `Sort by ${sortBy.charAt(0).toUpperCase() + sortBy.slice(1)}`,
      category: 'View',
      description: `Sort files by ${sortBy}`,
      icon: 'arrow-up-down',
      shortcut: sortBy === 'name' ? 'Ctrl+Shift+1' : 
                sortBy === 'size' ? 'Ctrl+Shift+2' :
                sortBy === 'modified' ? 'Ctrl+Shift+3' : 'Ctrl+Shift+4'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        CommandExecutor.changeSorting(context.panelId, this.sortBy);
        this.showSuccess(`Sorted by ${this.sortBy}`);
      },
      `Failed to sort by ${this.sortBy}`
    );
  }
}