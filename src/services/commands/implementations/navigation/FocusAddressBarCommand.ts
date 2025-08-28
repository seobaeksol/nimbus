import { NavigationCommand } from '../../base/NavigationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class FocusAddressBarCommand extends NavigationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'focus-address-bar',
      label: 'Focus Address Bar',
      category: 'Navigation',
      description: 'Focus the address bar for direct path input',
      icon: 'navigation',
      shortcut: 'Ctrl+L'
    };
    
    super(metadata, executor, dialogService);
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        this.validateNavigationContext(context);
        
        CommandExecutor.focusAddressBar();
        this.showInfo('Address bar focused');
      },
      'Failed to focus address bar'
    );
  }
}