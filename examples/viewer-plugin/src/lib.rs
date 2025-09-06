//! Markdown Viewer Plugin for Nimbus
//! 
//! This plugin provides advanced Markdown viewing and editing capabilities:
//! - Syntax highlighting for code blocks
//! - Live HTML preview generation
//! - Table of contents extraction
//! - Search within content
//! - Export to HTML
//! - Split-view editing (source + preview)

use async_trait::async_trait;
use nimbus_plugin_sdk::{
    ViewerPlugin, PluginInfo, Result, PluginError,
    viewer::{ViewerContent, ViewerCapabilities, ViewerOptions, ViewerAction, SearchMatch}
};
use std::path::Path;
use std::collections::HashMap;
use log::{debug, info, warn, error};
use pulldown_cmark::{Parser, Options, html};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::html::{highlighted_html_for_string, IncludeBackground};
use regex::Regex;

pub struct MarkdownViewerPlugin {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    initialized: bool,
}

impl MarkdownViewerPlugin {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            initialized: false,
        }
    }

    /// Convert markdown to HTML with syntax highlighting
    fn markdown_to_html(&self, markdown: &str, options: &ViewerOptions) -> Result<String> {
        debug!("Converting {} bytes of markdown to HTML", markdown.len());
        
        // Configure pulldown-cmark options
        let mut md_options = Options::empty();
        md_options.insert(Options::ENABLE_STRIKETHROUGH);
        md_options.insert(Options::ENABLE_TABLES);
        md_options.insert(Options::ENABLE_FOOTNOTES);
        md_options.insert(Options::ENABLE_TASKLISTS);
        md_options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(markdown, md_options);
        
        // Convert to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        // Apply syntax highlighting to code blocks
        let highlighted_html = self.highlight_code_blocks(&html_output, options)?;
        
        // Wrap in a complete HTML document
        let theme_name = options.theme.as_deref().unwrap_or("base16-ocean.dark");
        let full_html = self.wrap_in_html_document(&highlighted_html, theme_name);
        
        debug!("Generated {} bytes of HTML", full_html.len());
        Ok(full_html)
    }

    /// Apply syntax highlighting to code blocks in HTML
    fn highlight_code_blocks(&self, html: &str, options: &ViewerOptions) -> Result<String> {
        if !options.syntax_highlighting {
            return Ok(html.to_string());
        }
        
        debug!("Applying syntax highlighting to code blocks");
        
        // Regex to find code blocks with language specification
        let code_block_re = Regex::new(r#"<pre><code class="language-(\w+)">([\s\S]*?)</code></pre>"#)
            .map_err(|e| PluginError::execution_error(
                self.info().name,
                format!("Failed to create regex: {}", e)
            ))?;
        
        let theme_name = options.theme.as_deref().unwrap_or("base16-ocean.dark");
        let theme = &self.theme_set.themes[theme_name];
        
        let result = code_block_re.replace_all(html, |caps: &regex::Captures| {
            let language = &caps[1];
            let code = html_escape::decode_html_entities(&caps[2]);
            
            // Find syntax definition for the language
            if let Some(syntax) = self.syntax_set.find_syntax_by_token(language) {
                match highlighted_html_for_string(&code, &self.syntax_set, syntax, theme) {
                    Ok(highlighted) => {
                        format!("<div class=\"syntax-highlighted\">{}</div>", highlighted)
                    }
                    Err(_) => {
                        // Fallback to plain text
                        format!("<pre><code class=\"language-{}\">{}</code></pre>", language, caps[2])
                    }
                }
            } else {
                // Language not found, return original
                caps[0].to_string()
            }
        });
        
        Ok(result.to_string())
    }

    /// Wrap HTML content in a complete document with styling
    fn wrap_in_html_document(&self, content: &str, theme_name: &str) -> String {
        let css_styles = self.get_markdown_css(theme_name);
        
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Markdown Preview</title>
    <style>{}</style>
</head>
<body>
    <div class="markdown-content">
        {}
    </div>
    <script>
        // Add click-to-copy for code blocks
        document.querySelectorAll('pre code').forEach(block => {{
            block.addEventListener('click', () => {{
                navigator.clipboard.writeText(block.textContent);
                console.log('Code copied to clipboard');
            }});
        }});
    </script>
</body>
</html>"#, css_styles, content)
    }

    /// Get CSS styles for markdown rendering
    fn get_markdown_css(&self, theme_name: &str) -> String {
        let base_styles = r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #fff;
        }
        
        .markdown-content {
            font-size: 16px;
        }
        
        h1, h2, h3, h4, h5, h6 {
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }
        
        h1 { font-size: 2em; border-bottom: 1px solid #eaecef; padding-bottom: 10px; }
        h2 { font-size: 1.5em; border-bottom: 1px solid #eaecef; padding-bottom: 8px; }
        h3 { font-size: 1.25em; }
        
        p {
            margin-bottom: 16px;
        }
        
        code {
            background-color: rgba(27,31,35,0.05);
            border-radius: 3px;
            font-size: 85%;
            margin: 0;
            padding: 0.2em 0.4em;
            font-family: 'SFMono-Regular', 'Monaco', 'Inconsolata', 'Liberation Mono', 'Menlo', monospace;
        }
        
        pre {
            background-color: #f6f8fa;
            border-radius: 6px;
            font-size: 85%;
            line-height: 1.45;
            overflow: auto;
            padding: 16px;
            margin-bottom: 16px;
        }
        
        pre code {
            background-color: transparent;
            border: 0;
            font-size: 100%;
            margin: 0;
            padding: 0;
            word-wrap: normal;
            cursor: pointer;
        }
        
        pre code:hover {
            background-color: rgba(0,0,0,0.05);
        }
        
        blockquote {
            border-left: 4px solid #dfe2e5;
            margin: 0;
            padding: 0 16px;
            color: #6a737d;
        }
        
        table {
            border-collapse: collapse;
            margin-bottom: 16px;
            width: 100%;
        }
        
        table th, table td {
            border: 1px solid #dfe2e5;
            padding: 6px 13px;
        }
        
        table th {
            background-color: #f6f8fa;
            font-weight: 600;
        }
        
        ul, ol {
            margin-bottom: 16px;
            padding-left: 30px;
        }
        
        li {
            margin-bottom: 4px;
        }
        
        input[type="checkbox"] {
            margin-right: 8px;
        }
        
        .syntax-highlighted {
            border-radius: 6px;
            margin-bottom: 16px;
        }
        
        a {
            color: #0366d6;
            text-decoration: none;
        }
        
        a:hover {
            text-decoration: underline;
        }
        
        hr {
            border: none;
            border-top: 1px solid #eaecef;
            margin: 24px 0;
        }
        "#;

        // Add theme-specific styles
        let theme_styles = if theme_name.contains("dark") {
            r#"
            body {
                background-color: #0d1117;
                color: #c9d1d9;
            }
            
            h1, h2 {
                border-bottom-color: #21262d;
            }
            
            code {
                background-color: rgba(110,118,129,0.4);
                color: #e6edf3;
            }
            
            pre {
                background-color: #161b22;
            }
            
            blockquote {
                border-left-color: #30363d;
                color: #8b949e;
            }
            
            table th, table td {
                border-color: #30363d;
            }
            
            table th {
                background-color: #21262d;
            }
            
            a {
                color: #58a6ff;
            }
            
            hr {
                border-top-color: #21262d;
            }
            "#
        } else {
            ""
        };

        format!("{}{}", base_styles, theme_styles)
    }

    /// Extract table of contents from markdown
    fn extract_toc(&self, markdown: &str) -> Vec<TocItem> {
        debug!("Extracting table of contents");
        
        let header_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
        let mut toc = Vec::new();
        
        for (line_num, line) in markdown.lines().enumerate() {
            if let Some(caps) = header_re.captures(line) {
                let level = caps[1].len();
                let title = caps[2].trim().to_string();
                let id = self.generate_header_id(&title);
                
                toc.push(TocItem {
                    level,
                    title,
                    id,
                    line: line_num + 1,
                });
            }
        }
        
        debug!("Found {} TOC entries", toc.len());
        toc
    }

    /// Generate HTML-safe ID for headers
    fn generate_header_id(&self, title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    /// Search for text within markdown content
    fn search_markdown(&self, content: &str, query: &str, case_sensitive: bool) -> Vec<SearchMatch> {
        debug!("Searching for '{}' in markdown content", query);
        
        let search_query = if case_sensitive { query.to_string() } else { query.to_lowercase() };
        let search_content = if case_sensitive { content.to_string() } else { content.to_lowercase() };
        
        let mut matches = Vec::new();
        
        for (line_idx, line) in search_content.lines().enumerate() {
            if let Some(pos) = line.find(&search_query) {
                let original_line = content.lines().nth(line_idx).unwrap_or("");
                let context_start = pos.saturating_sub(20);
                let context_end = (pos + search_query.len() + 20).min(line.len());
                let context = original_line.get(context_start..context_end).unwrap_or(original_line);
                
                matches.push(SearchMatch {
                    line: line_idx,
                    column: pos,
                    length: query.len(),
                    context: context.to_string(),
                    highlight: query.to_string(),
                });
            }
        }
        
        debug!("Found {} matches", matches.len());
        matches
    }

    /// Generate preview thumbnail for markdown file
    fn generate_preview(&self, content: &str) -> Result<Vec<u8>> {
        debug!("Generating preview thumbnail");
        
        // Extract first few lines for preview
        let preview_text = content
            .lines()
            .take(10)
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .take(3)
            .collect::<Vec<_>>()
            .join("\n");
        
        // For a real implementation, you would render this as an image
        // For now, we'll return the text as UTF-8 bytes as a placeholder
        Ok(preview_text.into_bytes())
    }
}

#[derive(Debug, Clone)]
struct TocItem {
    level: usize,
    title: String,
    id: String,
    line: usize,
}

#[async_trait]
impl ViewerPlugin for MarkdownViewerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Markdown Viewer Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Advanced Markdown viewer with syntax highlighting and live preview".to_string(),
            author: "Nimbus Team".to_string(),
            homepage: Some("https://github.com/nimbus-file-manager/plugins/markdown-viewer".to_string()),
            repository: Some("https://github.com/nimbus-file-manager/plugins".to_string()),
            license: Some("MIT".to_string()),
            tags: vec![
                "markdown".to_string(),
                "viewer".to_string(),
                "editor".to_string(),
                "syntax".to_string(),
                "preview".to_string(),
                "html".to_string(),
                "export".to_string(),
            ],
            min_version: "0.1.0".to_string(),
            max_version: None,
        }
    }
    
    fn supported_extensions(&self) -> Vec<String> {
        vec![
            "md".to_string(),
            "markdown".to_string(),
            "mdown".to_string(),
            "mkdn".to_string(),
            "mkd".to_string(),
            "mdwn".to_string(),
            "mdtxt".to_string(),
        ]
    }
    
    fn supported_mime_types(&self) -> Vec<String> {
        vec![
            "text/markdown".to_string(),
            "text/x-markdown".to_string(),
            "application/markdown".to_string(),
        ]
    }
    
    fn capabilities(&self) -> ViewerCapabilities {
        ViewerCapabilities {
            can_view: true,
            can_edit: true,
            can_save: true,
            can_search: true,
            can_print: true,
            can_copy: true,
            can_zoom: false, // HTML scaling handled by browser
            can_fullscreen: true,
            max_file_size: Some(50 * 1024 * 1024), // 50MB
            preferred_size: Some((800, 600)),
        }
    }
    
    fn get_actions(&self) -> Vec<ViewerAction> {
        vec![
            ViewerAction {
                id: "export_html".to_string(),
                name: "Export as HTML".to_string(),
                description: Some("Export markdown as standalone HTML file".to_string()),
                shortcut: Some("Ctrl+E".to_string()),
                icon: Some("💾".to_string()),
                enabled: true,
            },
            ViewerAction {
                id: "toggle_preview".to_string(),
                name: "Toggle Preview".to_string(),
                description: Some("Switch between source and preview modes".to_string()),
                shortcut: Some("Ctrl+P".to_string()),
                icon: Some("👁️".to_string()),
                enabled: true,
            },
            ViewerAction {
                id: "show_toc".to_string(),
                name: "Table of Contents".to_string(),
                description: Some("Show document table of contents".to_string()),
                shortcut: Some("Ctrl+T".to_string()),
                icon: Some("📋".to_string()),
                enabled: true,
            },
            ViewerAction {
                id: "word_wrap".to_string(),
                name: "Toggle Word Wrap".to_string(),
                description: Some("Toggle word wrapping for long lines".to_string()),
                shortcut: Some("Alt+W".to_string()),
                icon: Some("📝".to_string()),
                enabled: true,
            },
        ]
    }
    
    async fn view_file(
        &self,
        file_path: &Path,
        options: &ViewerOptions,
    ) -> Result<ViewerContent> {
        debug!("Viewing markdown file: {:?}", file_path);
        
        let markdown_content = tokio::fs::read_to_string(file_path).await
            .map_err(|e| PluginError::execution_error(
                self.info().name,
                format!("Failed to read file: {}", e)
            ))?;
        
        // Check file size limit
        if let Some(max_size) = self.capabilities().max_file_size {
            if markdown_content.len() > max_size as usize {
                return Ok(ViewerContent::error(
                    format!("File too large: {} bytes (max: {} bytes)", 
                           markdown_content.len(), max_size)
                ));
            }
        }
        
        // Convert to HTML for preview
        let html_content = self.markdown_to_html(&markdown_content, options)?;
        
        // Extract table of contents
        let toc = self.extract_toc(&markdown_content);
        let toc_json = serde_json::to_value(toc)
            .map_err(|e| PluginError::execution_error(
                self.info().name,
                format!("Failed to serialize TOC: {}", e)
            ))?;
        
        // Create HTML content with metadata
        Ok(ViewerContent::Html {
            content: html_content,
            base_url: file_path.parent().and_then(|p| p.to_str()).map(|s| s.to_string()),
            scripts: vec![],
            styles: vec![],
        })
    }
    
    async fn save_file(
        &self,
        file_path: &Path,
        content: &ViewerContent,
        _options: &ViewerOptions,
    ) -> Result<()> {
        debug!("Saving markdown file: {:?}", file_path);
        
        let markdown_content = match content {
            ViewerContent::Text { content, .. } => content,
            _ => return Err(PluginError::execution_error(
                self.info().name,
                "Cannot save non-text content as markdown".to_string()
            )),
        };
        
        tokio::fs::write(file_path, markdown_content).await
            .map_err(|e| PluginError::execution_error(
                self.info().name,
                format!("Failed to save file: {}", e)
            ))?;
        
        info!("Successfully saved markdown file: {:?}", file_path);
        Ok(())
    }
    
    async fn perform_action(
        &self,
        action_id: &str,
        file_path: &Path,
        content: &ViewerContent,
    ) -> Result<ViewerContent> {
        debug!("Performing action '{}' on file: {:?}", action_id, file_path);
        
        match action_id {
            "export_html" => {
                // Export as standalone HTML file
                if let ViewerContent::Html { content, .. } = content {
                    let export_path = file_path.with_extension("html");
                    
                    tokio::fs::write(&export_path, content).await
                        .map_err(|e| PluginError::execution_error(
                            self.info().name,
                            format!("Failed to export HTML: {}", e)
                        ))?;
                    
                    info!("Exported HTML to: {:?}", export_path);
                    Ok(ViewerContent::Custom {
                        content_type: "export_result".to_string(),
                        data: serde_json::json!({
                            "message": format!("Exported to {}", export_path.display()),
                            "path": export_path.to_string_lossy()
                        }),
                        renderer: "notification".to_string(),
                    })
                } else {
                    Err(PluginError::execution_error(
                        self.info().name,
                        "Cannot export non-HTML content".to_string()
                    ))
                }
            }
            
            "toggle_preview" => {
                // Toggle between source and preview
                match content {
                    ViewerContent::Html { .. } => {
                        // Switch to source view
                        let markdown_content = tokio::fs::read_to_string(file_path).await
                            .map_err(|e| PluginError::execution_error(
                                self.info().name,
                                format!("Failed to read file: {}", e)
                            ))?;
                        
                        Ok(ViewerContent::text_with_language(markdown_content, "markdown".to_string()))
                    }
                    ViewerContent::Text { .. } => {
                        // Switch to preview
                        self.view_file(file_path, &ViewerOptions::default()).await
                    }
                    _ => Ok(content.clone()),
                }
            }
            
            "show_toc" => {
                // Extract and show table of contents
                let markdown_content = tokio::fs::read_to_string(file_path).await
                    .map_err(|e| PluginError::execution_error(
                        self.info().name,
                        format!("Failed to read file: {}", e)
                    ))?;
                
                let toc = self.extract_toc(&markdown_content);
                let toc_text = toc.iter()
                    .map(|item| format!("{}- {}", "  ".repeat(item.level - 1), item.title))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                Ok(ViewerContent::Custom {
                    content_type: "table_of_contents".to_string(),
                    data: serde_json::json!({
                        "toc": toc,
                        "text": toc_text
                    }),
                    renderer: "toc_viewer".to_string(),
                })
            }
            
            _ => Err(PluginError::execution_error(
                self.info().name,
                format!("Unknown action: {}", action_id)
            ))
        }
    }
    
    async fn search_content(
        &self,
        content: &ViewerContent,
        query: &str,
        case_sensitive: bool,
    ) -> Result<Vec<SearchMatch>> {
        debug!("Searching markdown content for: '{}'", query);
        
        let text_content = match content {
            ViewerContent::Text { content, .. } => content,
            ViewerContent::Html { .. } => {
                // For HTML content, we'd need to extract text or search the original markdown
                return Ok(vec![]);
            }
            _ => return Ok(vec![]),
        };
        
        let matches = self.search_markdown(text_content, query, case_sensitive);
        Ok(matches)
    }
    
    async fn get_preview(
        &self,
        file_path: &Path,
        size: (u32, u32),
    ) -> Result<Option<Vec<u8>>> {
        debug!("Generating preview for: {:?} (size: {:?})", file_path, size);
        
        let content = tokio::fs::read_to_string(file_path).await
            .map_err(|e| PluginError::execution_error(
                self.info().name,
                format!("Failed to read file: {}", e)
            ))?;
        
        let preview = self.generate_preview(&content)?;
        Ok(Some(preview))
    }
    
    async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        info!("Initializing Markdown Viewer Plugin v{}", self.info().version);
        
        // Verify syntax highlighting is available
        debug!("Loaded {} syntax definitions", self.syntax_set.syntaxes().len());
        debug!("Loaded {} themes", self.theme_set.themes.len());
        
        self.initialized = true;
        info!("Markdown Viewer Plugin initialized successfully");
        Ok(())
    }
    
    async fn cleanup(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        info!("Cleaning up Markdown Viewer Plugin");
        self.initialized = false;
        info!("Markdown Viewer Plugin cleanup completed");
        Ok(())
    }
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn plugin_main() -> *mut dyn ViewerPlugin {
    env_logger::init();
    info!("Creating Markdown Viewer Plugin instance");
    let plugin = MarkdownViewerPlugin::new();
    Box::into_raw(Box::new(plugin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio_test;

    #[tokio_test::tokio::test]
    async fn test_plugin_info() {
        let plugin = MarkdownViewerPlugin::new();
        let info = plugin.info();
        
        assert_eq!(info.name, "Markdown Viewer Plugin");
        assert_eq!(info.version, "1.0.0");
        assert!(info.tags.contains(&"markdown".to_string()));
    }

    #[tokio_test::tokio::test]
    async fn test_supported_extensions() {
        let plugin = MarkdownViewerPlugin::new();
        let extensions = plugin.supported_extensions();
        
        assert!(extensions.contains(&"md".to_string()));
        assert!(extensions.contains(&"markdown".to_string()));
    }

    #[tokio_test::tokio::test]
    async fn test_markdown_to_html_conversion() {
        let plugin = MarkdownViewerPlugin::new();
        let options = ViewerOptions::default();
        
        let markdown = "# Hello World\n\nThis is **bold** text.";
        let html = plugin.markdown_to_html(markdown, &options).unwrap();
        
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[tokio_test::tokio::test]
    async fn test_toc_extraction() {
        let plugin = MarkdownViewerPlugin::new();
        
        let markdown = r#"# Chapter 1
## Section 1.1
### Subsection 1.1.1
# Chapter 2"#;
        
        let toc = plugin.extract_toc(markdown);
        assert_eq!(toc.len(), 4);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].title, "Chapter 1");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[1].title, "Section 1.1");
    }

    #[tokio_test::tokio::test]
    async fn test_search_functionality() {
        let plugin = MarkdownViewerPlugin::new();
        
        let content = "Hello world\nThis is a test\nHello again";
        let matches = plugin.search_markdown(content, "Hello", false);
        
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[1].line, 2);
    }

    #[tokio_test::tokio::test]
    async fn test_view_file() {
        let mut plugin = MarkdownViewerPlugin::new();
        plugin.initialize().await.unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        
        let markdown_content = "# Test\n\nThis is a **test** file.";
        std::fs::write(&test_file, markdown_content).unwrap();
        
        let options = ViewerOptions::default();
        let result = plugin.view_file(&test_file, &options).await.unwrap();
        
        match result {
            ViewerContent::Html { content, .. } => {
                assert!(content.contains("<h1>Test</h1>"));
                assert!(content.contains("<strong>test</strong>"));
            }
            _ => panic!("Expected HTML content"),
        }
    }

    #[tokio_test::tokio::test]
    async fn test_plugin_actions() {
        let plugin = MarkdownViewerPlugin::new();
        let actions = plugin.get_actions();
        
        assert!(!actions.is_empty());
        
        let action_ids: Vec<String> = actions.iter().map(|a| a.id.clone()).collect();
        assert!(action_ids.contains(&"export_html".to_string()));
        assert!(action_ids.contains(&"toggle_preview".to_string()));
        assert!(action_ids.contains(&"show_toc".to_string()));
    }
}