import { FileOperationCommand } from '../../base/FileOperationCommand';
import { CommandMetadata, ExecutionContext } from '../../types';
import { CommandExecutor } from '../../../commandExecutor';
import { DialogService } from '../../services/DialogService';

export class RenameFileCommand extends FileOperationCommand {
  constructor(executor: CommandExecutor, dialogService: DialogService) {
    const metadata: CommandMetadata = {
      id: 'rename-file',
      label: 'Rename',
      category: 'File',
      description: 'Rename the selected file or folder',
      icon: 'edit',
      shortcut: 'F2'
    };
    
    super(metadata, executor, dialogService);
  }

  protected getRequiredSelectionCount(): number {
    return 1;
  }

  canExecute(context: ExecutionContext): boolean {
    return super.canExecute(context) && context.selectedFiles.length === 1;
  }

  async execute(context: ExecutionContext): Promise<void> {
    await this.withErrorHandling(
      async () => {
        this.validatePanel(context);

        const selectedFiles = this.getSelectedFiles(context);
        if (selectedFiles.length !== 1) {
          this.showWarning('Please select exactly one file to rename');
          return;
        }

        const file = selectedFiles[0];
        const newName = await this.dialogService.prompt(
          'Enter new name:',
          file.name
        );

        if (!newName || newName === file.name) {
          this.showInfo('Rename operation cancelled');
          return;
        }

        await CommandExecutor.renameFile(context.panelId, file, newName);
        this.showSuccess(`Renamed "${file.name}" to "${newName}"`);
      },
      'Failed to rename file'
    );
  }
}