//! Tests for core-engine crate

use super::*;
use local_fs::LocalFileSystem;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Create a temporary directory for testing
async fn create_temp_dir() -> Result<PathBuf, std::io::Error> {
    let temp_dir = std::env::temp_dir().join(format!("nimbus_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).await?;
    Ok(temp_dir)
}

/// Clean up test directory
async fn cleanup_temp_dir(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        fs::remove_dir_all(path).await?;
    }
    Ok(())
}

#[tokio::test]
async fn test_local_filesystem_creation() {
    let fs = LocalFileSystem::new();
    // Should create without error
    assert_eq!(std::mem::size_of_val(&fs), 0); // Zero-sized struct
}

#[tokio::test]
async fn test_file_info_creation() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let test_file = temp_dir.join("test_file.txt");
    
    // Create a test file
    fs::write(&test_file, b"Hello, World!").await.expect("Failed to create test file");
    
    let fs = LocalFileSystem::new();
    let file_info = fs.get_file_info(&test_file).await;
    
    cleanup_temp_dir(&temp_dir).await.ok();
    
    assert!(file_info.is_ok());
    let info = file_info.unwrap();
    assert_eq!(info.name, "test_file.txt");
    assert_eq!(info.file_type, FileType::File);
    assert_eq!(info.extension, Some("txt".to_string()));
    assert_eq!(info.size, 13); // "Hello, World!" is 13 bytes
}

#[tokio::test]
async fn test_directory_listing() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    
    // Create test files and directories
    let test_file = temp_dir.join("test_file.txt");
    let test_subdir = temp_dir.join("test_subdir");
    
    fs::write(&test_file, b"content").await.expect("Failed to create test file");
    fs::create_dir(&test_subdir).await.expect("Failed to create test subdir");
    
    let fs = LocalFileSystem::new();
    let result = fs.list_dir(&temp_dir).await;
    
    cleanup_temp_dir(&temp_dir).await.ok();
    
    assert!(result.is_ok());
    let files = result.unwrap();
    assert_eq!(files.len(), 2);
    
    // Should find both file and directory
    let names: Vec<&String> = files.iter().map(|f| &f.name).collect();
    assert!(names.contains(&&"test_file.txt".to_string()));
    assert!(names.contains(&&"test_subdir".to_string()));
}

#[tokio::test]
async fn test_file_operations() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let source_file = temp_dir.join("source.txt");
    let dest_file = temp_dir.join("dest.txt");
    let content = b"Test file content";
    
    let fs = LocalFileSystem::new();
    
    // Test write_file
    let write_result = fs.write_file(&source_file, content).await;
    assert!(write_result.is_ok());
    
    // Test read_file
    let read_result = fs.read_file(&source_file).await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), content);
    
    // Test copy_file
    let copy_result = fs.copy_file(&source_file, &dest_file).await;
    assert!(copy_result.is_ok());
    
    // Verify copy worked
    let copied_content = fs.read_file(&dest_file).await;
    assert!(copied_content.is_ok());
    assert_eq!(copied_content.unwrap(), content);
    
    // Test move_file
    let moved_file = temp_dir.join("moved.txt");
    let move_result = fs.move_file(&dest_file, &moved_file).await;
    assert!(move_result.is_ok());
    
    // Original should be gone, moved should exist
    assert!(fs.get_file_info(&dest_file).await.is_err());
    assert!(fs.get_file_info(&moved_file).await.is_ok());
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_directory_operations() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let test_subdir = temp_dir.join("test_subdir");
    
    let fs = LocalFileSystem::new();
    
    // Test create_dir
    let create_result = fs.create_dir(&test_subdir).await;
    assert!(create_result.is_ok());
    
    // Verify directory exists
    let info = fs.get_file_info(&test_subdir).await;
    assert!(info.is_ok());
    assert_eq!(info.unwrap().file_type, FileType::Directory);
    
    // Test delete_dir
    let delete_result = fs.delete_dir(&test_subdir).await;
    assert!(delete_result.is_ok());
    
    // Verify directory is gone
    assert!(fs.get_file_info(&test_subdir).await.is_err());
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_error_handling() {
    let fs = LocalFileSystem::new();
    let non_existent_path = Path::new("/this/path/does/not/exist");
    
    // Test file not found error
    let result = fs.get_file_info(non_existent_path).await;
    assert!(result.is_err());
    match result.err().unwrap() {
        FileError::NotFound { path } => {
            assert!(path.contains("does/not/exist"));
        },
        _ => panic!("Expected NotFound error"),
    }
    
    // Test directory not found error
    let list_result = fs.list_dir(non_existent_path).await;
    assert!(list_result.is_err());
    match list_result.err().unwrap() {
        FileError::NotFound { .. } => {}, // Expected
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_file_size_formatting() {
    let sizes = vec![
        (0, "0 B"),
        (1023, "1023 B"),
        (1024, "1.0 KB"),
        (1536, "1.5 KB"),
        (1048576, "1.0 MB"),
        (1073741824, "1.0 GB"),
    ];
    
    for (size, expected) in sizes {
        assert_eq!(utils::format_file_size(size), expected);
    }
}

#[tokio::test]
async fn test_hidden_file_detection() {
    assert!(utils::is_hidden(".hidden_file"));
    assert!(utils::is_hidden(".gitignore"));
    assert!(!utils::is_hidden("regular_file.txt"));
    assert!(!utils::is_hidden("file.hidden")); // . not at start
}

#[tokio::test]
async fn test_extension_extraction() {
    let test_cases = vec![
        ("/path/to/file.txt", Some("txt")),
        ("/path/to/file.TAR.GZ", Some("gz")),
        ("/path/to/file", None),
        ("/path/to/.hidden", None),
        ("/path/to/.hidden.txt", Some("txt")),
    ];
    
    for (path_str, expected) in test_cases {
        let path = Path::new(path_str);
        let result = utils::get_extension(path);
        let expected = expected.map(|s| s.to_string());
        assert_eq!(result, expected, "Failed for path: {}", path_str);
    }
}