import { NavigationCommand } from '../../base/NavigationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class GoToDownloadsCommand extends NavigationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'go-to-downloads',
      label: 'Go to Downloads',
      category: 'Navigation',
      description: 'Navigate to the Downloads folder',
      icon: 'download',
      shortcut: 'Alt+Shift+L'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.navigateUsingMethod(
      context,
      CommandExecutor.goToDownloads,
      'Go to Downloads'
    );
  }
}