use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileMover;

impl FileMover {
    pub fn move_file(source: &Path, destination_dir: &Path) -> Result<PathBuf> {
        if !source.exists() {
            anyhow::bail!("Source file does not exist: {:?}", source);
        }

        if !destination_dir.exists() {
            fs::create_dir_all(destination_dir).context(format!(
                "Failed to create destination directory: {:?}",
                destination_dir
            ))?;
        }

        let file_name = source
            .file_name()
            .context("Could not extract file name")?;

        let mut destination = destination_dir.join(file_name);

        // Handle file name conflicts
        destination = Self::resolve_conflict(&destination)?;

        log::info!("Moving {:?} -> {:?}", source, destination);

        // Attempt to move the file
        match fs::rename(source, &destination) {
            Ok(_) => {
                log::info!("Successfully moved file to {:?}", destination);
                Ok(destination)
            }
            Err(e) => {
                // If rename fails (e.g., across filesystems), try copy + delete
                log::warn!("Rename failed, attempting copy + delete: {}", e);
                fs::copy(source, &destination).context("Failed to copy file")?;
                fs::remove_file(source).context("Failed to remove source file after copy")?;
                log::info!("Successfully copied and removed file to {:?}", destination);
                Ok(destination)
            }
        }
    }

    fn resolve_conflict(path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            return Ok(path.to_path_buf());
        }

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Could not extract file stem")?;

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let parent = path.parent().context("Could not get parent directory")?;

        // Try numbered suffixes until we find an available name
        for i in 1..10000 {
            let new_name = if extension.is_empty() {
                format!("{}_{}", file_stem, i)
            } else {
                format!("{}_{}.{}", file_stem, i, extension)
            };

            let new_path = parent.join(new_name);
            if !new_path.exists() {
                log::warn!(
                    "File conflict detected, using new name: {:?}",
                    new_path.file_name()
                );
                return Ok(new_path);
            }
        }

        anyhow::bail!("Could not resolve file name conflict after 10000 attempts");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_move_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source = temp_dir.path().join("test.txt");
        let dest_dir = temp_dir.path().join("destination");

        File::create(&source)?;
        fs::write(&source, b"test content")?;

        let result = FileMover::move_file(&source, &dest_dir)?;

        assert!(result.exists());
        assert!(!source.exists());
        assert_eq!(fs::read_to_string(&result)?, "test content");

        Ok(())
    }

    #[test]
    fn test_conflict_resolution() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let path1 = temp_dir.path().join("test.txt");
        File::create(&path1)?;

        let resolved = FileMover::resolve_conflict(&path1)?;
        assert_eq!(resolved, temp_dir.path().join("test (1).txt"));

        Ok(())
    }
}
