import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';
import { FileInfo } from '../../../../types';

export class DeleteFilesCommand extends FileOperationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'delete-files',
      label: 'Delete',
      category: 'File',
      description: 'Delete selected files and folders',
      icon: 'trash',
      shortcut: 'Delete'
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
          this.showWarning('No files selected for deletion');
          return;
        }

        const confirmMessage = this.generateConfirmationMessage('delete', selectedFiles);
        const confirmed = await this.dialogService.confirm(confirmMessage);

        if (!confirmed) {
          this.showInfo('Delete operation cancelled');
          return;
        }

        await this.executeWithProgress(
          selectedFiles,
          async (file: FileInfo) => {
            await CommandExecutor.deleteFiles(context.panelId, [file]);
          },
          'Deleting'
        );

        const fileWord = selectedFiles.length === 1 ? 'item' : 'items';
        this.showSuccess(`Deleted ${selectedFiles.length} ${fileWord}`);
      },
      'Failed to delete files'
    );
  }
}