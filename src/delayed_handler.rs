//! The `delayed_handler` module contains code to manage functionality that needs
//! to be delayed until a later time.  The developer writes a handler function that
//! takes data of a particular type and then associates that handler with a key.
//! Later when the program has the needed data, the program can call the handler
//! by scheduling it for execution with the data in a thread pool.
//!

// This module exists to provide a way to schedule functionality for a long-running operation,
// such as communicating with another program or waiting on file access. The program can detail
// the next functionality to perform and then schedule it for execution at a later time. When the
// time comes, the program can execute the functionality in a thread pool, even if the program has
// moved on to some arbitrary number of operations in the meantime.

use crate::error::FoundationError;
use crate::threadpool::{ThreadJob, ThreadPool, WorkerId};
use std::collections::HashMap;
use std::hash::Hash;

/// The handler is a function or closure that takes the data and implements any functionality
/// needed to process the data.
pub type Handler<T> = Box<dyn Fn(T) + Send + Sync + 'static>;

/// The `DelayedHandler` struct is a container for handlers that need to be executed at a later time.
pub struct DelayedHandler<K: Clone + Hash + PartialEq + Eq, T: Send + Sync + 'static> {
    /// A map of keys to handlers.
    handlers: HashMap<K, Handler<T>>,

    /// The thread pool for executing the handlers.
    thread_pool: ThreadPool,
}

impl<K: Clone + Hash + PartialEq + Eq, T: Send + Sync + 'static> DelayedHandler<K, T> {
    /// Create a new `DelayedHandler` instance with the given maximum number of workers.
    pub fn new(max_workers: WorkerId) -> Self {
        DelayedHandler {
            handlers: HashMap::new(),
            thread_pool: ThreadPool::new(max_workers),
        }
    }

    /// Add a handler to the `DelayedHandler` instance with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the handler.
    /// * `handler` - The handler to associate with the key.
    pub fn add_handler(&mut self, key: &K, handler: Handler<T>) {
        self.handlers.insert(key.clone(), handler);
    }

    /// Schedule the handler with the given key and data for execution in the thread pool.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the handler.
    /// * `data` - The data to pass to the handler.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the scheduling operation.
    pub fn schedule_handler(&mut self, key: &K, data: T) -> Result<(), FoundationError> {
        let mut thread_job = ThreadJob::new();
        if let Some(handler) = self.handlers.remove(key) {
            thread_job.add_task(Box::pin(async move {
                handler(data);
                Ok(())
            }));
            self.thread_pool.add_job(thread_job)
        } else {
            Err(FoundationError::HandlerNotFound)
        }
    }

    /// Check if the `DelayedHandler` instance contains a handler for the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for in the `DelayedHandler` instance.
    ///
    /// # Returns
    ///
    /// True if the `DelayedHandler` instance contains a handler for the given key, false otherwise.
    pub fn contains_handler_for_key(&self, key: &K) -> bool {
        self.handlers.contains_key(key)
    }

    /// Remove the handler associated with the given key from the `DelayedHandler` instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove from the `DelayedHandler` instance.
    pub fn remove_handler(&mut self, key: &K) {
        self.handlers.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_delayed_handler() {
        let wrapped_bool = Arc::new(Mutex::new(false));
        let wrapped_bool_c = wrapped_bool.clone();
        let mut delayed_handler: DelayedHandler<String, String> = DelayedHandler::new(1);

        let handler = Box::new(move |data: String| {
            if data == "Hello, world!" {
                let mut wrapped_bool = wrapped_bool_c.lock().unwrap();
                *wrapped_bool = true;
            }
        });

        let key = String::from("test");

        delayed_handler.add_handler(&key, handler);

        let result = delayed_handler.schedule_handler(&key, "Hello, world!".to_string());
        assert!(result.is_ok());

        sleep(Duration::from_secs(1)).await;

        let wrapped_bool = wrapped_bool.lock().unwrap();
        assert!(*wrapped_bool);
    }

    #[tokio::test]
    async fn test_schedule_handler_missing_key_returns_error() {
        let mut delayed_handler: DelayedHandler<String, String> = DelayedHandler::new(1);

        let key = String::from("missing");
        let result = delayed_handler.schedule_handler(&key, "data".to_string());

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FoundationError::HandlerNotFound
        ));
    }

    #[tokio::test]
    async fn test_contains_handler_for_key() {
        let mut delayed_handler: DelayedHandler<String, String> = DelayedHandler::new(1);

        let key = String::from("test");
        assert!(!delayed_handler.contains_handler_for_key(&key));

        delayed_handler.add_handler(&key, Box::new(|_data: String| {}));
        assert!(delayed_handler.contains_handler_for_key(&key));

        let other_key = String::from("other");
        assert!(!delayed_handler.contains_handler_for_key(&other_key));
    }

    #[tokio::test]
    async fn test_remove_handler() {
        let mut delayed_handler: DelayedHandler<String, String> = DelayedHandler::new(1);

        let key = String::from("test");
        delayed_handler.add_handler(&key, Box::new(|_data: String| {}));
        assert!(delayed_handler.contains_handler_for_key(&key));

        delayed_handler.remove_handler(&key);
        assert!(!delayed_handler.contains_handler_for_key(&key));

        // Removing a key that does not exist should be a no-op and not panic.
        delayed_handler.remove_handler(&key);
        assert!(!delayed_handler.contains_handler_for_key(&key));
    }

    #[tokio::test]
    async fn test_schedule_handler_consumes_handler() {
        let mut delayed_handler: DelayedHandler<String, String> = DelayedHandler::new(1);

        let key = String::from("test");
        delayed_handler.add_handler(&key, Box::new(|_data: String| {}));
        assert!(delayed_handler.contains_handler_for_key(&key));

        let result = delayed_handler.schedule_handler(&key, "data".to_string());
        assert!(result.is_ok());

        // After scheduling, the handler is removed and cannot be scheduled again.
        assert!(!delayed_handler.contains_handler_for_key(&key));

        let second = delayed_handler.schedule_handler(&key, "data".to_string());
        assert!(second.is_err());
        assert!(matches!(
            second.unwrap_err(),
            FoundationError::HandlerNotFound
        ));
    }

    #[tokio::test]
    async fn test_add_handler_overwrites_existing() {
        let counter = Arc::new(Mutex::new(0_i32));
        let mut delayed_handler: DelayedHandler<String, String> = DelayedHandler::new(1);

        let key = String::from("test");

        // The first handler adds 1; it should be overwritten and never run.
        let counter_first = counter.clone();
        delayed_handler.add_handler(
            &key,
            Box::new(move |_data: String| {
                *counter_first.lock().unwrap() += 1;
            }),
        );

        // The second handler adds 100 and replaces the first.
        let counter_second = counter.clone();
        delayed_handler.add_handler(
            &key,
            Box::new(move |_data: String| {
                *counter_second.lock().unwrap() += 100;
            }),
        );

        let result = delayed_handler.schedule_handler(&key, "data".to_string());
        assert!(result.is_ok());

        sleep(Duration::from_secs(1)).await;

        assert_eq!(*counter.lock().unwrap(), 100);
    }

    #[tokio::test]
    async fn test_multiple_handlers_with_distinct_keys() {
        let results = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut delayed_handler: DelayedHandler<i32, String> = DelayedHandler::new(2);

        for key in [1, 2, 3] {
            let results_c = results.clone();
            delayed_handler.add_handler(
                &key,
                Box::new(move |data: String| {
                    results_c.lock().unwrap().push(format!("{key}:{data}"));
                }),
            );
        }

        assert!(delayed_handler.schedule_handler(&1, "a".to_string()).is_ok());
        assert!(delayed_handler.schedule_handler(&2, "b".to_string()).is_ok());
        assert!(delayed_handler.schedule_handler(&3, "c".to_string()).is_ok());

        sleep(Duration::from_secs(1)).await;

        let mut collected = results.lock().unwrap().clone();
        collected.sort();
        assert_eq!(collected, vec!["1:a", "2:b", "3:c"]);
    }
}
