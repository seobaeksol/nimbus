import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class PasteFilesCommand extends FileOperationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'paste-files',
      label: 'Paste',
      category: 'File',
      description: 'Paste files from clipboard',
      icon: 'clipboard',
      shortcut: 'Ctrl+V'
    };
    
    super(metadata, executor, dialogService);
  }

  canExecute(context: ExecutionContext): boolean {
    return super.canExecute(context) && context.clipboardHasFiles;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        this.validatePanel(context);

        if (!context.clipboardHasFiles) {
          this.showWarning('No files in clipboard to paste');
          return;
        }

        await CommandExecutor.pasteFiles(context.panelId);
        this.showSuccess('Pasted files from clipboard');
      },
      'Failed to paste files'
    );
  }
}