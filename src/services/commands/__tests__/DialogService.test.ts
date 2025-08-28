import { BrowserDialogService, MockDialogService } from '../services/DialogService';

describe('DialogService', () => {
  describe('MockDialogService', () => {
    let mockDialogService: MockDialogService;

    beforeEach(() => {
      mockDialogService = new MockDialogService();
    });

    it('should queue and return prompt responses', async () => {
      mockDialogService.queuePromptResponse('test-response');
      mockDialogService.queuePromptResponse(null);

      const result1 = await mockDialogService.prompt('Test prompt 1');
      const result2 = await mockDialogService.prompt('Test prompt 2');

      expect(result1).toBe('test-response');
      expect(result2).toBe(null);
    });

    it('should queue and return confirm responses', async () => {
      mockDialogService.queueConfirmResponse(true);
      mockDialogService.queueConfirmResponse(false);

      const result1 = await mockDialogService.confirm('Confirm 1?');
      const result2 = await mockDialogService.confirm('Confirm 2?');

      expect(result1).toBe(true);
      expect(result2).toBe(false);
    });

    it('should track notifications', () => {
      mockDialogService.showNotification('Test message', 'success');
      mockDialogService.showNotification('Error message', 'error');

      expect(mockDialogService.getNotificationCount()).toBe(2);
      expect(mockDialogService.getLastNotification()).toEqual({
        message: 'Error message',
        type: 'error'
      });
      expect(mockDialogService.hasNotificationType('success')).toBe(true);
      expect(mockDialogService.hasNotificationType('warning')).toBe(false);
    });

    it('should clear all responses and notifications', () => {
      mockDialogService.queuePromptResponse('test');
      mockDialogService.queueConfirmResponse(true);
      mockDialogService.showNotification('test', 'info');

      mockDialogService.clearResponses();

      expect(mockDialogService.getNotificationCount()).toBe(0);
      expect(mockDialogService.promptResponses).toHaveLength(0);
      expect(mockDialogService.confirmResponses).toHaveLength(0);
    });

    it('should return default values when no responses queued', async () => {
      const promptResult = await mockDialogService.prompt('Test', 'default');
      const confirmResult = await mockDialogService.confirm('Test');

      expect(promptResult).toBe('default');
      expect(confirmResult).toBe(false);
    });
  });

  describe('BrowserDialogService', () => {
    let browserDialogService: BrowserDialogService;
    let mockDispatch: jest.Mock;

    beforeEach(() => {
      mockDispatch = jest.fn();
      browserDialogService = new BrowserDialogService(mockDispatch);

      // Mock browser dialog functions
      global.prompt = jest.fn();
      global.confirm = jest.fn();
    });

    afterEach(() => {
      delete (global as any).prompt;
      delete (global as any).confirm;
    });

    it('should use browser prompt', async () => {
      (global.prompt as jest.Mock).mockReturnValue('user-input');

      const result = await browserDialogService.prompt('Enter value:', 'default');

      expect(global.prompt).toHaveBeenCalledWith('Enter value:', 'default');
      expect(result).toBe('user-input');
    });

    it('should use browser confirm', async () => {
      (global.confirm as jest.Mock).mockReturnValue(true);

      const result = await browserDialogService.confirm('Are you sure?');

      expect(global.confirm).toHaveBeenCalledWith('Are you sure?');
      expect(result).toBe(true);
    });

    it('should dispatch notifications to store', () => {
      browserDialogService.showNotification('Test notification', 'success');

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          type: expect.stringContaining('addNotification'),
          payload: expect.objectContaining({
            message: 'Test notification',
            type: 'success',
            autoClose: true,
            duration: 3000
          })
        })
      );
    });

    it('should configure notification duration by type', () => {
      // Test different notification types
      browserDialogService.showNotification('Success', 'success');
      browserDialogService.showNotification('Info', 'info');
      browserDialogService.showNotification('Warning', 'warning');
      browserDialogService.showNotification('Error', 'error');

      const calls = mockDispatch.mock.calls;
      
      expect(calls[0][0].payload.duration).toBe(3000); // success
      expect(calls[1][0].payload.duration).toBe(4000); // info
      expect(calls[2][0].payload.duration).toBe(5000); // warning
      expect(calls[3][0].payload.duration).toBe(0); // error (manual close)
      expect(calls[3][0].payload.autoClose).toBe(false); // error doesn't auto-close
    });
  });
});