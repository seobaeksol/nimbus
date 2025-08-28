import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class CreateFolderCommand extends FileOperationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'create-folder',
      label: 'New Folder',
      category: 'File',
      description: 'Create a new folder in the current directory',
      icon: 'folder-plus',
      shortcut: 'Ctrl+Shift+N'
    };
    
    super(metadata, executor, dialogService);
  }

  canExecute(context: ExecutionContext): boolean {
    return super.canExecute(context) && !!context.currentPath;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        this.validatePanel(context);

        const folderName = await this.dialogService.prompt(
          'Enter folder name:',
          'New Folder'
        );

        if (!folderName) {
          this.showInfo('Folder creation cancelled');
          return;
        }

        await CommandExecutor.createFolder(context.panelId, folderName);
        this.showSuccess(`Created folder: ${folderName}`);
      },
      'Failed to create folder'
    );
  }
}