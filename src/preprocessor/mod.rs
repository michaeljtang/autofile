mod heic_converter;
mod image_renamer;

use anyhow::Result;
use std::path::{Path, PathBuf};

pub use heic_converter::HeicConverter;
pub use image_renamer::ImageRenamer;

/// Trait for file preprocessors that transform files before organization
pub trait Preprocessor: Send + Sync {
    /// Returns the name of this preprocessor
    fn name(&self) -> &str;

    /// Check if this preprocessor should handle the given file
    fn should_process(&self, path: &Path) -> bool;

    /// Process the file and return the new path (or original if unchanged)
    /// The original file may be deleted/replaced depending on the preprocessor
    fn process(&self, path: &Path) -> Result<PathBuf>;
}

/// Manages multiple preprocessors and applies them in order
pub struct PreprocessorPipeline {
    preprocessors: Vec<Box<dyn Preprocessor>>,
}

impl PreprocessorPipeline {
    /// Create a new preprocessing pipeline with default preprocessors
    pub fn new() -> Self {
        let mut preprocessors: Vec<Box<dyn Preprocessor>> = Vec::new();

        // Add default preprocessors here
        // Order matters: preprocessors run in the order they are added

        // 1. Image renaming (before format conversion)
        preprocessors.push(Box::new(ImageRenamer::new()));

        // 2. Format conversion (HEIC to PNG, etc.)
        preprocessors.push(Box::new(HeicConverter::new()));

        log::info!(
            "Initialized preprocessing pipeline with {} preprocessor(s)",
            preprocessors.len()
        );
        for preprocessor in &preprocessors {
            log::info!("  - {}", preprocessor.name());
        }

        Self { preprocessors }
    }

    /// Process a file through all applicable preprocessors
    /// Returns the final path after all preprocessing
    pub fn process(&self, path: &Path) -> Result<PathBuf> {
        let mut current_path = path.to_path_buf();

        for preprocessor in &self.preprocessors {
            if preprocessor.should_process(&current_path) {
                log::info!(
                    "Applying preprocessor '{}' to {:?}",
                    preprocessor.name(),
                    current_path
                );
                current_path = preprocessor.process(&current_path)?;
                log::info!("Preprocessor result: {:?}", current_path);
            }
        }

        Ok(current_path)
    }
}

impl Default for PreprocessorPipeline {
    fn default() -> Self {
        Self::new()
    }
}
