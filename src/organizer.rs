use crate::categorizer::Categorizer;
use crate::config::Config;
use crate::detector::{FileCategory, FileDetector};
use crate::matcher::SubfolderMatcher;
use crate::mover::FileMover;
use crate::preprocessor::PreprocessorPipeline;
use anyhow::Result;
use std::path::Path;

pub struct FileOrganizer {
    categorizer: Categorizer,
    matcher: SubfolderMatcher,
    preprocessor: PreprocessorPipeline,
}

impl FileOrganizer {
    pub fn new() -> Result<Self> {
        let categorizer = Categorizer::new()?;
        categorizer.ensure_destinations_exist()?;

        // Load configuration
        let config = Config::load()?;

        log::info!("Initializing semantic matcher...");
        let matcher = SubfolderMatcher::new(config.matcher.excluded_folders)?;
        log::info!("Semantic matcher initialized");

        // Initialize preprocessing pipeline
        let preprocessor = PreprocessorPipeline::new();

        Ok(Self {
            categorizer,
            matcher,
            preprocessor,
        })
    }

    pub fn organize_file(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            log::warn!("File no longer exists, skipping: {:?}", file_path);
            return Ok(());
        }

        if !file_path.is_file() {
            log::warn!("Path is not a file, skipping: {:?}", file_path);
            return Ok(());
        }

        log::info!("Processing file: {:?}", file_path);

        // Apply preprocessing (e.g., HEIC to PNG conversion)
        let processed_path = self.preprocessor.process(file_path)?;

        // Detect file category (using processed path)
        let category = match FileDetector::detect_category(&processed_path) {
            Ok(cat) => cat,
            Err(e) => {
                log::error!("Failed to detect file category: {}", e);
                return Err(e);
            }
        };

        log::info!("Detected category: {:?}", category);

        // Skip unknown files
        if category == FileCategory::Unknown {
            log::warn!("Unknown file type, skipping: {:?}", processed_path);
            return Ok(());
        }

        // Get top-level destination from rules
        let top_level_destination = match self.categorizer.get_destination(&category) {
            Some(dest) => dest,
            None => {
                log::warn!("No rule configured for category {:?}, skipping", category);
                return Ok(());
            }
        };

        // Find matching subfolder within the top-level destination
        let final_destination = self.matcher.find_matching_subfolder(
            &processed_path,
            top_level_destination,
        )?;

        log::info!(
            "Destination: {} -> {}",
            top_level_destination.display(),
            final_destination.display()
        );

        // Move the file
        match FileMover::move_file(&processed_path, &final_destination) {
            Ok(new_path) => {
                log::info!("Successfully organized file to: {:?}", new_path);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to move file: {}", e);
                Err(e)
            }
        }
    }
}

impl Default for FileOrganizer {
    fn default() -> Self {
        Self::new().expect("Failed to create default file organizer")
    }
}
