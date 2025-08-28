import { BaseCommand } from '../../base/BaseCommand';
import { CommandMetadata, ExecutionContext, GridLayoutConfig } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class SetGridLayoutCommand extends BaseCommand {
  constructor(
    private layout: GridLayoutConfig,
    executor: CommandExecutor,
    dialogService: DialogService
  ) {
    const metadata: CommandMetadata = {
      id: `set-layout-${layout.rows}x${layout.cols}`,
      label: layout.name,
      category: 'Panel',
      description: `Switch to ${layout.name} layout`,
      icon: 'layout-grid',
      shortcut: layout.rows === 1 && layout.cols === 1 ? 'Alt+1' :
                layout.rows === 1 && layout.cols === 2 ? 'Alt+2' :
                layout.rows === 2 && layout.cols === 2 ? 'Alt+3' : undefined
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        CommandExecutor.changeGridLayout(this.layout.rows, this.layout.cols, this.layout.name);
        this.showSuccess(`Switched to ${this.layout.name} layout`);
      },
      `Failed to set ${this.layout.name} layout`
    );
  }
}