//! The `threadcontroller` module provides a thread controller object that allows a thread to
//! signal and control another thread.

use log::error;
use std::sync::{Condvar, Mutex};

/// A thread controller that allows the thread to wait for a signal and
/// supports shutting down the thread.
pub struct ThreadController {
    /// The mutex that controls the condition variable.
    mutex: Mutex<bool>,

    /// The condition variable that allows the thread to wait for a signal.
    condition: Condvar,

    /// If true, the signal will be reset after the thread wakes up.
    auto_reset: bool,

    /// If true, the thread should stop.
    stop: Mutex<bool>,
}

impl ThreadController {
    /// Create a new thread controller.
    ///
    /// # Arguments
    ///
    /// * `auto_reset` - If true, the signal will be reset after the thread wakes up.
    pub fn new(auto_reset: bool) -> ThreadController {
        ThreadController {
            mutex: Mutex::new(false),
            condition: Condvar::new(),
            auto_reset,
            stop: Mutex::new(false),
        }
    }

    /// Wait for a signal.
    ///
    /// This function will block the thread until a signal is received.
    pub fn wait(&self) {
        match self.mutex.lock() {
            Ok(mut guard) => {
                while !*guard {
                    guard = self.condition.wait(guard).unwrap();
                }
                if self.auto_reset {
                    *guard = false;
                }
            }
            Err(_) => {
                error!("Thread controller failed to lock mutex");
            }
        }
    }

    /// Wait for a signal with a timeout.
    ///
    /// This function will block the thread until a signal is received or the timeout is reached.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The duration to wait for a signal.
    pub fn wait_timeout(&self, timeout: std::time::Duration) -> bool {
        match self.mutex.lock() {
            Ok(guard) => {
                let mut result = self.condition.wait_timeout(guard, timeout).unwrap();
                if result.1.timed_out() {
                    return false;
                }
                if self.auto_reset {
                    *result.0 = false;
                }
                true
            }
            Err(_) => {
                error!("Thread controller failed to lock mutex");
                false
            }
        }
    }

    /// Signal the thread to wake up.
    pub fn signal(&self) {
        let mut guard = self.mutex.lock().unwrap();
        *guard = true;
        self.condition.notify_all();
    }

    /// Reset the signal.
    pub fn reset(&self) {
        let mut guard = self.mutex.lock().unwrap();
        *guard = false;
    }

    /// Signal the thread to stop.
    pub fn signal_stop(&self) {
        let mut guard = self.mutex.lock().unwrap();
        *guard = true;

        let mut stop_guard = self.stop.lock().unwrap();
        *stop_guard = true;

        self.condition.notify_all();
    }

    /// Check if the thread should stop.
    ///
    /// Returns true if the thread should stop.
    pub fn should_stop(&self) -> bool {
        let stop_guard = self.stop.lock().unwrap();
        *stop_guard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_new() {
        let controller = ThreadController::new(false);
        assert!(!controller.should_stop());
        controller.signal();
        // Reaching this point means `wait` returned without blocking after the signal.
        controller.wait();
        controller.signal_stop();
        assert!(controller.should_stop());
    }

    #[test]
    fn test_wait() {
        let controller = Arc::new(ThreadController::new(false));
        let controller_clone = controller.clone();
        let thing = Arc::new(RwLock::new(0));
        let thing_clone = thing.clone();

        let handle = std::thread::Builder::new()
            .name("threadcontroller-test-wait".to_string())
            .spawn(move || {
                while !controller_clone.should_stop() {
                    controller_clone.wait();
                    *thing_clone.write().unwrap() = 1;
                }
            })
            .unwrap();

        // Let the thread start
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*thing.read().unwrap(), 0);

        // Wake up the thread to change the thing variable.
        controller.signal();
        // Let the other thread run
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*thing.read().unwrap(), 1);

        // Now tell the other thread to terminate
        controller.signal_stop();
        controller.signal();
        // Give the thread a chance to wake up and quit
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(handle.is_finished());
    }

    #[test]
    fn test_wait_timeout() {
        let controller = Arc::new(ThreadController::new(false));
        let controller_clone = controller.clone();
        let thing = Arc::new(RwLock::new(0));
        let thing_clone = thing.clone();

        let handle = std::thread::Builder::new()
            .name("threadcontroller-test-wait-timeout".to_string())
            .spawn(move || {
                while !controller_clone.should_stop() {
                    if controller_clone.wait_timeout(std::time::Duration::from_secs(100)) {
                        *thing_clone.write().unwrap() = 1;
                    }
                }
            })
            .unwrap();

        // Let the thread start
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*thing.read().unwrap(), 0);

        // Wake up the thread to change the thing variable.
        controller.signal();
        // Let the other thread run
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*thing.read().unwrap(), 1);

        // Now tell the other thread to terminate
        controller.signal_stop();
        controller.signal();
        // Give the thread a chance to wake up and quit
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(handle.is_finished());
    }

    #[test]
    fn test_wait_timeout_times_out() {
        // With no signal, wait_timeout should block until the timeout expires and
        // then return false.
        let controller = ThreadController::new(false);
        let start = std::time::Instant::now();
        let result = controller.wait_timeout(std::time::Duration::from_millis(100));
        assert!(!result);
        assert!(start.elapsed() >= std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_auto_reset_wait() {
        // With auto_reset enabled, each signal should wake the thread exactly once;
        // the signal is cleared after the thread wakes, so it blocks again until the
        // next signal.
        let controller = Arc::new(ThreadController::new(true));
        let controller_clone = controller.clone();
        let counter = Arc::new(RwLock::new(0));
        let counter_clone = counter.clone();

        let handle = std::thread::Builder::new()
            .name("threadcontroller-test-auto-reset-wait".to_string())
            .spawn(move || {
                while !controller_clone.should_stop() {
                    controller_clone.wait();
                    if controller_clone.should_stop() {
                        break;
                    }
                    *counter_clone.write().unwrap() += 1;
                }
            })
            .unwrap();

        // Let the thread start and block in wait().
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*counter.read().unwrap(), 0);

        // One signal should result in exactly one increment because auto_reset
        // clears the signal after the thread wakes.
        controller.signal();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*counter.read().unwrap(), 1);

        // A second signal should produce a second increment.
        controller.signal();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*counter.read().unwrap(), 2);

        // Tell the thread to terminate.
        controller.signal_stop();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(handle.is_finished());
    }

    #[test]
    fn test_auto_reset_wait_timeout() {
        // With auto_reset enabled, wait_timeout should clear the signal when it
        // wakes from a signal rather than a timeout.
        let controller = Arc::new(ThreadController::new(true));
        let controller_clone = controller.clone();
        let thing = Arc::new(RwLock::new(0));
        let thing_clone = thing.clone();

        let handle = std::thread::Builder::new()
            .name("threadcontroller-test-auto-reset-wait-timeout".to_string())
            .spawn(move || {
                while !controller_clone.should_stop() {
                    if controller_clone.wait_timeout(std::time::Duration::from_secs(100)) {
                        *thing_clone.write().unwrap() += 1;
                    }
                }
            })
            .unwrap();

        // Let the thread start.
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*thing.read().unwrap(), 0);

        // Wake the thread; the signal should be auto reset afterwards.
        controller.signal();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(*thing.read().unwrap(), 1);

        // Tell the thread to terminate.
        controller.signal_stop();
        controller.signal();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(handle.is_finished());
    }

    #[test]
    fn test_reset() {
        // After a signal is reset, a thread waiting on the controller should remain
        // blocked until a new signal arrives.
        let controller = Arc::new(ThreadController::new(false));

        // Set and then clear the signal.
        controller.signal();
        controller.reset();

        let controller_clone = controller.clone();
        let done = Arc::new(RwLock::new(false));
        let done_clone = done.clone();

        let handle = std::thread::Builder::new()
            .name("threadcontroller-test-reset".to_string())
            .spawn(move || {
                controller_clone.wait();
                *done_clone.write().unwrap() = true;
            })
            .unwrap();

        // The reset cleared the signal, so the thread should still be blocked.
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!*done.read().unwrap());

        // Signalling now should wake the thread.
        controller.signal();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(*done.read().unwrap());
        assert!(handle.is_finished());
    }
}
