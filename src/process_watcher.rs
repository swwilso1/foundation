//! The process watcher module provides a way to watch processes for termination.
//! The module provides `ProcessWatcher` which will monitor a set of process for termination and
//! call a callback when the process terminates.

use crate::error::FoundationError;
use crate::process::watch_processes_for_termination;
use crate::threadcontroller::ThreadController;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::Builder;

/// Type for a process ID.
pub type ProcessId = i32;

/// Type for a callback that is called when a process terminates.
pub type Callback = Box<dyn FnMut(i32) + Send + Sync + 'static>;

/// A process watcher that can be used to watch processes for termination.
pub struct ProcessWatcher {
    /// The callbacks that are called when a process terminates.
    callbacks: Arc<Mutex<HashMap<ProcessId, Callback>>>,

    /// The thread controller that controls the thread that watches the processes.
    thread_controller: Arc<ThreadController>,

    /// The handle to the thread that watches the processes.
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl ProcessWatcher {
    /// Create a new process watcher.
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            thread_controller: Arc::new(ThreadController::new(true)),
            thread_handle: None,
        }
    }

    /// Start the process watcher.
    pub fn start(&mut self) -> Result<(), FoundationError> {
        let thread_controller = self.thread_controller.clone();
        let callbacks = self.callbacks.clone();

        // Start the thread that monitors the processes.
        self.thread_handle = Some(Builder::new().name("ProcessWatcher[]".to_string()).spawn(
            move || {
                // We loop and wait until the thread controller signals that we should stop.
                while !thread_controller.should_stop() {
                    // Get the keys of the callbacks.
                    let mut keys: Vec<ProcessId> = Vec::new();
                    for (key, _) in callbacks.lock().unwrap().iter() {
                        keys.push(*key);
                    }

                    // Call the platform-specific code that watches the processes.
                    if let Ok(dead_processes) = watch_processes_for_termination(keys) {
                        // Call the callbacks for the dead processes.
                        for process_id in dead_processes {
                            if let Some(callback) = callbacks.lock().unwrap().get_mut(&process_id) {
                                callback(process_id);
                            }
                        }
                    }

                    // Wait a bit here so that we do not suck a huge amount of CPU. This is polling and
                    // not terribly efficient, but some platforms do not have an easy mechanism for
                    // waiting on process termination.
                    thread_controller.wait_timeout(std::time::Duration::from_millis(100));
                }
            },
        )?);

        Ok(())
    }

    /// Stop the process watcher.
    pub fn stop(&mut self) -> Result<(), FoundationError> {
        self.thread_controller.signal_stop();
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                let error_msg = format!("Error joining thread: {:?}", e);
                return Err(FoundationError::JoinError(error_msg));
            }
        }
        Ok(())
    }

    /// Add a callback to the process watcher.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The process ID to watch.
    /// * `callback` - The callback to call when the process terminates.
    ///
    /// # Example
    ///
    /// ```rust
    /// use foundation::process_watcher::{ProcessWatcher, ProcessId};
    /// let mut watcher = ProcessWatcher::new();
    /// watcher.start();
    /// watcher.add_callback(1234, Box::new(|pid| {
    ///    println!("Process {} terminated", pid);
    /// }));
    /// watcher.stop().unwrap();
    /// ```
    pub fn add_callback(&mut self, process_id: ProcessId, callback: Callback) {
        self.callbacks.lock().unwrap().insert(process_id, callback);
    }

    /// Remove a callback from the process watcher.
    pub fn remove_callback(&mut self, process_id: ProcessId) {
        self.callbacks.lock().unwrap().remove(&process_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_watcher() {
        let mut watcher = ProcessWatcher::new();
        watcher.start().unwrap();
        watcher.stop().unwrap();
    }

    #[test]
    fn test_already_dead_process() {
        let mut watcher = ProcessWatcher::new();
        let is_dead = Arc::new(Mutex::new(false));
        let is_dead_clone = is_dead.clone();

        // This test might fail if process 2147483647 exists. We will adjust the test if that starts
        // happening a lot.
        watcher.add_callback(
            2147483647,
            Box::new(move |pid| {
                if pid == 2147483647 {
                    *is_dead_clone.lock().unwrap() = true;
                }
            }),
        );
        watcher.start().unwrap();

        // The sleep here gives the watcher thread a chance to run.
        std::thread::sleep(std::time::Duration::from_millis(200));

        watcher.stop().unwrap();
        assert!(is_dead.lock().unwrap().clone());
    }
}
