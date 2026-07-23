use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

const MAX_TEXT_PREVIEW_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug)]
pub enum PreviewContent {
    Image { width: u32, height: u32 },
    Text { content: String, truncated: bool },
    Pdf,
    Video,
    OfficeDocument,
    MetadataOnly { reason: String },
}

#[derive(Clone, Debug)]
pub struct QuickPreview {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub content: PreviewContent,
}

impl QuickPreview {
    pub fn load(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Quick Look previews files; open the folder to browse it",
            ));
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let content = if is_image_extension(&extension) {
            match image::ImageReader::open(path)
                .and_then(|reader| reader.with_guessed_format())
                .and_then(|reader| reader.into_dimensions().map_err(io::Error::other))
            {
                Ok((width, height)) => PreviewContent::Image { width, height },
                Err(error) => PreviewContent::MetadataOnly {
                    reason: format!("Image preview could not be decoded: {error}"),
                },
            }
        } else if is_text_extension(&extension) || looks_like_text(path)? {
            load_text(path)?
        } else if extension == "pdf" {
            PreviewContent::Pdf
        } else if matches!(
            extension.as_str(),
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "wmv" | "m4v"
        ) {
            PreviewContent::Video
        } else if matches!(
            extension.as_str(),
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
        ) {
            PreviewContent::OfficeDocument
        } else {
            PreviewContent::MetadataOnly {
                reason: "No inline preview provider is available for this file type.".to_string(),
            }
        };
        Ok(Self {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy()
                .into_owned(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            content,
        })
    }
}

fn load_text(path: &Path) -> io::Result<PreviewContent> {
    let mut file = fs::File::open(path)?;
    let mut buffer =
        Vec::with_capacity(MAX_TEXT_PREVIEW_BYTES.min(file.metadata()?.len() as usize));
    file.by_ref()
        .take(MAX_TEXT_PREVIEW_BYTES as u64 + 1)
        .read_to_end(&mut buffer)?;
    let truncated = buffer.len() > MAX_TEXT_PREVIEW_BYTES;
    buffer.truncate(MAX_TEXT_PREVIEW_BYTES);
    let content = String::from_utf8_lossy(&buffer).into_owned();
    Ok(PreviewContent::Text { content, truncated })
}

fn looks_like_text(path: &Path) -> io::Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0_u8; 8 * 1024];
    let read = file.read(&mut buffer)?;
    if read == 0 {
        return Ok(true);
    }
    if buffer[..read].contains(&0) {
        return Ok(false);
    }
    Ok(std::str::from_utf8(&buffer[..read]).is_ok())
}

fn is_image_extension(extension: &str) -> bool {
    matches!(
        extension,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff"
    )
}

fn is_text_extension(extension: &str) -> bool {
    matches!(
        extension,
        "txt"
            | "md"
            | "markdown"
            | "rs"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "py"
            | "java"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "cs"
            | "go"
            | "log"
            | "csv"
            | "ini"
            | "cfg"
            | "ps1"
            | "bat"
            | "cmd"
            | "sh"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_preview_types() {
        assert!(is_image_extension("png"));
        assert!(is_text_extension("md"));
        assert!(!is_text_extension("exe"));
    }
}
