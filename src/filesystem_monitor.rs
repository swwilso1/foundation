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
#[derive(Clone)]
pub struct FileSystemMonitor {
    /// The thread controller for the monitor thread.
    // The file system monitor requires a thread to run the notify crate's poll watcher.
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
    /// # Arguments
    ///
    /// * `timeout` - The timeout in milliseconds for the monitor to poll the system.
    ///
    /// # Returns
    ///
    /// Ok(()) on success and a `FoundationError` if an error occurred.
    pub fn start(&mut self, timeout: u64) -> Result<(), FoundationError> {
        let controller = self.thread_controller.clone();
        let watcher = self.poll_watcher.clone();

        trace!("Starting FileSystemMonitor thread");
        Builder::new()
            .name("filesystem-monitor".to_string())
            .spawn(move || {
                while !controller.should_stop() {
                    watcher.lock().unwrap().poll()?;

                    // Sleep for a short time to avoid busy waiting.
                    controller.wait_timeout(Duration::from_millis(timeout));
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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex as StdMutex;
    use std::thread::sleep;

    /// Create a unique temporary directory for a test and return its path. The caller is
    /// responsible for removing it.
    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fs_monitor_test_{}_{}_{:?}",
            label,
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_event_handler_invokes_callback_on_ok() {
        let received = Arc::new(StdMutex::new(Vec::new()));
        let sink = received.clone();
        let mut handler = MonitorEventHandler::new(Box::new(move |event: Event| {
            sink.lock().unwrap().push(event);
        }));

        let event = Event::new(EventKind::Create(notify::event::CreateKind::Any))
            .add_path(std::path::PathBuf::from("/some/path"));
        handler.handle_event(Ok(event));

        let events = received.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].kind,
            EventKind::Create(notify::event::CreateKind::Any)
        );
        assert_eq!(
            events[0].paths,
            vec![std::path::PathBuf::from("/some/path")]
        );
    }

    #[test]
    fn test_event_handler_skips_callback_on_err() {
        let fired = Arc::new(AtomicBool::new(false));
        let flag = fired.clone();
        let mut handler = MonitorEventHandler::new(Box::new(move |_event: Event| {
            flag.store(true, Ordering::Relaxed);
        }));

        // An error result must not invoke the callback; it should only be logged.
        handler.handle_event(Err(notify::Error::generic("synthetic error")));

        assert!(!fired.load(Ordering::Relaxed));
    }

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
        monitor.start(100).unwrap();
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
        monitor.start(100).unwrap();
        let tmp_file = temp_dir.join("filesystem_monitor_test.txt");
        std::fs::write(&tmp_file, "test").unwrap();
        sleep(Duration::from_secs(1));
        assert!(event_handler_fired.load(std::sync::atomic::Ordering::Relaxed));
        std::fs::remove_file(tmp_file).unwrap();
        monitor.stop();
    }

    #[test]
    fn test_clone_shares_thread_controller() {
        let callback = Box::new(|_event: Event| {});
        let monitor = FileSystemMonitor::new(callback, Config::default()).unwrap();
        let mut clone = monitor.clone();

        // The clone shares the underlying thread controller, so stopping the clone is
        // observable through the original.
        assert!(!monitor.thread_controller.should_stop());
        clone.stop();
        assert!(monitor.thread_controller.should_stop());
    }

    #[test]
    fn test_watch_nonexistent_path_is_tolerated() {
        let callback = Box::new(|_event: Event| {});
        let mut monitor = FileSystemMonitor::new(callback, Config::default()).unwrap();
        let missing = std::env::temp_dir().join(format!(
            "fs_monitor_does_not_exist_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        // The poll watcher accepts a path that does not (yet) exist without error.
        let result = monitor.watch(&missing, RecursiveMode::Recursive);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watch_reports_event_path() {
        let dir = unique_temp_dir("event_path");
        let target = dir.join("watched_file.txt");
        let target_for_cb = target.clone();
        let matched = Arc::new(AtomicBool::new(false));
        let flag = matched.clone();

        let callback = Box::new(move |event: Event| {
            if event.paths.iter().any(|p| p == &target_for_cb) {
                flag.store(true, Ordering::Relaxed);
            }
        });

        // Use an explicit short poll interval so changes are detected promptly.
        let config = Config::default().with_poll_interval(Duration::from_millis(50));
        let mut monitor = FileSystemMonitor::new(callback, config).unwrap();
        monitor.watch(&dir, RecursiveMode::Recursive).unwrap();
        monitor.start(50).unwrap();

        std::fs::write(&target, "hello").unwrap();

        // Poll for the event for up to ~2 seconds to avoid flakiness.
        let mut detected = false;
        for _ in 0..40 {
            if matched.load(Ordering::Relaxed) {
                detected = true;
                break;
            }
            sleep(Duration::from_millis(50));
        }

        monitor.stop();
        std::fs::remove_dir_all(&dir).ok();
        assert!(detected, "expected an event referencing the watched file");
    }

    #[test]
    fn test_watch_nonrecursive_mode() {
        let dir = unique_temp_dir("nonrecursive");
        let count = Arc::new(AtomicUsize::new(0));
        let counter = count.clone();
        let callback = Box::new(move |_event: Event| {
            counter.fetch_add(1, Ordering::Relaxed);
        });

        let config = Config::default().with_poll_interval(Duration::from_millis(50));
        let mut monitor = FileSystemMonitor::new(callback, config).unwrap();
        monitor.watch(&dir, RecursiveMode::NonRecursive).unwrap();
        monitor.start(50).unwrap();

        let file = dir.join("top_level.txt");
        std::fs::write(&file, "data").unwrap();

        let mut saw_event = false;
        for _ in 0..40 {
            if count.load(Ordering::Relaxed) > 0 {
                saw_event = true;
                break;
            }
            sleep(Duration::from_millis(50));
        }

        monitor.stop();
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            saw_event,
            "expected at least one event in non-recursive mode"
        );
    }
}
