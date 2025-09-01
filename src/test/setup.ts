import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';

// Extend Vitest's expect with testing-library matchers
expect.extend(matchers);

// Cleanup after each test case (e.g. clearing jsdom)
afterEach(() => {
  cleanup();
});

// Mock Tauri API for tests
const mockInvoke = vi.fn().mockImplementation((command: string, args?: any) => {
  // Mock different commands with appropriate responses
  switch (command) {
    case 'list_dir':
      return Promise.resolve([
        {
          name: 'test-file.txt',
          path: '/test/path/test-file.txt',
          size: 1024,
          sizeFormatted: '1.0 KB',
          modified: '2024-01-01T12:00:00Z',
          fileType: 'file',
          extension: 'txt',
          permissions: { read: true, write: true, execute: false },
          isHidden: false,
          isReadonly: false,
        }
      ]);
    case 'get_system_paths':
      return Promise.resolve({
        home: '/home/testuser',
        documents: '/home/testuser/Documents',
        downloads: '/home/testuser/Downloads',
      });
    case 'resolve_path':
      return Promise.resolve(args?.path || '/resolved/path');
    default:
      return Promise.resolve();
  }
});

const mockListen = vi.fn().mockResolvedValue(() => {});

// Global mocks
Object.defineProperty(window, '__TAURI_INTERNALS__', {
  value: {
    invoke: mockInvoke,
    listen: mockListen,
  },
});

// Mock @tauri-apps/api modules
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: mockInvoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen,
  emit: vi.fn(),
}));

// Export mocks for use in tests
export { mockInvoke, mockListen };

// Add custom matchers types
declare module 'vitest' {
  interface Assertion<T = any>
    extends jest.Matchers<void, T>, 
           ReturnType<typeof matchers> {}
}