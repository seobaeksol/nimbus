//! Integration tests for Tauri commands

use super::*;
use files::*;
use system::*;
use std::path::PathBuf;
use tokio::fs;

/// Create a temporary directory for testing
async fn create_temp_dir() -> Result<PathBuf, std::io::Error> {
    let temp_dir = std::env::temp_dir().join(format!("nimbus_cmd_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).await?;
    Ok(temp_dir)
}

/// Clean up test directory
async fn cleanup_temp_dir(path: &PathBuf) -> Result<(), std::io::Error> {
    if path.exists() {
        fs::remove_dir_all(path).await?;
    }
    Ok(())
}

#[tokio::test]
async fn test_list_dir_command() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    
    // Create test files
    let test_file = temp_dir.join("test.txt");
    let test_subdir = temp_dir.join("subdir");
    
    fs::write(&test_file, b"content").await.expect("Failed to create test file");
    fs::create_dir(&test_subdir).await.expect("Failed to create test subdir");
    
    // Test the command
    let result = list_dir(temp_dir.to_string_lossy().to_string()).await;
    
    cleanup_temp_dir(&temp_dir).await.ok();
    
    assert!(result.is_ok());
    let files = result.unwrap();
    assert_eq!(files.len(), 2);
    
    let names: Vec<&String> = files.iter().map(|f| &f.name).collect();
    assert!(names.contains(&&"test.txt".to_string()));
    assert!(names.contains(&&"subdir".to_string()));
}

#[tokio::test]
async fn test_list_dir_nonexistent() {
    let result = list_dir("/nonexistent/path".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("Failed to list directory"));
}

#[tokio::test]
async fn test_get_file_info_command() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let test_file = temp_dir.join("info_test.txt");
    let content = b"File info test content";
    
    fs::write(&test_file, content).await.expect("Failed to create test file");
    
    let result = get_file_info(test_file.to_string_lossy().to_string()).await;
    
    cleanup_temp_dir(&temp_dir).await.ok();
    
    assert!(result.is_ok());
    let file_info = result.unwrap();
    assert_eq!(file_info.name, "info_test.txt");
    assert_eq!(file_info.size, content.len() as u64);
    assert_eq!(file_info.extension, Some("txt".to_string()));
}

#[tokio::test]
async fn test_create_directory_command() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let new_dir_name = "new_test_dir";
    
    let result = create_directory(
        temp_dir.to_string_lossy().to_string(),
        new_dir_name.to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify directory was created
    let created_dir = temp_dir.join(new_dir_name);
    assert!(created_dir.exists());
    assert!(created_dir.is_dir());
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_create_file_command() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let new_file_name = "new_test_file.txt";
    
    let result = create_file(
        temp_dir.to_string_lossy().to_string(),
        new_file_name.to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify file was created
    let created_file = temp_dir.join(new_file_name);
    assert!(created_file.exists());
    assert!(created_file.is_file());
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_copy_item_file() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let source_file = temp_dir.join("source.txt");
    let dest_file = temp_dir.join("destination.txt");
    let content = b"Copy test content";
    
    fs::write(&source_file, content).await.expect("Failed to create source file");
    
    let result = copy_item(
        source_file.to_string_lossy().to_string(),
        dest_file.to_string_lossy().to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify both files exist with same content
    assert!(source_file.exists());
    assert!(dest_file.exists());
    
    let dest_content = fs::read(&dest_file).await.expect("Failed to read dest file");
    assert_eq!(dest_content, content);
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_move_item_command() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let source_file = temp_dir.join("source.txt");
    let dest_file = temp_dir.join("moved.txt");
    let content = b"Move test content";
    
    fs::write(&source_file, content).await.expect("Failed to create source file");
    
    let result = move_item(
        source_file.to_string_lossy().to_string(),
        dest_file.to_string_lossy().to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify source is gone, destination exists
    assert!(!source_file.exists());
    assert!(dest_file.exists());
    
    let dest_content = fs::read(&dest_file).await.expect("Failed to read moved file");
    assert_eq!(dest_content, content);
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_delete_item_file() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let test_file = temp_dir.join("to_delete.txt");
    
    fs::write(&test_file, b"Delete me").await.expect("Failed to create test file");
    assert!(test_file.exists());
    
    let result = delete_item(test_file.to_string_lossy().to_string()).await;
    assert!(result.is_ok());
    
    // Verify file is gone
    assert!(!test_file.exists());
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_delete_item_directory() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let test_subdir = temp_dir.join("to_delete_dir");
    
    fs::create_dir(&test_subdir).await.expect("Failed to create test directory");
    assert!(test_subdir.exists());
    
    let result = delete_item(test_subdir.to_string_lossy().to_string()).await;
    assert!(result.is_ok());
    
    // Verify directory is gone
    assert!(!test_subdir.exists());
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_rename_item_command() {
    let temp_dir = create_temp_dir().await.expect("Failed to create temp dir");
    let old_file = temp_dir.join("old_name.txt");
    let content = b"Rename test content";
    
    fs::write(&old_file, content).await.expect("Failed to create test file");
    
    let result = rename_item(
        old_file.to_string_lossy().to_string(),
        "new_name.txt".to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify old file is gone, new file exists
    assert!(!old_file.exists());
    
    let new_file = temp_dir.join("new_name.txt");
    assert!(new_file.exists());
    
    let new_content = fs::read(&new_file).await.expect("Failed to read renamed file");
    assert_eq!(new_content, content);
    
    cleanup_temp_dir(&temp_dir).await.ok();
}

#[tokio::test]
async fn test_get_system_paths_command() {
    let result = get_system_paths().await;
    assert!(result.is_ok());
    
    let paths = result.unwrap();
    assert!(paths.is_object());
    
    // Should contain at least home directory
    let obj = paths.as_object().unwrap();
    assert!(obj.contains_key("home"));
    
    // Home should be a string
    assert!(obj["home"].is_string());
}

#[tokio::test]
async fn test_resolve_path_command() {
    // Test home directory resolution
    let result = resolve_path("~".to_string()).await;
    assert!(result.is_ok());
    let home_path = result.unwrap();
    assert!(!home_path.is_empty());
    assert!(!home_path.contains('~')); // Should be expanded
    
    // Test tilde expansion
    let result = resolve_path("~/Documents".to_string()).await;
    assert!(result.is_ok());
    let docs_path = result.unwrap();
    assert!(docs_path.contains("Documents"));
    assert!(!docs_path.contains('~')); // Should be expanded
    
    // Test system directory alias
    let result = resolve_path("documents".to_string()).await;
    assert!(result.is_ok());
    let docs_alias = result.unwrap();
    assert!(!docs_alias.is_empty());
}

#[tokio::test]
async fn test_system_commands() {
    // Test get_system_info
    let result = get_system_info().await;
    assert!(result.is_ok());
    
    let info = result.unwrap();
    assert!(!info.platform.is_empty());
    assert!(!info.hostname.is_empty());
    assert!(!info.username.is_empty());
    
    // Test greet command
    let result = greet("Test User").await;
    assert!(result.is_ok());
    
    let greeting = result.unwrap();
    assert!(greeting.contains("Test User"));
}

#[tokio::test]
async fn test_error_handling_consistency() {
    // Test that all commands return consistent error formats
    let invalid_path = "/definitely/does/not/exist/anywhere";
    
    let list_result = list_dir(invalid_path.to_string()).await;
    assert!(list_result.is_err());
    assert!(list_result.err().unwrap().contains("Failed to list directory"));
    
    let info_result = get_file_info(invalid_path.to_string()).await;
    assert!(info_result.is_err());
    assert!(info_result.err().unwrap().contains("Failed to get file info"));
    
    let copy_result = copy_item(
        invalid_path.to_string(),
        "/another/invalid/path".to_string(),
    ).await;
    assert!(copy_result.is_err());
    assert!(copy_result.err().unwrap().contains("Source file not found"));
}