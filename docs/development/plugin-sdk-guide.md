# Nimbus Plugin SDK Guide

## Overview

The Nimbus Plugin SDK provides a powerful framework for extending the file manager with custom functionality. This guide covers everything you need to know to develop, build, and distribute plugins for Nimbus.

## Plugin Architecture

Nimbus supports three types of plugins:

### 1. Content Plugins
Extend file metadata and add custom columns to the file list view.
- **Use Cases**: Media metadata extraction, code analysis, document properties
- **Examples**: ID3 tag reader, EXIF data extractor, Git status viewer

### 2. Protocol Plugins
Add support for custom remote file systems and protocols.
- **Use Cases**: Cloud storage integration, custom network protocols
- **Examples**: WebDAV, S3, custom REST APIs

### 3. Viewer Plugins
Provide custom file viewers and editors for specific file types.
- **Use Cases**: Specialized file formats, syntax highlighting, preview generation
- **Examples**: Markdown renderer, 3D model viewer, audio waveform display

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo (included with Rust)
- Basic knowledge of async Rust programming

### Creating Your First Plugin

#### 1. Project Setup

Create a new Rust library project:

```bash
cargo new --lib my_plugin
cd my_plugin
```

Update your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
nimbus-plugin-sdk = { path = "../path/to/nimbus/src-tauri/crates/plugin-sdk" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }

[package.metadata.plugin]
name = "My Plugin"
version = "1.0.0"
description = "A sample plugin for demonstration"
author = "Your Name"
plugin_type = "Content"  # or "Protocol" or "Viewer"
```

#### 2. Plugin Manifest

Create a `plugin.json` manifest file:

```json
{
  "info": {
    "name": "My Plugin",
    "version": "1.0.0",
    "description": "A sample plugin for demonstration",
    "author": "Your Name",
    "homepage": "https://github.com/user/my-plugin",
    "license": "MIT",
    "tags": ["demo", "example"],
    "minVersion": "0.1.0"
  },
  "pluginType": "Content",
  "entryPoint": "plugin_main",
  "dependencies": [],
  "platforms": [
    {
      "os": "windows",
      "arch": "x86_64"
    },
    {
      "os": "macos", 
      "arch": "x86_64"
    },
    {
      "os": "linux",
      "arch": "x86_64"
    }
  ]
}
```

## Content Plugin Development

Content plugins extend file information by providing additional metadata and custom columns.

### Basic Content Plugin

```rust
use async_trait::async_trait;
use nimbus_plugin_sdk::{
    ContentPlugin, PluginInfo, Result,
    content::{ColumnDefinition, ColumnValue, ColumnAlignment}
};
use std::collections::HashMap;
use std::path::Path;

pub struct MyContentPlugin;

#[async_trait]
impl ContentPlugin for MyContentPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Content Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Adds custom metadata".to_string(),
            author: "Your Name".to_string(),
            tags: vec!["metadata".to_string()],
            // ... other fields
        }
    }
    
    fn supported_extensions(&self) -> Vec<String> {
        vec!["txt".to_string(), "md".to_string()]
    }
    
    fn column_definitions(&self) -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition {
                id: "my_plugin.word_count".to_string(),
                name: "Word Count".to_string(),
                description: Some("Number of words in file".to_string()),
                width: 80,
                sortable: true,
                visible_by_default: false,
                alignment: ColumnAlignment::Right,
            }
        ]
    }
    
    async fn get_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        // Read file and extract metadata
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let word_count = content.split_whitespace().count();
            metadata.insert("word_count".to_string(), word_count.to_string());
            metadata.insert("line_count".to_string(), content.lines().count().to_string());
        }
        
        Ok(metadata)
    }
    
    async fn get_columns(&self, file_path: &Path) -> Result<HashMap<String, ColumnValue>> {
        let mut columns = HashMap::new();
        
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let word_count = content.split_whitespace().count();
            columns.insert(
                "word_count".to_string(),
                ColumnValue::Number(word_count as f64)
            );
        }
        
        Ok(columns)
    }
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn plugin_main() -> *mut dyn ContentPlugin {
    Box::into_raw(Box::new(MyContentPlugin))
}
```

### Advanced Content Plugin Features

#### Custom Column Types

```rust
// Different column value types
columns.insert("status".to_string(), ColumnValue::Text("Ready".to_string()));
columns.insert("progress".to_string(), ColumnValue::Progress(0.75)); // 75%
columns.insert("size_formatted".to_string(), ColumnValue::FileSize(1024));
columns.insert("is_valid".to_string(), ColumnValue::Boolean(true));
columns.insert("timestamp".to_string(), ColumnValue::DateTime("2024-01-01T00:00:00Z".to_string()));

// Custom formatted values with tooltips
columns.insert("custom".to_string(), ColumnValue::Custom {
    display: "★★★★☆".to_string(),
    tooltip: Some("Rating: 4/5 stars".to_string()),
    sort_value: Some("4".to_string()),
});
```

#### Thumbnail Generation

```rust
async fn get_thumbnail(&self, file_path: &Path, size: u32) -> Result<Option<Vec<u8>>> {
    // Generate thumbnail for supported file types
    if self.can_generate_thumbnail(file_path) {
        let thumbnail_data = generate_thumbnail(file_path, size)?;
        Ok(Some(thumbnail_data))
    } else {
        Ok(None)
    }
}
```

## Protocol Plugin Development

Protocol plugins add support for remote file systems and custom protocols.

### Basic Protocol Plugin

```rust
use async_trait::async_trait;
use nimbus_plugin_sdk::{
    ProtocolPlugin, RemoteClient, PluginInfo, Result,
    protocol::{ConnectionConfig, RemoteFileInfo, ProtocolCapabilities}
};

pub struct MyProtocolPlugin;

#[async_trait]
impl ProtocolPlugin for MyProtocolPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Protocol Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Custom protocol support".to_string(),
            author: "Your Name".to_string(),
            tags: vec!["protocol", "remote".to_string()],
            // ... other fields
        }
    }
    
    fn scheme(&self) -> String {
        "myprotocol".to_string()
    }
    
    fn default_port(&self) -> u16 {
        8080
    }
    
    fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            can_create_directories: true,
            can_delete: true,
            can_rename: true,
            can_resume_transfers: true,
            max_file_size: Some(1024 * 1024 * 1024), // 1GB
            ..Default::default()
        }
    }
    
    async fn create_client(&self, config: ConnectionConfig) -> Result<Box<dyn RemoteClient>> {
        let client = MyRemoteClient::new(config)?;
        Ok(Box::new(client))
    }
}

// Remote client implementation
pub struct MyRemoteClient {
    config: ConnectionConfig,
    // Connection state...
}

#[async_trait]
impl RemoteClient for MyRemoteClient {
    async fn connect(&mut self) -> Result<()> {
        // Establish connection
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        // Close connection
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        // Check connection status
        true
    }
    
    async fn list_directory(&self, path: &str) -> Result<Vec<RemoteFileInfo>> {
        // List directory contents
        Ok(vec![])
    }
    
    // Implement other RemoteClient methods...
}
```

## Viewer Plugin Development

Viewer plugins provide custom file viewers and editors.

### Basic Viewer Plugin

```rust
use async_trait::async_trait;
use nimbus_plugin_sdk::{
    ViewerPlugin, PluginInfo, Result,
    viewer::{ViewerContent, ViewerCapabilities, ViewerOptions}
};
use std::path::Path;

pub struct MyViewerPlugin;

#[async_trait]
impl ViewerPlugin for MyViewerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Viewer Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Custom file viewer".to_string(),
            author: "Your Name".to_string(),
            tags: vec!["viewer", "custom".to_string()],
            // ... other fields
        }
    }
    
    fn supported_extensions(&self) -> Vec<String> {
        vec!["myfile".to_string()]
    }
    
    fn capabilities(&self) -> ViewerCapabilities {
        ViewerCapabilities {
            can_view: true,
            can_edit: true,
            can_save: true,
            can_search: true,
            max_file_size: Some(50 * 1024 * 1024), // 50MB
            ..Default::default()
        }
    }
    
    async fn view_file(
        &self,
        file_path: &Path,
        options: &ViewerOptions,
    ) -> Result<ViewerContent> {
        // Read and process file
        let content = std::fs::read_to_string(file_path)?;
        
        // Return appropriate content type
        Ok(ViewerContent::text_with_language(content, "myformat".to_string()))
    }
    
    async fn save_file(
        &self,
        file_path: &Path,
        content: &ViewerContent,
        options: &ViewerOptions,
    ) -> Result<()> {
        // Save edited content
        if let ViewerContent::Text { content, .. } = content {
            std::fs::write(file_path, content)?;
        }
        Ok(())
    }
}
```

## Building and Testing

### Building Your Plugin

```bash
# Build in development mode
cargo build

# Build for release
cargo build --release

# The plugin library will be in target/debug/ or target/release/
```

### Testing Your Plugin

Create unit tests for your plugin logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_metadata_extraction() {
        let plugin = MyContentPlugin;
        let test_file = PathBuf::from("test.txt");
        
        // Create test file
        std::fs::write(&test_file, "Hello world test").unwrap();
        
        let metadata = plugin.get_metadata(&test_file).await.unwrap();
        assert_eq!(metadata.get("word_count"), Some(&"3".to_string()));
        
        // Cleanup
        std::fs::remove_file(&test_file).unwrap();
    }
}
```

### Integration Testing

Test your plugin with the Nimbus plugin system:

```rust
#[tokio::test]
async fn test_plugin_loading() {
    use nimbus_plugin_sdk::manager::PluginManager;
    use semver::Version;
    
    let mut manager = PluginManager::new(Version::parse("0.1.0").unwrap());
    let plugin_id = manager.load_plugin(Path::new("target/debug/libmy_plugin.so")).await.unwrap();
    
    let info = manager.get_plugin_info(&plugin_id).await;
    assert!(info.is_some());
}
```

## Best Practices

### Error Handling

- Use the provided `Result<T>` type for all fallible operations
- Provide meaningful error messages
- Handle edge cases gracefully

```rust
async fn get_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| PluginError::execution_error(
            self.info().name,
            format!("Failed to read file: {}", e)
        ))?;
    
    // Process content...
    Ok(metadata)
}
```

### Performance Considerations

- Use async/await for I/O operations
- Avoid blocking the main thread
- Cache expensive computations
- Handle large files efficiently

```rust
// Good: Streaming large files
async fn process_large_file(&self, file_path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = [0u8; 8192];
    
    // Process in chunks
    while let Ok(bytes_read) = file.read(&mut buffer).await {
        if bytes_read == 0 { break; }
        // Process chunk...
    }
    
    Ok("processed".to_string())
}
```

### Memory Management

- Use appropriate data structures
- Avoid memory leaks with proper cleanup
- Consider memory usage with large datasets

### Security

- Validate all input parameters
- Sanitize file paths to prevent directory traversal
- Handle untrusted file content safely

```rust
fn validate_path(path: &Path) -> Result<()> {
    let canonical = path.canonicalize()
        .map_err(|_| PluginError::invalid_path(path.to_path_buf()))?;
    
    // Additional security checks...
    Ok(())
}
```

## Plugin Distribution

### Packaging

Create a plugin package with:
- Compiled library (`.so`, `.dll`, or `.dylib`)
- Manifest file (`plugin.json`)
- Documentation (`README.md`)
- License file

### Installation

Users can install plugins by:
1. Copying files to the plugin directory
2. Using the plugin manager UI
3. Installing from URL or registry

### Versioning

Follow semantic versioning:
- `MAJOR.MINOR.PATCH`
- Increment MAJOR for breaking changes
- Increment MINOR for new features
- Increment PATCH for bug fixes

## API Reference

### Core Types

```rust
// Plugin information
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
    pub min_version: String,
    pub max_version: Option<String>,
}

// Plugin types
pub enum PluginType {
    Content,
    Protocol,
    Viewer,
}

// Plugin status
pub enum PluginStatus {
    Active,
    Inactive,
    Loading,
    Error,
    Unloaded,
}
```

### Error Types

```rust
pub enum PluginError {
    NotFound { path: PathBuf },
    LoadingFailed { path: PathBuf, source: Box<dyn std::error::Error> },
    InvalidManifest { path: PathBuf, reason: String },
    VersionIncompatible { name: String, version: String, required: String },
    PlatformIncompatible { name: String },
    ConfigurationError { name: String, message: String },
    ExecutionError { plugin: String, message: String },
    DependencyMissing { plugin: String, dependency: String },
    PluginAlreadyLoaded { name: String },
    InitializationError { message: String },
}
```

## Examples and Templates

Complete example plugins are available in the `examples/` directory:

- `examples/content-plugin/` - Basic content plugin with metadata extraction
- `examples/protocol-plugin/` - Protocol plugin for custom remote filesystem
- `examples/viewer-plugin/` - Viewer plugin with custom rendering
- `examples/advanced-content/` - Advanced content plugin with thumbnails
- `examples/protocol-webdav/` - WebDAV protocol implementation

## Troubleshooting

### Common Issues

**Plugin not loading**:
- Check manifest file syntax
- Verify plugin library compatibility
- Check plugin dependencies
- Ensure proper export symbols

**Performance issues**:
- Profile your plugin code
- Use async operations for I/O
- Avoid blocking main thread
- Optimize memory usage

**Crashes**:
- Check for null pointer dereferences
- Validate all inputs
- Handle errors gracefully
- Use safe Rust practices

### Debugging

Enable debug logging in your plugin:

```rust
use log::{debug, info, warn, error};

async fn get_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
    debug!("Processing file: {:?}", file_path);
    
    // Your code here...
    
    info!("Extracted {} metadata fields", metadata.len());
    Ok(metadata)
}
```

### Getting Help

- Check the official documentation
- Browse example plugins
- Join the community forum
- Report bugs on GitHub

## Contributing

We welcome contributions to the Plugin SDK:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

See `CONTRIBUTING.md` for detailed guidelines.

## License

The Nimbus Plugin SDK is licensed under the MIT License. See `LICENSE` file for details.