use anyhow::Result;
use fastembed::TextEmbedding;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Minimum cosine similarity threshold for a match (0.0 to 1.0)
const SIMILARITY_THRESHOLD: f32 = 0.7;

pub struct SubfolderMatcher {
    model: Arc<Mutex<TextEmbedding>>,
    excluded_folders: HashSet<String>,
}

impl SubfolderMatcher {
    pub fn new(excluded_folders: Vec<String>) -> Result<Self> {
        // Initialize the embedding model (using a small, fast model)
        let model = TextEmbedding::try_new(
            Default::default()
        )?;

        let excluded_set: HashSet<String> = excluded_folders.into_iter().collect();

        if !excluded_set.is_empty() {
            log::info!("Excluding folders from matching: {:?}", excluded_set);
        }

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            excluded_folders: excluded_set,
        })
    }

    /// Finds a matching subfolder in the destination directory based on semantic similarity
    /// Returns the matched subfolder path, or the original destination if no match found
    /// Uses a greedy approach: at each depth, finds the best match and recurses only into that folder
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

        if !destination_dir.exists() {
            return Ok(destination_dir.to_path_buf());
        }

        // Generate embeddings for the file stem once
        let file_embedding = {
            let mut model = self.model.lock().unwrap();
            let embeddings = model.embed(vec![file_stem.to_string()], None)?;
            embeddings.into_iter().next().unwrap()
        };

        // Start greedy recursive search from the destination directory
        let final_path = self.find_best_match_greedy(
            destination_dir,
            &file_embedding,
            file_stem,
            0,
        )?;

        if final_path != destination_dir {
            log::info!(
                "Matched '{}' to folder '{}' at depth {}",
                file_stem,
                final_path.file_name().unwrap().to_str().unwrap(),
                final_path.strip_prefix(destination_dir)
                    .map(|p| p.components().count())
                    .unwrap_or(0)
            );
        } else {
            log::info!("No semantic match found for '{}', using top-level destination", file_stem);
        }

        Ok(final_path)
    }

    /// Greedy recursive search: at each level, find the best matching folder
    /// If a good match is found, recurse into it. Otherwise, return current directory.
    fn find_best_match_greedy(
        &self,
        current_dir: &Path,
        file_embedding: &[f32],
        file_stem: &str,
        depth: usize,
    ) -> Result<PathBuf> {
        let entries = match fs::read_dir(current_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(current_dir.to_path_buf()),
        };

        let mut folders = Vec::new();
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    if let Some(folder_name) = entry.file_name().to_str() {
                        // Skip hidden folders (those starting with a dot)
                        if folder_name.starts_with('.') {
                            continue;
                        }

                        // Skip excluded folders
                        if self.excluded_folders.contains(folder_name) {
                            log::debug!("Skipping excluded folder: {}", folder_name);
                            continue;
                        }

                        folders.push((entry.path(), folder_name.to_string()));
                    }
                }
            }
        }

        if folders.is_empty() {
            return Ok(current_dir.to_path_buf());
        }

        // Find the best match at this depth level
        let mut best_match: Option<(PathBuf, String, f32)> = None;

        for (folder_path, folder_name) in folders {
            // Calculate similarity for this folder
            let folder_embedding = {
                let mut model = self.model.lock().unwrap();
                let embeddings = model.embed(vec![folder_name.clone()], None)?;
                embeddings.into_iter().next().unwrap()
            };

            let similarity = cosine_similarity(file_embedding, &folder_embedding);

            log::debug!(
                "{}[depth {}] '{}' <-> '{}': similarity = {:.3}",
                "  ".repeat(depth),
                depth,
                file_stem,
                folder_name,
                similarity
            );

            // Track the best match at this level
            if let Some((_, _, best_sim)) = &best_match {
                if similarity > *best_sim {
                    best_match = Some((folder_path, folder_name, similarity));
                }
            } else {
                best_match = Some((folder_path, folder_name, similarity));
            }
        }

        // If we found a match above the threshold, recurse into it
        if let Some((path, name, similarity)) = best_match {
            if similarity >= SIMILARITY_THRESHOLD {
                log::info!(
                    "{}Greedy match at depth {}: '{}' (similarity: {:.3})",
                    "  ".repeat(depth),
                    depth,
                    name,
                    similarity
                );
                // Recurse into the best match to see if there's an even better match deeper
                return self.find_best_match_greedy(&path, file_embedding, file_stem, depth + 1);
            }
        }

        // No match above threshold at this level, return current directory
        Ok(current_dir.to_path_buf())
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
