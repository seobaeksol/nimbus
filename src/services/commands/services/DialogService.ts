import { AppDispatch } from "../../../store";
import { addNotification } from "../../../store/slices/panelSlice";
import { DialogService, NotificationType } from "../types";

// Re-export the DialogService interface for easier importing
export type { DialogService, NotificationType } from "../types";

/**
 * Browser-based implementation of DialogService
 * Uses native browser dialogs for user interaction
 */
export class BrowserDialogService implements DialogService {
  constructor(private dispatch: AppDispatch) {}

  async prompt(message: string, defaultValue = ""): Promise<string | null> {
    return window.prompt(message, defaultValue);
  }

  async confirm(message: string): Promise<boolean> {
    return window.confirm(message);
  }

  showNotification(message: string, type: NotificationType): void {
    this.dispatch(
      addNotification({
        id: `dialog-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        message,
        type,
        timestamp: Date.now(),
        autoClose: type !== "error", // Keep error notifications visible
        duration: this.getDurationForType(type),
      })
    );
  }

  private getDurationForType(type: NotificationType): number {
    switch (type) {
      case "success":
        return 3000;
      case "info":
        return 4000;
      case "warning":
        return 5000;
      case "error":
        return 0; // Manual close for errors
      default:
        return 4000;
    }
  }
}
