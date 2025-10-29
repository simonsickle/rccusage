use crate::data_loader::get_claude_data_dirs;
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Live monitoring mode for watching file changes
pub struct LiveMonitor {
    dirs: Vec<PathBuf>,
}

impl LiveMonitor {
    /// Create a new live monitor for Claude data directories
    pub fn new() -> Self {
        Self {
            dirs: get_claude_data_dirs(),
        }
    }

    /// Start watching for file changes and run callback on changes
    pub fn watch<F>(&self, mut on_change: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        // Create a channel to receive file system events
        let (tx, rx) = mpsc::channel();

        // Create a file watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Only care about modifications and creations to JSONL files
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for path in &event.paths {
                            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                                let _ = tx.send(());
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        })
        .context("Failed to create file watcher")?;

        // Watch all Claude data directories
        for dir in &self.dirs {
            watcher
                .watch(dir, RecursiveMode::Recursive)
                .with_context(|| format!("Failed to watch directory: {}", dir.display()))?;
        }

        println!("Watching for changes in Claude data directories...");
        println!("Press Ctrl+C to stop.");

        // Initial run
        on_change()?;

        // Watch for changes
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) => {
                    // Debounce multiple events
                    std::thread::sleep(Duration::from_millis(100));

                    // Drain any additional events
                    while rx.try_recv().is_ok() {}

                    // Clear screen and run callback
                    print!("\x1B[2J\x1B[1;1H"); // Clear screen
                    on_change()?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // No events, continue watching
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        Ok(())
    }
}
