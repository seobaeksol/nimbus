use crate::{FileViewer, ViewerCapabilities, ViewerContent, ViewerOptions, ViewerError, SearchOptions, SearchResult, BinaryDisplayFormat};
use async_trait::async_trait;
use std::path::Path;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;
use memmap2::MmapOptions;

/// Binary file viewer with hex display and pattern search
pub struct BinaryViewer {
    max_file_size: u64,
    default_bytes_per_row: usize,
}

impl BinaryViewer {
    /// Create a new binary viewer with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 1024 * 1024 * 1024, // 1GB default limit for binary files
            default_bytes_per_row: 16,
        }
    }

    /// Create a new binary viewer with custom settings
    pub fn with_settings(max_size: u64, bytes_per_row: usize) -> Self {
        Self {
            max_file_size: max_size,
            default_bytes_per_row: bytes_per_row,
        }
    }

    /// Check if a file is likely to be binary based on content
    async fn is_likely_binary(&self, path: &Path) -> Result<bool, ViewerError> {
        const SAMPLE_SIZE: usize = 8192; // Check first 8KB
        
        let mut file = File::open(path).map_err(ViewerError::Io)?;
        let mut buffer = vec![0u8; SAMPLE_SIZE];
        let bytes_read = file.read(&mut buffer).map_err(ViewerError::Io)?;
        buffer.truncate(bytes_read);
        
        // If file is empty, consider it binary
        if buffer.is_empty() {
            return Ok(true);
        }
        
        // Check for null bytes (strong indicator of binary content)
        if buffer.contains(&0) {
            return Ok(true);
        }
        
        // Check for high ratio of non-printable ASCII characters
        let non_printable_count = buffer.iter()
            .filter(|&&byte| byte < 32 && byte != b'\t' && byte != b'\n' && byte != b'\r')
            .count();
        
        let ratio = non_printable_count as f64 / buffer.len() as f64;
        Ok(ratio > 0.1) // If more than 10% non-printable characters, consider binary
    }

    /// Convert byte to printable ASCII character for mixed display
    fn byte_to_ascii_char(&self, byte: u8) -> char {
        if byte >= 32 && byte <= 126 {
            byte as char
        } else {
            '.'
        }
    }

    /// Format bytes as hexadecimal string
    fn format_hex_bytes(&self, bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format bytes as ASCII string
    fn format_ascii_bytes(&self, bytes: &[u8]) -> String {
        bytes.iter()
            .map(|&b| self.byte_to_ascii_char(b))
            .collect()
    }

    /// Read binary data with memory mapping for large files
    async fn read_binary_data(
        &self,
        path: &Path,
        offset: u64,
        length: u64,
        file_size: u64,
    ) -> Result<Vec<u8>, ViewerError> {
        let mut file = File::open(path).map_err(ViewerError::Io)?;
        
        // For small reads or files, use regular I/O
        if length < 1024 * 1024 || file_size < 10 * 1024 * 1024 {
            file.seek(SeekFrom::Start(offset)).map_err(ViewerError::Io)?;
            let mut buffer = vec![0u8; length as usize];
            let bytes_read = file.read(&mut buffer).map_err(ViewerError::Io)?;
            buffer.truncate(bytes_read);
            Ok(buffer)
        } else {
            // Use memory mapping for larger files
            let mmap = unsafe {
                MmapOptions::new()
                    .offset(offset)
                    .len(length as usize)
                    .map(&file)
                    .map_err(|e| ViewerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            Ok(mmap.to_vec())
        }
    }

    /// Search for byte pattern in binary data
    fn search_binary_pattern(&self, data: &[u8], pattern: &[u8]) -> Vec<usize> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let pattern_len = pattern.len();
        
        if data.len() < pattern_len {
            return matches;
        }

        for i in 0..=(data.len() - pattern_len) {
            if data[i..i + pattern_len] == *pattern {
                matches.push(i);
            }
        }

        matches
    }

    /// Parse hex string to bytes
    fn parse_hex_string(&self, hex_str: &str) -> Result<Vec<u8>, ViewerError> {
        let hex_clean = hex_str.replace(' ', "").replace('\n', "").replace('\t', "");
        
        if hex_clean.len() % 2 != 0 {
            return Err(ViewerError::Other("Hex string must have even number of characters".to_string()));
        }

        let mut bytes = Vec::new();
        for chunk in hex_clean.as_bytes().chunks(2) {
            let hex_byte = std::str::from_utf8(chunk).map_err(|_| {
                ViewerError::Other("Invalid UTF-8 in hex string".to_string())
            })?;
            
            let byte = u8::from_str_radix(hex_byte, 16).map_err(|_| {
                ViewerError::Other(format!("Invalid hex byte: {}", hex_byte))
            })?;
            
            bytes.push(byte);
        }

        Ok(bytes)
    }

    /// Get all file extensions that should be treated as binary
    fn get_binary_extensions(&self) -> Vec<String> {
        vec![
            // Executables
            "exe", "dll", "so", "dylib", "bin", "app",
            // Archives (when not handled by archive viewer)
            "tar", "gz", "bz2", "xz", "lz4",
            // Media files
            "mp3", "mp4", "avi", "mkv", "wav", "flac",
            // Database files
            "db", "sqlite", "mdb",
            // Office files
            "doc", "docx", "xls", "xlsx", "ppt", "pptx",
            // Font files
            "ttf", "otf", "woff", "woff2",
            // System files
            "sys", "drv", "lib", "a", "o", "obj",
            // Compiled files
            "class", "pyc", "pyo",
        ].iter().map(|s| s.to_string()).collect()
    }
}

impl Default for BinaryViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileViewer for BinaryViewer {
    fn capabilities(&self) -> ViewerCapabilities {
        ViewerCapabilities {
            name: "Binary Viewer".to_string(),
            description: "View binary files in hex, ASCII, or mixed format with pattern search".to_string(),
            supported_extensions: self.get_binary_extensions(),
            max_file_size: self.max_file_size,
            supports_search: true,
            supports_editing: false,
        }
    }

    fn can_handle(&self, path: &Path) -> bool {
        // Check by extension first
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();
            if self.get_binary_extensions().contains(&ext_lower) {
                return true;
            }
        }

        // For unknown extensions, we could check file content
        // This is a fallback viewer, so it can handle any file
        true
    }

    async fn view_file(
        &self,
        path: &Path,
        options: ViewerOptions,
    ) -> Result<ViewerContent, ViewerError> {
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ViewerError::FileNotFound {
                    path: path.to_string_lossy().to_string(),
                }
            } else {
                ViewerError::Io(e)
            }
        })?;

        let file_size = metadata.len();
        let max_size = options.max_size.unwrap_or(self.max_file_size);

        if file_size > max_size {
            return Err(ViewerError::FileTooLarge {
                size: file_size,
                max_size,
            });
        }

        // Determine what portion of the file to read
        let offset = options.offset.unwrap_or(0);
        let length = if let Some(length) = options.length {
            std::cmp::min(length, file_size - offset)
        } else {
            // Read up to 1MB by default for display
            std::cmp::min(1024 * 1024, file_size - offset)
        };

        // Read binary data
        let data = self.read_binary_data(path, offset, length, file_size).await?;

        // Determine display format
        let bytes_per_row = self.default_bytes_per_row;
        let display_format = BinaryDisplayFormat::Mixed { bytes_per_row };

        Ok(ViewerContent::Binary {
            data,
            offset,
            total_size: file_size,
            display_format,
        })
    }

    async fn search(
        &self,
        path: &Path,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, ViewerError> {
        // For binary search, we support hex pattern search
        let pattern = if query.chars().all(|c| c.is_ascii_hexdigit() || c.is_whitespace()) {
            // Treat as hex pattern
            self.parse_hex_string(query)?
        } else {
            // Treat as ASCII string
            query.as_bytes().to_vec()
        };

        if pattern.is_empty() {
            return Ok(Vec::new());
        }

        // Read file in chunks to avoid loading entire large files
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
        const OVERLAP_SIZE: usize = 1024; // Overlap to catch patterns across chunk boundaries

        let mut file = File::open(path).map_err(ViewerError::Io)?;
        let file_size = file.metadata().map_err(ViewerError::Io)?.len();
        let mut results = Vec::new();
        let mut offset = 0u64;

        while offset < file_size && results.len() < options.max_results {
            // Calculate chunk size
            let remaining = file_size - offset;
            let chunk_size = std::cmp::min(CHUNK_SIZE as u64, remaining) as usize;
            
            // Read chunk
            file.seek(SeekFrom::Start(offset)).map_err(ViewerError::Io)?;
            let mut buffer = vec![0u8; chunk_size];
            let bytes_read = file.read(&mut buffer).map_err(ViewerError::Io)?;
            buffer.truncate(bytes_read);

            // Search in this chunk
            let matches = self.search_binary_pattern(&buffer, &pattern);

            for match_pos in matches {
                let global_offset = offset + match_pos as u64;
                
                // Create context (show some bytes before and after)
                let context_size = 32;
                let context_start = if match_pos >= context_size { match_pos - context_size } else { 0 };
                let context_end = std::cmp::min(buffer.len(), match_pos + pattern.len() + context_size);
                
                let context_before = self.format_hex_bytes(&buffer[context_start..match_pos]);
                let matched_text = self.format_hex_bytes(&buffer[match_pos..match_pos + pattern.len()]);
                let context_after = self.format_hex_bytes(&buffer[match_pos + pattern.len()..context_end]);

                results.push(SearchResult {
                    line_number: None, // Binary files don't have line numbers
                    offset: global_offset,
                    length: pattern.len(),
                    context_before,
                    matched_text,
                    context_after,
                });

                if results.len() >= options.max_results {
                    break;
                }
            }

            // Move to next chunk with overlap
            if remaining <= CHUNK_SIZE as u64 {
                break;
            }
            
            offset += (CHUNK_SIZE - OVERLAP_SIZE) as u64;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_binary_viewer_capabilities() {
        let viewer = BinaryViewer::new();
        let caps = viewer.capabilities();
        
        assert_eq!(caps.name, "Binary Viewer");
        assert!(caps.supported_extensions.contains(&"exe".to_string()));
        assert!(caps.supported_extensions.contains(&"bin".to_string()));
        assert!(caps.supports_search);
    }

    #[tokio::test]
    async fn test_can_handle() {
        let viewer = BinaryViewer::new();
        
        assert!(viewer.can_handle(Path::new("test.exe")));
        assert!(viewer.can_handle(Path::new("test.bin")));
        assert!(viewer.can_handle(Path::new("unknown.xyz"))); // Fallback viewer
    }

    #[tokio::test]
    async fn test_hex_parsing() {
        let viewer = BinaryViewer::new();
        
        let result = viewer.parse_hex_string("41 42 43").unwrap();
        assert_eq!(result, vec![0x41, 0x42, 0x43]);
        
        let result = viewer.parse_hex_string("414243").unwrap();
        assert_eq!(result, vec![0x41, 0x42, 0x43]);
        
        assert!(viewer.parse_hex_string("41 4").is_err()); // Odd number of chars
        assert!(viewer.parse_hex_string("GG").is_err()); // Invalid hex
    }

    #[tokio::test]
    async fn test_ascii_conversion() {
        let viewer = BinaryViewer::new();
        
        assert_eq!(viewer.byte_to_ascii_char(65), 'A'); // Printable
        assert_eq!(viewer.byte_to_ascii_char(0), '.'); // Non-printable
        assert_eq!(viewer.byte_to_ascii_char(255), '.'); // Non-printable
    }

    #[tokio::test]
    async fn test_view_binary_file() {
        let viewer = BinaryViewer::new();
        
        // Create a test binary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0xFF]).unwrap(); // "Hello" + null + 0xFF
        
        let options = ViewerOptions::default();
        let result = viewer.view_file(temp_file.path(), options).await.unwrap();
        
        match result {
            ViewerContent::Binary { data, offset, total_size, .. } => {
                assert_eq!(offset, 0);
                assert_eq!(total_size, 7);
                assert_eq!(data, vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0xFF]);
            }
            _ => panic!("Expected binary content"),
        }
    }

    #[tokio::test]
    async fn test_binary_search() {
        let viewer = BinaryViewer::new();
        
        // Create a test binary file with pattern
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[0x41, 0x42, 0x43, 0x44, 0x41, 0x42, 0x43, 0x44]).unwrap();
        
        let options = SearchOptions {
            case_sensitive: true,
            regex: false,
            max_results: 100,
        };
        
        // Search for hex pattern "41 42"
        let results = viewer.search(temp_file.path(), "41 42", options).await.unwrap();
        assert_eq!(results.len(), 2); // Should find pattern at positions 0 and 4
        
        assert_eq!(results[0].offset, 0);
        assert_eq!(results[1].offset, 4);
    }

    #[tokio::test]
    async fn test_pattern_search() {
        let viewer = BinaryViewer::new();
        let data = vec![0x41, 0x42, 0x43, 0x44, 0x41, 0x42, 0x43, 0x44];
        let pattern = vec![0x41, 0x42];
        
        let matches = viewer.search_binary_pattern(&data, &pattern);
        assert_eq!(matches, vec![0, 4]);
        
        let pattern = vec![0x43, 0x44];
        let matches = viewer.search_binary_pattern(&data, &pattern);
        assert_eq!(matches, vec![2, 6]);
    }
}