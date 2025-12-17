use anyhow::Result;
use log::{error, info};
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
        info!("Starting file watcher on: {:?}", self.watch_path);

        let tx_clone = tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    for event in events {
                        if let Err(e) = Self::handle_event(&event.event, &tx_clone) {
                            error!("Error handling event: {}", e);
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        error!("Watch error: {:?}", error);
                    }
                }
            },
        )?;

        debouncer
            .watcher()
            .watch(&self.watch_path, RecursiveMode::NonRecursive)?;

        info!("File watcher initialized successfully");

        Ok(debouncer)
    }

    fn handle_event(event: &Event, tx: &Sender<PathBuf>) -> Result<()> {
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in &event.paths {
                    if path.is_file() {
                        info!("New file detected: {:?}", path);

                        // Small delay to ensure file is fully written
                        std::thread::sleep(Duration::from_millis(500));

                        if let Err(e) = tx.send(path.clone()) {
                            error!("Failed to send file path: {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
