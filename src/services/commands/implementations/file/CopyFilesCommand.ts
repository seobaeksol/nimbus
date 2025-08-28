import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class CopyFilesCommand extends FileOperationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'copy-files',
      label: 'Copy',
      category: 'File',
      description: 'Copy selected files to clipboard',
      icon: 'copy',
      shortcut: 'Ctrl+C'
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
          this.showWarning('No files selected for copying');
          return;
        }

        CommandExecutor.copyFiles(context.panelId, selectedFiles);

        const fileWord = selectedFiles.length === 1 ? 'item' : 'items';
        this.showSuccess(`Copied ${selectedFiles.length} ${fileWord} to clipboard`);
      },
      'Failed to copy files'
    );
  }
}