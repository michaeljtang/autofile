use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Unknown,
}

pub struct FileDetector;

impl FileDetector {
    pub fn detect_category(path: &Path) -> Result<FileCategory> {
        // First try magic bytes detection
        if let Ok(bytes) = fs::read(path) {
            if let Some(kind) = infer::get(&bytes) {
                let mime_type = kind.mime_type();
                let matcher_type = kind.matcher_type();

                let category = match matcher_type {
                    infer::MatcherType::Image => FileCategory::Image,
                    infer::MatcherType::Video => FileCategory::Video,
                    infer::MatcherType::Audio => FileCategory::Audio,
                    infer::MatcherType::Archive => FileCategory::Document,
                    infer::MatcherType::Doc => FileCategory::Document,
                    infer::MatcherType::Font => FileCategory::Document,
                    _ => Self::detect_by_extension(path)
                };

                log::info!(
                    "MIME {} | Categorized as: {:?}",
                    mime_type,
                    category
                );
                return Ok(category);
            }
        }

        // Fallback to extension-based detection
        log::warn!("Could not detect file type by magic bytes, falling back to extension");
        Ok(Self::detect_by_extension(path))
    }

    fn detect_by_extension(path: &Path) -> FileCategory {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            // Documents
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" => {
                FileCategory::Document
            }

            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif" => {
                FileCategory::Image
            }

            // Videos
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" => {
                FileCategory::Video
            }

            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => {
                FileCategory::Audio
            }

            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "tgz" => FileCategory::Archive,

            // Code files
            "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "cs"
            | "rb" | "php" | "swift" | "kt" | "scala" | "r" | "m" | "sh" | "bash" | "zsh"
            | "fish" | "html" | "css" | "scss" | "sass" | "json" | "xml" | "yaml" | "yml"
            | "toml" | "sql" | "md" | "rst" | "tex" => FileCategory::Code,

            _ => FileCategory::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_detection() {
        use std::path::PathBuf;

        assert_eq!(
            FileDetector::detect_by_extension(&PathBuf::from("test.pdf")),
            FileCategory::Document
        );
        assert_eq!(
            FileDetector::detect_by_extension(&PathBuf::from("image.png")),
            FileCategory::Image
        );
        assert_eq!(
            FileDetector::detect_by_extension(&PathBuf::from("script.rs")),
            FileCategory::Code
        );
    }
}
