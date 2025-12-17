use anyhow::Result;
use fastembed::TextEmbedding;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Minimum cosine similarity threshold for a match (0.0 to 1.0)
const SIMILARITY_THRESHOLD: f32 = 0.5;

pub struct SubfolderMatcher {
    model: Arc<Mutex<TextEmbedding>>,
}

impl SubfolderMatcher {
    pub fn new() -> Result<Self> {
        // Initialize the embedding model (using a small, fast model)
        let model = TextEmbedding::try_new(
            Default::default()
        )?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    /// Finds a matching subfolder in the destination directory based on semantic similarity
    /// Returns the matched subfolder path, or the original destination if no match found
    pub fn find_matching_subfolder(
        &self,
        file_path: &Path,
        destination_dir: &Path,
    ) -> Result<PathBuf> {
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if file_stem.is_empty() {
            return Ok(destination_dir.to_path_buf());
        }

        // Read all subdirectories in the destination
        if !destination_dir.exists() {
            return Ok(destination_dir.to_path_buf());
        }

        let entries = match fs::read_dir(destination_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(destination_dir.to_path_buf()),
        };

        let mut folders = Vec::new();
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    if let Some(folder_name) = entry.file_name().to_str() {
                        folders.push((entry.path(), folder_name.to_string()));
                    }
                }
            }
        }

        if folders.is_empty() {
            return Ok(destination_dir.to_path_buf());
        }

        // Generate embeddings for the file stem
        let file_embedding = {
            let mut model = self.model.lock().unwrap();
            let embeddings = model.embed(vec![file_stem.to_string()], None)?;
            embeddings.into_iter().next().unwrap()
        };

        // Find the best matching folder (greedy approach)
        let mut best_match: Option<(PathBuf, f32)> = None;

        for (folder_path, folder_name) in folders {
            let folder_embedding = {
                let mut model = self.model.lock().unwrap();
                let embeddings = model.embed(vec![folder_name.clone()], None)?;
                embeddings.into_iter().next().unwrap()
            };

            let similarity = cosine_similarity(&file_embedding, &folder_embedding);

            debug!(
                "  '{}' <-> '{}': similarity = {:.3}",
                file_stem, folder_name, similarity
            );

            if similarity >= SIMILARITY_THRESHOLD {
                if let Some((_, best_sim)) = &best_match {
                    if similarity > *best_sim {
                        info!(
                            "Better match: '{}' (similarity: {:.3})",
                            folder_name, similarity
                        );
                        best_match = Some((folder_path, similarity));
                    }
                } else {
                    info!(
                        "Found match: '{}' (similarity: {:.3})",
                        folder_name, similarity
                    );
                    best_match = Some((folder_path, similarity));
                }
            }
        }

        match best_match {
            Some((path, similarity)) => {
                info!(
                    "Matched '{}' to folder '{}' (similarity: {:.3})",
                    file_stem,
                    path.file_name().unwrap().to_str().unwrap(),
                    similarity
                );
                Ok(path)
            }
            None => {
                info!("No semantic match found for '{}', using top-level destination", file_stem);
                Ok(destination_dir.to_path_buf())
            }
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        0.0
    } else {
        dot_product / (magnitude_a * magnitude_b)
    }
}
