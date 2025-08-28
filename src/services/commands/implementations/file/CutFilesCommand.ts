import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class CutFilesCommand extends FileOperationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'cut-files',
      label: 'Cut',
      category: 'File',
      description: 'Cut selected files to clipboard',
      icon: 'scissors',
      shortcut: 'Ctrl+X'
    };
    
    super(metadata, executor, dialogService);
  }

  protected getRequiredSelectionCount(): number {
    return 1;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        this.validatePanel(context);

        const selectedFiles = this.getSelectedFiles(context);
        if (selectedFiles.length === 0) {
          this.showWarning('No files selected for cutting');
          return;
        }

        CommandExecutor.cutFiles(context.panelId, selectedFiles);

        const fileWord = selectedFiles.length === 1 ? 'item' : 'items';
        this.showSuccess(`Cut ${selectedFiles.length} ${fileWord} to clipboard`);
      },
      'Failed to cut files'
    );
  }
}