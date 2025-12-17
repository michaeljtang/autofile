use super::Preprocessor;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Preprocessor that converts HEIC/HEIF images to PNG format
pub struct HeicConverter;

impl HeicConverter {
    pub fn new() -> Self {
        Self
    }

    /// Check if the conversion tools are available
    fn check_tools_available() -> bool {
        // Check for sips (macOS built-in image tool)
        #[cfg(target_os = "macos")]
        {
            Command::new("sips")
                .arg("--version")
                .output()
                .is_ok()
        }

        // Check for ImageMagick convert (cross-platform)
        #[cfg(not(target_os = "macos"))]
        {
            Command::new("convert")
                .arg("-version")
                .output()
                .is_ok()
        }
    }

    /// Convert HEIC to PNG using available tools
    fn convert_heic(&self, source: &Path) -> Result<PathBuf> {
        let output_path = source.with_extension("png");

        #[cfg(target_os = "macos")]
        {
            // Use sips on macOS (built-in, no dependencies)
            let status = Command::new("sips")
                .arg("-s")
                .arg("format")
                .arg("png")
                .arg(source)
                .arg("--out")
                .arg(&output_path)
                .status()
                .context("Failed to execute sips command")?;

            if !status.success() {
                anyhow::bail!("sips command failed with status: {}", status);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Use ImageMagick convert on other platforms
            let status = Command::new("convert")
                .arg(source)
                .arg(&output_path)
                .status()
                .context("Failed to execute convert command")?;

            if !status.success() {
                anyhow::bail!("convert command failed with status: {}", status);
            }
        }

        // Delete original HEIC file after successful conversion
        std::fs::remove_file(source)
            .context("Failed to remove original HEIC file")?;

        log::info!("Converted HEIC to PNG: {:?} -> {:?}", source, output_path);

        Ok(output_path)
    }
}

impl Preprocessor for HeicConverter {
    fn name(&self) -> &str {
        "HEIC to PNG Converter"
    }

    fn should_process(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_str().unwrap_or("").to_lowercase();
            (ext_str == "heic" || ext_str == "heif") && Self::check_tools_available()
        } else {
            false
        }
    }

    fn process(&self, path: &Path) -> Result<PathBuf> {
        self.convert_heic(path)
    }
}
