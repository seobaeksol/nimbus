import { BaseCommand } from '../../base/BaseCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class SwitchPanelCommand extends BaseCommand {
  constructor(
    private panelId: string,
    executor: CommandExecutor,
    dialogService: DialogService
  ) {
    const panelNumber = panelId.replace('panel-', '');
    const metadata: CommandMetadata = {
      id: `switch-to-${panelId}`,
      label: `Switch to Panel ${panelNumber}`,
      category: 'Panel',
      description: `Switch focus to panel ${panelNumber}`,
      icon: 'layout-panel',
      shortcut: `Ctrl+${panelNumber}`
    };
    
    super(metadata, executor, dialogService);
  }

  canExecute(context: ExecutionContext): boolean {
    if (!super.canExecute(context)) return false;
    
    // Only allow switching if panel exists and is different from current
    const panel = context.panels[this.panelId];
    return panel && this.panelId !== context.panelId;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        const panel = context.panels[this.panelId];
        if (!panel) {
          this.showWarning(`Panel ${this.panelId} does not exist`);
          return;
        }

        if (this.panelId === context.panelId) {
          this.showInfo(`Already on panel ${this.panelId}`);
          return;
        }

        CommandExecutor.switchToPanel(this.panelId);
        const panelNumber = this.panelId.replace('panel-', '');
        this.showSuccess(`Switched to panel ${panelNumber}`);
      },
      `Failed to switch to panel ${this.panelId}`
    );
  }
}