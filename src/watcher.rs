use crate::utils;
use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct FileWatcher {
    watch_path: PathBuf,
}

impl FileWatcher {
    pub fn new(watch_path: PathBuf) -> Self {
        Self { watch_path }
    }

    pub fn start(self, tx: Sender<PathBuf>) -> Result<impl Drop> {
        log::info!("Starting file watcher on: {:?}", self.watch_path);

        let tx_clone = tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    for event in events {
                        if let Err(e) = Self::handle_event(&event.event, &tx_clone) {
                            log::error!("Error handling event: {}", e);
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        log::error!("Watch error: {:?}", error);
                    }
                }
            },
        )?;

        debouncer
            .watcher()
            .watch(&self.watch_path, RecursiveMode::NonRecursive)?;

        log::info!("File watcher initialized successfully");

        Ok(debouncer)
    }

    fn handle_event(event: &Event, tx: &Sender<PathBuf>) -> Result<()> {
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in &event.paths {
                    if path.is_file() {
                        // Ignore hidden files.
                        if utils::file::is_hidden_file(path) {
                            log::debug!("Ignoring hidden file: {:?}", path);
                            continue;
                        }

                        log::info!("New file detected: {:?}", path);

                        // Small delay to ensure file is fully written
                        std::thread::sleep(Duration::from_millis(500));

                        if let Err(e) = tx.send(path.clone()) {
                            log::error!("Failed to send file path: {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
