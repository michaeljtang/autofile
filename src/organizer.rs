use crate::categorizer::Categorizer;
use crate::detector::{FileCategory, FileDetector};
use crate::matcher::SubfolderMatcher;
use crate::mover::FileMover;
use anyhow::Result;
use log::{error, info, warn};
use std::path::Path;

pub struct FileOrganizer {
    categorizer: Categorizer,
    matcher: SubfolderMatcher,
}

impl FileOrganizer {
    pub fn new() -> Result<Self> {
        let categorizer = Categorizer::new()?;
        categorizer.ensure_destinations_exist()?;

        info!("Initializing semantic matcher...");
        let matcher = SubfolderMatcher::new()?;
        info!("Semantic matcher initialized");

        Ok(Self {
            categorizer,
            matcher,
        })
    }

    pub fn organize_file(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            warn!("File no longer exists, skipping: {:?}", file_path);
            return Ok(());
        }

        if !file_path.is_file() {
            warn!("Path is not a file, skipping: {:?}", file_path);
            return Ok(());
        }

        info!("Processing file: {:?}", file_path);

        // Detect file category
        let category = match FileDetector::detect_category(file_path) {
            Ok(cat) => cat,
            Err(e) => {
                error!("Failed to detect file category: {}", e);
                return Err(e);
            }
        };

        info!("Detected category: {:?}", category);

        // Skip unknown files
        if category == FileCategory::Unknown {
            warn!("Unknown file type, skipping: {:?}", file_path);
            return Ok(());
        }

        // Get top-level destination from rules
        let top_level_destination = match self.categorizer.get_destination(&category) {
            Some(dest) => dest,
            None => {
                warn!("No rule configured for category {:?}, skipping", category);
                return Ok(());
            }
        };

        // Find matching subfolder within the top-level destination
        let final_destination = self.matcher.find_matching_subfolder(
            file_path,
            top_level_destination,
        )?;

        info!(
            "Destination: {} -> {}",
            top_level_destination.display(),
            final_destination.display()
        );

        // Move the file
        match FileMover::move_file(file_path, &final_destination) {
            Ok(new_path) => {
                info!("Successfully organized file to: {:?}", new_path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to move file: {}", e);
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
