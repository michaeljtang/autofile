use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub matcher: MatcherConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatcherConfig {
    /// Folders to exclude from semantic matching (e.g., "Archive", "Old Files")
    #[serde(default)]
    pub excluded_folders: Vec<String>,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self {
            excluded_folders: vec![],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            matcher: MatcherConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from the default location
    /// If the config file doesn't exist, create it with default values
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            log::info!("Config file not found, creating default config at {:?}", config_path);
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        log::info!("Loaded config from {:?}", config_path);
        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, contents)
            .context("Failed to write config file")?;

        log::info!("Saved config to {:?}", config_path);
        Ok(())
    }

    /// Get the path to the config file
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("autofile").join("config.toml"))
    }
}
