use crate::detector::FileCategory;
use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub name: String,
    pub destination: PathBuf,
}

pub struct Categorizer {
    rules: HashMap<FileCategory, CategoryRule>,
}

impl Categorizer {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;

        let mut rules = HashMap::new();

        rules.insert(
            FileCategory::Document,
            CategoryRule {
                name: "Documents".to_string(),
                destination: home_dir.join("Documents"),
            },
        );

        rules.insert(
            FileCategory::Image,
            CategoryRule {
                name: "Images".to_string(),
                destination: home_dir.join("Pictures"),
            },
        );

        rules.insert(
            FileCategory::Video,
            CategoryRule {
                name: "Videos".to_string(),
                destination: home_dir.join("Videos"),
            },
        );

        rules.insert(
            FileCategory::Audio,
            CategoryRule {
                name: "Music".to_string(),
                destination: home_dir.join("Music"),
            },
        );

        rules.insert(
            FileCategory::Archive,
            CategoryRule {
                name: "Archives".to_string(),
                destination: home_dir.join("Documents").join("Archives"),
            },
        );

        rules.insert(
            FileCategory::Code,
            CategoryRule {
                name: "Projects".to_string(),
                destination: home_dir.join("Projects"),
            },
        );

        Ok(Self { rules })
    }

    pub fn with_custom_rules(rules: HashMap<FileCategory, CategoryRule>) -> Self {
        Self { rules }
    }

    pub fn get_destination(&self, category: &FileCategory) -> Option<&PathBuf> {
        self.rules.get(category).map(|rule| &rule.destination)
    }

    pub fn get_rule(&self, category: &FileCategory) -> Option<&CategoryRule> {
        self.rules.get(category)
    }

    pub fn ensure_destinations_exist(&self) -> Result<()> {
        for (category, rule) in &self.rules {
            if !rule.destination.exists() {
                info!(
                    "Creating destination directory for {:?}: {:?}",
                    category, rule.destination
                );
                std::fs::create_dir_all(&rule.destination).context(format!(
                    "Failed to create directory: {:?}",
                    rule.destination
                ))?;
            }
        }
        Ok(())
    }
}

impl Default for Categorizer {
    fn default() -> Self {
        Self::new().expect("Failed to create default categorizer")
    }
}
