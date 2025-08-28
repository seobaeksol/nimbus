import { NavigationCommand } from '../../base/NavigationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class GoToDesktopCommand extends NavigationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'go-to-desktop',
      label: 'Go to Desktop',
      category: 'Navigation',
      description: 'Navigate to the Desktop folder',
      icon: 'monitor',
      shortcut: 'Alt+Shift+D'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.navigateUsingMethod(
      context,
      CommandExecutor.goToDesktop,
      'Go to Desktop'
    );
  }
}