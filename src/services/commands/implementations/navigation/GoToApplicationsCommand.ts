import { NavigationCommand } from '../../base/NavigationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class GoToApplicationsCommand extends NavigationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'go-to-applications',
      label: 'Go to Applications',
      category: 'Navigation',
      description: 'Navigate to the Applications folder',
      icon: 'app-window',
      shortcut: 'Alt+A'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.navigateUsingMethod(
      context,
      CommandExecutor.goToApplications,
      'Go to Applications'
    );
  }
}