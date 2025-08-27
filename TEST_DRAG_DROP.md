# Drag and Drop Testing Guide

## Overview
This guide covers comprehensive testing of the drag and drop functionality across all panel layouts in Nimbus File Manager.

## Test Environment Setup

1. **Start Development Server**
   ```bash
   npm run tauri dev
   ```

2. **Create Test Files/Folders**
   Create a test directory structure for testing:
   ```
   test_directory/
   ├── test_file.txt
   ├── test_image.png
   ├── test_document.pdf
   ├── subfolder1/
   │   ├── nested_file.txt
   │   └── deep_folder/
   │       └── deep_file.txt
   ├── subfolder2/
   └── empty_folder/
   ```

## Test Cases by Layout

### 1x1 Layout (Single Panel) - Ctrl+G+1
**Expected Behavior**: No drag and drop operations (source = destination)
- ✅ **Test**: Try dragging files within same panel
- ✅ **Expected**: Should not trigger transfer (same panel check)
- ✅ **Validation**: No file operations should occur

### 1x2 Layout (Dual Panel) - Ctrl+G+2  
**Core drag and drop functionality**

#### Test Case 1: File Transfer (Move)
1. Navigate Panel A to source directory with test files
2. Navigate Panel B to different destination directory
3. Select a single file in Panel A
4. Drag file from Panel A to Panel B (without Ctrl)
5. ✅ **Expected**: File moves from A to B
6. ✅ **Validation**: File disappears from Panel A, appears in Panel B

#### Test Case 2: File Transfer (Copy)
1. Select a file in Panel A
2. Hold Ctrl key and drag to Panel B
3. ✅ **Expected**: File copies from A to B
4. ✅ **Validation**: File remains in Panel A, copy appears in Panel B

#### Test Case 3: Directory Transfer (Move)
1. Drag a folder with subfolders from Panel A to Panel B
2. ✅ **Expected**: Entire directory structure moves
3. ✅ **Validation**: All subdirectories and files preserved in destination

#### Test Case 4: Directory Transfer (Copy)
1. Hold Ctrl and drag a folder from Panel A to Panel B
2. ✅ **Expected**: Entire directory structure copied recursively
3. ✅ **Validation**: Original and copy both exist with identical structure

#### Test Case 5: Multiple File Selection
1. Select multiple files (Ctrl+click)
2. Drag selection from Panel A to Panel B
3. ✅ **Expected**: All selected files transfer together
4. ✅ **Validation**: All files appear in destination

### 2x2 Layout (Quad Panel) - Ctrl+G+3
**Cross-panel transfers in 4-panel grid**

#### Test Case 6: Adjacent Panel Transfer
1. Drag files between horizontally adjacent panels (top-left → top-right)
2. ✅ **Expected**: Normal transfer behavior
3. ✅ **Validation**: Files transfer correctly

#### Test Case 7: Diagonal Panel Transfer  
1. Drag files diagonally (top-left → bottom-right)
2. ✅ **Expected**: Normal transfer behavior
3. ✅ **Validation**: Files transfer correctly

#### Test Case 8: Vertical Panel Transfer
1. Drag files vertically (top-left → bottom-left)
2. ✅ **Expected**: Normal transfer behavior
3. ✅ **Validation**: Files transfer correctly

### 2x3 Layout (Six Panel) - Ctrl+G+4
**Multi-panel coordination**

#### Test Case 9: Cross-Row Transfer
1. Drag files from top row to bottom row
2. ✅ **Expected**: Normal transfer behavior
3. ✅ **Validation**: Files transfer correctly

#### Test Case 10: Within-Row Transfer
1. Drag files between panels in same row
2. ✅ **Expected**: Normal transfer behavior
3. ✅ **Validation**: Files transfer correctly

### 3x2 Layout (Vertical Layout) - Ctrl+G+5  
**Vertical panel arrangement**

#### Test Case 11: Top-to-Bottom Transfer
1. Drag files from top panel to bottom panel
2. ✅ **Expected**: Normal transfer behavior
3. ✅ **Validation**: Files transfer correctly

## Visual Feedback Tests

### Drag State Visual Indicators
- ✅ **Dragging State**: File items show opacity/rotation effects during drag
- ✅ **Drop Zone**: Target panel highlights when dragging over it  
- ✅ **Operation Indicator**: Shows "Move files here" vs "Copy files here (Ctrl held)"
- ✅ **Drag Ghost**: Shows number of files being dragged

### Operation Type Visual Feedback
- ✅ **Move Operation**: Default cursor and "Move files here" message
- ✅ **Copy Operation**: Ctrl+drag shows "Copy files here (Ctrl held)" message
- ✅ **Dynamic Switching**: Message updates when Ctrl key pressed/released during drag

## Error Handling Tests

### Test Case 12: Permission Denied
1. Create read-only files/folders
2. Try to move/copy to restricted location
3. ✅ **Expected**: Error message displayed in panel
4. ✅ **Validation**: Operation fails gracefully with specific error

### Test Case 13: Destination Already Exists
1. Try to copy file to location where file with same name exists
2. ✅ **Expected**: "Destination already exists" error
3. ✅ **Validation**: Original files unchanged

### Test Case 14: Source Not Found
1. Delete a file after drag starts but before drop
2. ✅ **Expected**: "Source file not found" error  
3. ✅ **Validation**: Operation fails gracefully

### Test Case 15: Insufficient Disk Space
1. Try copying large file to location with insufficient space
2. ✅ **Expected**: Disk space error message
3. ✅ **Validation**: Partial files cleaned up

## Performance Tests

### Test Case 16: Large File Transfer
1. Create/copy a large file (>100MB)
2. Drag to another panel
3. ✅ **Expected**: Transfer completes without UI freeze
4. ✅ **Validation**: Progress indication or completion notification

### Test Case 17: Many Small Files
1. Create directory with many small files (>1000)
2. Copy directory to another panel  
3. ✅ **Expected**: All files transfer successfully
4. ✅ **Validation**: File count matches in destination

### Test Case 18: Deep Directory Structure
1. Create deeply nested directory (>10 levels)
2. Copy to another panel
3. ✅ **Expected**: Full structure preserved
4. ✅ **Validation**: All nested levels copied correctly

## Edge Cases

### Test Case 19: Special Characters in Names
1. Create files with unicode, spaces, special characters
2. Transfer via drag and drop
3. ✅ **Expected**: Names preserved correctly
4. ✅ **Validation**: No character corruption

### Test Case 20: Root Directory Operations
1. Test drag and drop from/to root directory "/"
2. ✅ **Expected**: Path construction handles root correctly
3. ✅ **Validation**: No double slashes or path errors

### Test Case 21: Symlink Handling
1. Create symbolic links
2. Try to drag and drop symlinks
3. ✅ **Expected**: Appropriate handling (copy target or skip)
4. ✅ **Validation**: No broken links or crashes

## Test Execution Checklist

### Before Testing
- [ ] Build project successfully (`npm run build`)
- [ ] Start development server (`npm run tauri dev`)
- [ ] Create test directory structure
- [ ] Verify all layouts accessible (Ctrl+G+1-5)

### During Testing  
- [ ] Test each layout systematically
- [ ] Verify visual feedback for all operations
- [ ] Check error messages are user-friendly
- [ ] Validate file integrity after transfers
- [ ] Test both move and copy operations

### After Testing
- [ ] Clean up test files
- [ ] Document any issues found
- [ ] Verify no memory leaks or performance degradation
- [ ] Confirm all panels refresh correctly after operations

## Known Limitations
1. Symlink copying not fully implemented (currently skipped)
2. No progress indicators for large operations (future enhancement)
3. No undo functionality for drag and drop operations

## Success Criteria
- ✅ All test cases pass without crashes
- ✅ File integrity maintained during transfers  
- ✅ UI remains responsive during operations
- ✅ Error messages are clear and actionable
- ✅ Visual feedback is consistent across all layouts
- ✅ Performance acceptable for typical file operations