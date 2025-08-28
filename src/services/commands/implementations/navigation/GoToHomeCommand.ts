import { NavigationCommand } from '../../base/NavigationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class GoToHomeCommand extends NavigationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'go-to-home',
      label: 'Go to Home',
      category: 'Navigation',
      description: 'Navigate to the home directory',
      icon: 'home',
      shortcut: 'Alt+Home'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.navigateUsingMethod(
      context,
      CommandExecutor.goToHome,
      'Go to Home'
    );
  }
}