import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutorService } from '../../services/CommandExecutorService';
import { DialogService } from '../../services/DialogService';

export class CreateFileCommand extends FileOperationCommand {
  constructor(executor: CommandExecutorService, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'create-file',
      label: 'New File',
      category: 'File',
      description: 'Create a new file in the current directory',
      icon: 'file-plus',
      shortcut: 'Ctrl+N'
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

        const fileName = await this.dialogService.prompt(
          'Enter file name:',
          'new-file.txt'
        );

        if (!fileName) {
          this.showInfo('File creation cancelled');
          return;
        }

        await this.executor.createFile(context.panelId, fileName);
        this.showSuccess(`Created file: ${fileName}`);
      },
      'Failed to create file'
    );
  }
}