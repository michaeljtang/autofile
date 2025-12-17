use crate::categorizer::Categorizer;
use crate::detector::{FileCategory, FileDetector};
use crate::mover::FileMover;
use anyhow::Result;
use log::{error, info, warn};
use std::path::Path;

pub struct FileOrganizer {
    categorizer: Categorizer,
}

impl FileOrganizer {
    pub fn new() -> Result<Self> {
        let categorizer = Categorizer::new()?;
        categorizer.ensure_destinations_exist()?;

        Ok(Self { categorizer })
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

        // Get destination from rules
        let destination = match self.categorizer.get_destination(&category) {
            Some(dest) => dest,
            None => {
                warn!("No rule configured for category {:?}, skipping", category);
                return Ok(());
            }
        };

        // Move the file
        match FileMover::move_file(file_path, destination) {
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
