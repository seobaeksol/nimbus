use crate::{FileViewer, ViewerCapabilities, ViewerContent, ViewerOptions, ViewerError, SearchOptions, SearchResult};
use async_trait::async_trait;
use std::path::Path;
use encoding_rs::{Encoding, UTF_8};
use std::io::Read;
use std::fs::File;

/// Text file viewer with encoding detection and syntax highlighting
pub struct TextViewer {
    max_file_size: u64,
}

impl TextViewer {
    /// Create a new text viewer with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB default limit
        }
    }

    /// Create a new text viewer with custom max file size
    pub fn with_max_size(max_size: u64) -> Self {
        Self {
            max_file_size: max_size,
        }
    }

    /// Detect encoding from file content
    fn detect_encoding(&self, bytes: &[u8]) -> &'static Encoding {
        // Check for BOM
        if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
            return UTF_8;
        }
        
        if bytes.len() >= 2 {
            match &bytes[0..2] {
                b"\xFF\xFE" => return encoding_rs::UTF_16LE,
                b"\xFE\xFF" => return encoding_rs::UTF_16BE,
                _ => {}
            }
        }

        // Use chardet-like detection
        encoding_rs::Encoding::for_bom(bytes)
            .map(|(encoding, _)| encoding)
            .unwrap_or_else(|| {
                // Simple heuristics for common encodings
                if bytes.iter().all(|&b| b <= 127) {
                    UTF_8
                } else {
                    // Check if valid UTF-8
                    match std::str::from_utf8(bytes) {
                        Ok(_) => UTF_8,
                        Err(_) => {
                            // Try common encodings
                            for encoding in &[encoding_rs::WINDOWS_1252, encoding_rs::ISO_8859_15] {
                                let (_decoded, _, had_errors) = encoding.decode(bytes);
                                if !had_errors {
                                    return *encoding;
                                }
                            }
                            UTF_8 // Fallback to UTF-8
                        }
                    }
                }
            })
    }

    /// Detect programming language from file extension
    fn detect_language(&self, path: &Path) -> Option<String> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        match extension.as_str() {
            "rs" => Some("rust".to_string()),
            "js" | "mjs" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "jsx" => Some("javascript".to_string()),
            "py" | "pyw" => Some("python".to_string()),
            "java" => Some("java".to_string()),
            "c" => Some("c".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "h" | "hpp" => Some("c".to_string()),
            "cs" => Some("csharp".to_string()),
            "go" => Some("go".to_string()),
            "php" => Some("php".to_string()),
            "rb" => Some("ruby".to_string()),
            "swift" => Some("swift".to_string()),
            "kt" | "kts" => Some("kotlin".to_string()),
            "scala" => Some("scala".to_string()),
            "html" | "htm" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            "scss" | "sass" => Some("scss".to_string()),
            "less" => Some("less".to_string()),
            "xml" | "xsd" | "xsl" => Some("xml".to_string()),
            "json" => Some("json".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "ini" | "cfg" => Some("ini".to_string()),
            "md" | "markdown" => Some("markdown".to_string()),
            "sh" | "bash" | "zsh" => Some("bash".to_string()),
            "ps1" => Some("powershell".to_string()),
            "bat" | "cmd" => Some("batch".to_string()),
            "sql" => Some("sql".to_string()),
            "dockerfile" => Some("dockerfile".to_string()),
            "makefile" => Some("makefile".to_string()),
            "tex" => Some("latex".to_string()),
            "r" => Some("r".to_string()),
            "m" => Some("matlab".to_string()),
            "pl" => Some("perl".to_string()),
            "lua" => Some("lua".to_string()),
            "vim" => Some("vim".to_string()),
            _ => {
                // Check filename patterns
                let filename = path.file_name()?.to_str()?.to_lowercase();
                match filename.as_str() {
                    "makefile" | "gnumakefile" => Some("makefile".to_string()),
                    "dockerfile" => Some("dockerfile".to_string()),
                    "cmakelists.txt" => Some("cmake".to_string()),
                    "cargo.toml" | "cargo.lock" => Some("toml".to_string()),
                    "package.json" | "tsconfig.json" => Some("json".to_string()),
                    _ => None,
                }
            }
        }
    }

    /// Get supported text file extensions
    fn get_supported_extensions(&self) -> Vec<String> {
        vec![
            // Programming languages
            "txt", "text", "log", "md", "markdown", "rst",
            "rs", "js", "ts", "tsx", "jsx", "py", "java", "c", "cpp", "h", "hpp",
            "cs", "go", "php", "rb", "swift", "kt", "scala",
            // Web technologies
            "html", "htm", "css", "scss", "sass", "less", "xml", "xsd", "xsl",
            // Data formats
            "json", "yaml", "yml", "toml", "ini", "cfg", "conf",
            // Scripts
            "sh", "bash", "zsh", "ps1", "bat", "cmd",
            // Database
            "sql",
            // Documentation
            "tex", "rtf",
            // Configuration
            "gitignore", "gitattributes", "dockerignore",
        ].iter().map(|s| s.to_string()).collect()
    }

    /// Count lines in text content
    fn count_lines(&self, content: &str) -> usize {
        if content.is_empty() {
            0
        } else {
            content.lines().count()
        }
    }
}

impl Default for TextViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileViewer for TextViewer {
    fn capabilities(&self) -> ViewerCapabilities {
        ViewerCapabilities {
            name: "Text Viewer".to_string(),
            description: "View and search text files with encoding detection and syntax highlighting".to_string(),
            supported_extensions: self.get_supported_extensions(),
            max_file_size: self.max_file_size,
            supports_search: true,
            supports_editing: false,
        }
    }

    fn can_handle(&self, path: &Path) -> bool {
        // Check by extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();
            if self.get_supported_extensions().contains(&ext_lower) {
                return true;
            }
        }

        // Check by filename
        if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
            let filename_lower = filename.to_lowercase();
            match filename_lower.as_str() {
                "makefile" | "gnumakefile" | "dockerfile" | "cmakelists.txt" => return true,
                _ => {}
            }
        }

        // For unknown extensions, we could check MIME type or file content
        false
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

        // Read file content
        let mut file = File::open(path).map_err(ViewerError::Io)?;
        let mut buffer = Vec::new();

        // If offset and length are specified, seek and read partial content
        if let (Some(offset), Some(length)) = (options.offset, options.length) {
            use std::io::Seek;
            file.seek(std::io::SeekFrom::Start(offset)).map_err(ViewerError::Io)?;
            buffer.resize(length as usize, 0);
            file.read_exact(&mut buffer).map_err(ViewerError::Io)?;
        } else {
            file.read_to_end(&mut buffer).map_err(ViewerError::Io)?;
        }

        // Detect encoding
        let encoding = if let Some(ref encoding_name) = options.encoding {
            Encoding::for_label(encoding_name.as_bytes())
                .unwrap_or(UTF_8)
        } else {
            self.detect_encoding(&buffer)
        };

        // Decode content
        let (content, encoding_used, had_errors) = encoding.decode(&buffer);
        
        if had_errors {
            return Err(ViewerError::EncodingError {
                message: format!("Failed to decode file with {} encoding", encoding_used.name()),
            });
        }

        let content_string = content.to_string();
        let line_count = self.count_lines(&content_string);
        let language = if options.syntax_highlighting {
            self.detect_language(path)
        } else {
            None
        };

        Ok(ViewerContent::Text {
            content: content_string,
            encoding: encoding_used.name().to_string(),
            language,
            line_count,
        })
    }

    async fn search(
        &self,
        path: &Path,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, ViewerError> {
        // First, load the file content
        let viewer_options = ViewerOptions::default();
        let content = match self.view_file(path, viewer_options).await? {
            ViewerContent::Text { content, .. } => content,
            _ => return Err(ViewerError::Other("Unexpected content type".to_string())),
        };

        let mut results = Vec::new();
        let search_query = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        // Process content line by line
        for (line_number, line) in content.lines().enumerate() {
            let search_line = if options.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            let matches: Vec<usize> = if options.regex {
                // Use regex matching
                match regex::Regex::new(&search_query) {
                    Ok(re) => re.find_iter(&search_line).map(|m| m.start()).collect(),
                    Err(_) => {
                        return Err(ViewerError::Other("Invalid regex pattern".to_string()));
                    }
                }
            } else {
                // Simple string matching
                let mut matches = Vec::new();
                let mut start = 0;
                while let Some(pos) = search_line[start..].find(&search_query) {
                    matches.push(start + pos);
                    start = start + pos + search_query.len();
                }
                matches
            };

            // Create search results for each match
            for match_pos in matches {
                let context_start = if match_pos >= 20 { match_pos - 20 } else { 0 };
                let context_end = std::cmp::min(line.len(), match_pos + search_query.len() + 20);
                
                let context_before = line[context_start..match_pos].to_string();
                let matched_text = line[match_pos..match_pos + search_query.len()].to_string();
                let context_after = line[match_pos + search_query.len()..context_end].to_string();

                results.push(SearchResult {
                    line_number: Some(line_number + 1),
                    offset: match_pos as u64,
                    length: search_query.len(),
                    context_before,
                    matched_text,
                    context_after,
                });

                if results.len() >= options.max_results {
                    return Ok(results);
                }
            }

            if results.len() >= options.max_results {
                break;
            }
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
    async fn test_text_viewer_basic() {
        let viewer = TextViewer::new();
        
        // Create a temporary text file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        writeln!(temp_file, "This is a test file.").unwrap();
        
        let options = ViewerOptions::default();
        let result = viewer.view_file(temp_file.path(), options).await.unwrap();
        
        match result {
            ViewerContent::Text { content, encoding, line_count, .. } => {
                assert!(content.contains("Hello, World!"));
                assert!(content.contains("This is a test file."));
                assert_eq!(encoding, "utf-8");
                assert_eq!(line_count, 2);
            }
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_language_detection() {
        let viewer = TextViewer::new();
        
        assert_eq!(viewer.detect_language(Path::new("test.rs")), Some("rust".to_string()));
        assert_eq!(viewer.detect_language(Path::new("test.js")), Some("javascript".to_string()));
        assert_eq!(viewer.detect_language(Path::new("test.py")), Some("python".to_string()));
        assert_eq!(viewer.detect_language(Path::new("Makefile")), Some("makefile".to_string()));
    }

    #[tokio::test]
    async fn test_can_handle() {
        let viewer = TextViewer::new();
        
        assert!(viewer.can_handle(Path::new("test.txt")));
        assert!(viewer.can_handle(Path::new("test.rs")));
        assert!(viewer.can_handle(Path::new("test.js")));
        assert!(viewer.can_handle(Path::new("Makefile")));
        assert!(!viewer.can_handle(Path::new("test.jpg")));
        assert!(!viewer.can_handle(Path::new("test.exe")));
    }

    #[tokio::test]
    async fn test_search() {
        let viewer = TextViewer::new();
        
        // Create a temporary text file with multiple lines
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        writeln!(temp_file, "This is a test file.").unwrap();
        writeln!(temp_file, "Hello again!").unwrap();
        
        let options = SearchOptions {
            case_sensitive: false,
            regex: false,
            max_results: 100,
        };
        
        let results = viewer.search(temp_file.path(), "hello", options).await.unwrap();
        assert_eq!(results.len(), 2);
        
        assert_eq!(results[0].line_number, Some(1));
        assert_eq!(results[1].line_number, Some(3));
    }
}