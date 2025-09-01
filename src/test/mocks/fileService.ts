import { FileInfo } from '@/types';

// Mock file data for testing
export const mockFileInfo: FileInfo = {
  name: 'test-file.txt',
  path: '/test/path/test-file.txt',
  size: 1024,
  sizeFormatted: '1.0 KB',
  modified: '2024-01-01T12:00:00Z',
  created: '2024-01-01T10:00:00Z',
  accessed: '2024-01-01T14:00:00Z',
  fileType: 'file' as const,
  extension: 'txt',
  permissions: {
    read: true,
    write: true,
    execute: false,
    ownerRead: true,
    ownerWrite: true,
    ownerExecute: false,
    groupRead: true,
    groupWrite: false,
    groupExecute: false,
    otherRead: true,
    otherWrite: false,
    otherExecute: false,
  },
  isHidden: false,
  isReadonly: false,
  owner: 'testuser',
  group: 'testgroup',
};

export const mockDirectoryInfo: FileInfo = {
  ...mockFileInfo,
  name: 'test-directory',
  path: '/test/path/test-directory',
  fileType: 'directory' as const,
  extension: undefined,
  size: 0,
  sizeFormatted: '0 B',
};

export const mockFileList: FileInfo[] = [
  mockDirectoryInfo,
  mockFileInfo,
  {
    ...mockFileInfo,
    name: 'document.pdf',
    path: '/test/path/document.pdf',
    extension: 'pdf',
    size: 204800,
    sizeFormatted: '200 KB',
  },
];

// Mock file service responses
export const createMockFileServiceResponse = {
  listDirectory: vi.fn().mockResolvedValue(mockFileList),
  getFileInfo: vi.fn().mockResolvedValue(mockFileInfo),
  createDirectory: vi.fn().mockResolvedValue(undefined),
  createFile: vi.fn().mockResolvedValue(undefined),
  copyFiles: vi.fn().mockResolvedValue('copy-operation-id'),
  moveFiles: vi.fn().mockResolvedValue('move-operation-id'),
  deleteFiles: vi.fn().mockResolvedValue('delete-operation-id'),
  renameFile: vi.fn().mockResolvedValue('/new/path/renamed-file.txt'),
  getSystemPaths: vi.fn().mockResolvedValue({
    home: '/home/testuser',
    documents: '/home/testuser/Documents',
    downloads: '/home/testuser/Downloads',
    desktop: '/home/testuser/Desktop',
  }),
  resolvePath: vi.fn().mockResolvedValue('/resolved/path'),
};

// Reset all mocks
export const resetFileServiceMocks = () => {
  Object.values(createMockFileServiceResponse).forEach(mock => {
    if (typeof mock === 'function' && 'mockReset' in mock) {
      mock.mockReset();
    }
  });
};