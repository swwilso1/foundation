//! The `filesystem_monitor` module provides a veneer around the `notify` crate to monitor file system
//! objects for changes. The veneer takes care of threading and event handling for the notify crate.

use crate::error::FoundationError;
use crate::threadcontroller::ThreadController;
use log::{error, trace};
use notify::{poll::PollWatcher, EventHandler, Watcher};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::Builder;
use std::time::Duration;

/// Configuration for the file system monitor.
pub type Config = notify::Config;

/// The event object passed to the event handler callback.
pub type Event = notify::Event;

/// The attributes of the event object.
pub type EventAttributes = notify::event::EventAttributes;

/// The kind of event.
pub type EventKind = notify::EventKind;

/// Recursion mode for watching directories.
pub type RecursiveMode = notify::RecursiveMode;

/// Callback function that receives events from the file system monitor.
type EventCallback = dyn FnMut(Event) + Send + Sync;

/// The event handler for the file system monitor.
struct MonitorEventHandler {
    /// The callback function that receives events from the file system monitor.
    callback: Box<EventCallback>,
}

impl MonitorEventHandler {
    /// Create a new `MonitorEventHandler` with the given callback.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function that receives events from the file system monitor.
    pub fn new(callback: Box<EventCallback>) -> MonitorEventHandler {
        MonitorEventHandler { callback }
    }
}

impl EventHandler for MonitorEventHandler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        match event {
            Ok(event) => {
                trace!("FileSystemMonitor Event: {:?}", event);
                (self.callback)(event);
            }
            Err(e) => {
                error!("Error handling event: {}", e);
            }
        }
    }
}

/// The file system monitor object.
pub struct FileSystemMonitor {
    /// The thread controller for the monitor thread.
    thread_controller: Arc<ThreadController>,

    /// The poll watcher for the monitor thread.
    poll_watcher: Arc<Mutex<PollWatcher>>,
}

impl FileSystemMonitor {
    /// Create a new `FileSystemMonitor` with the given callback and configuration.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function that receives events from the file system monitor.
    /// * `config` - The configuration for the file system monitor.
    pub fn new(
        callback: Box<EventCallback>,
        config: Config,
    ) -> Result<FileSystemMonitor, FoundationError> {
        let thread_controller = Arc::new(ThreadController::new(true));
        let event_handler = MonitorEventHandler::new(callback);
        let poll_watcher = Arc::new(Mutex::new(PollWatcher::new(event_handler, config)?));

        Ok(FileSystemMonitor {
            thread_controller,
            poll_watcher,
        })
    }

    /// Start the file system monitor thread.
    ///
    /// # Returns
    ///
    /// Ok(()) on success and a `FoundationError` if an error occurred.
    pub fn start(&mut self) -> Result<(), FoundationError> {
        let controller = self.thread_controller.clone();
        let watcher = self.poll_watcher.clone();

        trace!("Starting FileSystemMonitor thread");
        Builder::new()
            .name("filesystem-monitor".to_string())
            .spawn(move || {
                while !controller.should_stop() {
                    watcher.lock().unwrap().poll()?;

                    // Sleep for a short time to avoid busy waiting.
                    controller.wait_timeout(Duration::from_millis(100));
                }
                Ok::<(), FoundationError>(())
            })?;
        Ok(())
    }

    /// Stop the file system monitor thread.
    pub fn stop(&mut self) {
        trace!("Stopping FileSystemMonitor thread");
        self.thread_controller.signal_stop();
    }

    /// Watch a path for changes.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to watch.
    /// * `recursive_mode` - The recursive mode for watching directories.
    ///
    /// # Returns
    ///
    /// Ok(()) on success and a `FoundationError` if an error occurred.
    pub fn watch(
        &mut self,
        path: &Path,
        recursive_mode: RecursiveMode,
    ) -> Result<(), FoundationError> {
        self.poll_watcher
            .lock()
            .unwrap()
            .watch(path, recursive_mode)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_new() {
        let callback = Box::new(|event: Event| {
            println!("Event: {:?}", event);
        });
        let config = Config::default();
        let monitor = FileSystemMonitor::new(callback, config).unwrap();
        assert!(!monitor.thread_controller.should_stop());
    }

    #[test]
    fn test_start() {
        let callback = Box::new(|event: Event| {
            println!("Event: {:?}", event);
        });
        let config = Config::default();
        let mut monitor = FileSystemMonitor::new(callback, config).unwrap();
        monitor.start().unwrap();
        monitor.stop();
        assert!(monitor.thread_controller.should_stop());
    }

    #[test]
    fn test_watch() {
        let event_handler_fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fired = event_handler_fired.clone();
        let callback = Box::new(move |event: Event| {
            println!("Event: {:?}", event);
            fired.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        let config = Config::default();
        let mut monitor = FileSystemMonitor::new(callback, config).unwrap();
        let temp_dir = std::env::temp_dir();
        monitor.watch(&temp_dir, RecursiveMode::Recursive).unwrap();
        monitor.start().unwrap();
        let tmp_file = temp_dir.join("filesystem_monitor_test.txt");
        std::fs::write(&tmp_file, "test").unwrap();
        sleep(Duration::from_secs(1));
        assert_eq!(
            event_handler_fired.load(std::sync::atomic::Ordering::Relaxed),
            true
        );
        std::fs::remove_file(tmp_file).unwrap();
        monitor.stop();
    }
}
