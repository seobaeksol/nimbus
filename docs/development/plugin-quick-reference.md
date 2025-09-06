# Plugin SDK Quick Reference

## Plugin Types at a Glance

| Type | Purpose | Key Traits | Common Use Cases |
|------|---------|------------|------------------|
| **Content** | Extend file metadata | `ContentPlugin` | Media info, code analysis, Git status |
| **Protocol** | Remote filesystems | `ProtocolPlugin` + `RemoteClient` | Cloud storage, FTP/SFTP, WebDAV |
| **Viewer** | Custom file viewers | `ViewerPlugin` | Markdown, 3D models, hex editor |

## Essential Imports

```rust
use async_trait::async_trait;
use nimbus_plugin_sdk::{
    // Core types
    PluginInfo, PluginError, Result,
    
    // Content plugins
    ContentPlugin,
    content::{ColumnDefinition, ColumnValue, ColumnAlignment},
    
    // Protocol plugins  
    ProtocolPlugin, RemoteClient,
    protocol::{ConnectionConfig, RemoteFileInfo, ProtocolCapabilities},
    
    // Viewer plugins
    ViewerPlugin,
    viewer::{ViewerContent, ViewerCapabilities, ViewerOptions},
};
use std::collections::HashMap;
use std::path::Path;
```

## Basic Plugin Structure

### Content Plugin Template

```rust
pub struct MyContentPlugin;

#[async_trait]
impl ContentPlugin for MyContentPlugin {
    fn info(&self) -> PluginInfo { /* ... */ }
    fn supported_extensions(&self) -> Vec<String> { vec!["txt".into()] }
    async fn get_metadata(&self, file_path: &Path) -> Result<HashMap<String, String>> {
        // Extract metadata
        Ok(HashMap::new())
    }
}

#[no_mangle]
pub extern "C" fn plugin_main() -> *mut dyn ContentPlugin {
    Box::into_raw(Box::new(MyContentPlugin))
}
```

### Protocol Plugin Template

```rust
pub struct MyProtocolPlugin;

#[async_trait]
impl ProtocolPlugin for MyProtocolPlugin {
    fn info(&self) -> PluginInfo { /* ... */ }
    fn scheme(&self) -> String { "myproto".into() }
    fn default_port(&self) -> u16 { 8080 }
    async fn create_client(&self, config: ConnectionConfig) -> Result<Box<dyn RemoteClient>> {
        Ok(Box::new(MyRemoteClient::new(config)?))
    }
}
```

### Viewer Plugin Template

```rust
pub struct MyViewerPlugin;

#[async_trait]
impl ViewerPlugin for MyViewerPlugin {
    fn info(&self) -> PluginInfo { /* ... */ }
    fn supported_extensions(&self) -> Vec<String> { vec!["myfile".into()] }
    async fn view_file(&self, file_path: &Path, options: &ViewerOptions) -> Result<ViewerContent> {
        let content = std::fs::read_to_string(file_path)?;
        Ok(ViewerContent::text(content))
    }
}
```

## Common Column Types

```rust
// Text column
ColumnValue::Text("Status: Ready".into())

// Numeric column  
ColumnValue::Number(42.0)

// Boolean column (Yes/No)
ColumnValue::Boolean(true)

// File size (auto-formatted)
ColumnValue::FileSize(1024)

// Progress bar (0.0 to 1.0)
ColumnValue::Progress(0.75)

// Date/time (ISO 8601)
ColumnValue::DateTime("2024-01-01T00:00:00Z".into())

// Custom formatted with tooltip
ColumnValue::Custom {
    display: "★★★★☆".into(),
    tooltip: Some("4/5 stars".into()),
    sort_value: Some("4".into()),
}
```

## Viewer Content Types

```rust
// Plain text
ViewerContent::text("Hello world".into())

// Text with syntax highlighting
ViewerContent::text_with_language("fn main() {}".into(), "rust".into())

// HTML content
ViewerContent::html("<h1>Title</h1>".into())

// Image data
ViewerContent::image(image_bytes, "png".into())

// Structured data (JSON, etc.)
ViewerContent::structured(json_value, "json".into())

// Error state
ViewerContent::error("Failed to load".into())
```

## Error Handling Patterns

```rust
// File I/O errors
std::fs::read_to_string(path)
    .map_err(|e| PluginError::execution_error(
        self.info().name,
        format!("Failed to read file: {}", e)
    ))?;

// Configuration errors
if config.host.is_empty() {
    return Err(PluginError::configuration_error(
        self.info().name,
        "Host cannot be empty".into()
    ));
}

// Version compatibility
if !manifest.is_compatible_with(&nimbus_version) {
    return Err(PluginError::version_incompatible(
        manifest.info.name,
        manifest.info.version.to_string(),
        nimbus_version.to_string(),
    ));
}
```

## Common Async Patterns

```rust
// Reading files asynchronously
let content = tokio::fs::read_to_string(file_path).await?;

// Making HTTP requests
let response = reqwest::get(&url).await?;
let data = response.bytes().await?;

// Spawning background tasks
let handle = tokio::spawn(async move {
    // Long-running operation
});

// Timeout operations
let result = tokio::time::timeout(
    Duration::from_secs(30),
    operation()
).await??;
```

## Build Configuration

### Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
nimbus-plugin-sdk = { path = "../path/to/sdk" }
serde = { version = "1.0", features = ["derive"] }
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }

[package.metadata.plugin]
name = "My Plugin"
version = "1.0.0"
plugin_type = "Content"  # or "Protocol" or "Viewer"
```

### Manifest (plugin.json)

```json
{
  "info": {
    "name": "My Plugin",
    "version": "1.0.0", 
    "description": "Plugin description",
    "author": "Your Name",
    "tags": ["tag1", "tag2"],
    "minVersion": "0.1.0"
  },
  "pluginType": "Content",
  "entryPoint": "plugin_main",
  "dependencies": [],
  "platforms": [
    { "os": "windows", "arch": "x86_64" },
    { "os": "macos", "arch": "x86_64" },
    { "os": "linux", "arch": "x86_64" }
  ]
}
```

## Testing Quick Setup

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_plugin() {
        let plugin = MyPlugin;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        std::fs::write(&test_file, "test content").unwrap();
        
        let result = plugin.get_metadata(&test_file).await.unwrap();
        assert!(!result.is_empty());
    }
}
```

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check for errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## File Extensions by Platform

| Platform | Extension | Location |
|----------|-----------|----------|
| Linux | `.so` | `target/debug/libplugin_name.so` |
| macOS | `.dylib` | `target/debug/libplugin_name.dylib` |
| Windows | `.dll` | `target/debug/plugin_name.dll` |

## Common Pitfalls

❌ **Don't do this:**
```rust
// Blocking I/O in async context
async fn bad_example(&self, path: &Path) -> Result<String> {
    std::fs::read_to_string(path) // Blocks!
}

// Not validating inputs  
fn unsafe_path(&self, path: &str) {
    std::fs::read(&format!("../{}", path)) // Directory traversal!
}
```

✅ **Do this instead:**
```rust
// Async I/O
async fn good_example(&self, path: &Path) -> Result<String> {
    tokio::fs::read_to_string(path).await
}

// Input validation
fn safe_path(&self, path: &Path) -> Result<()> {
    let canonical = path.canonicalize()?;
    // Validate canonical path...
    Ok(())
}
```

## Performance Tips

- ✅ Use `tokio::fs` for file I/O
- ✅ Stream large files instead of loading entirely  
- ✅ Cache expensive computations
- ✅ Use `Arc<Mutex<>>` or `Arc<RwLock<>>` for shared state
- ✅ Avoid blocking operations in async functions
- ❌ Don't use `std::thread::sleep` in async code
- ❌ Don't load entire large files into memory

## Debugging

```rust
use log::{debug, info, warn, error};

// Add to plugin methods
debug!("Processing file: {:?}", file_path);
info!("Plugin initialized successfully");
warn!("File size exceeds recommended limit: {} bytes", size);
error!("Failed to connect: {}", error);
```

## Common Dependencies

```toml
# Logging
log = "0.4"

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Image processing  
image = "0.24"

# Archive handling
zip = "0.6"

# Database
rusqlite = "0.29"

# Regex
regex = "1.0"

# Temporary files
tempfile = "3.0"
```