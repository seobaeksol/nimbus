import { NavigationCommand } from '../../base/NavigationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class GoToDocumentsCommand extends NavigationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'go-to-documents',
      label: 'Go to Documents',
      category: 'Navigation',
      description: 'Navigate to the Documents folder',
      icon: 'folder',
      shortcut: 'Alt+D'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.navigateUsingMethod(
      context,
      CommandExecutor.goToDocuments,
      'Go to Documents'
    );
  }
}