mod categorizer;
mod detector;
mod matcher;
mod mover;
mod organizer;
mod watcher;

use anyhow::{Context, Result};
use log::{error, info};
use organizer::FileOrganizer;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use watcher::FileWatcher;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting AutoFile - Smart File Organizer");

    // Get watch directory from args or use Downloads
    let watch_dir = get_watch_directory().unwrap();

    info!("Monitoring directory: {:?}", watch_dir);

    // Validate watch directory exists
    if !watch_dir.exists() {
        error!("Watch directory doesn't exist");
        std::process::exit(1);
    }

    // Create file organizer
    let organizer = FileOrganizer::new().context("Failed to create file organizer").unwrap();

    // Create channel for file events
    let (tx, rx) = mpsc::channel::<PathBuf>();

    // Spawn organizer thread
    std::thread::spawn(move || {
        for file_path in rx {
            if let Err(e) = organizer.organize_file(&file_path) {
                error!("Error organizing file {:?}: {}", file_path, e);
            }
        }
    });

    // Start file watcher and keep it alive
    let watcher = FileWatcher::new(watch_dir);
    let _debouncer = watcher.start(tx).unwrap();

    // Keep the main thread alive indefinitely
    std::thread::park();
}

fn get_watch_directory() -> Result<PathBuf> {
    // Check for command line argument first
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        if path.exists() && path.is_dir() {
            return Ok(path);
        } else {
            anyhow::bail!("Provided path is not a valid directory: {:?}", path);
        }
    }

    // Default to Downloads directory
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let downloads = home_dir.join("Downloads");

    if downloads.exists() {
        Ok(downloads)
    } else {
        anyhow::bail!(
            "Default Downloads directory not found. Please provide a directory path as an argument."
        )
    }
}
