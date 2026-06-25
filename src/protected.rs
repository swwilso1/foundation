//! The `protected` module provides a simple wrapper around `Arc<Mutex<T>>` to allow for safe
//! sharing of data between threads.

use std::sync::{Arc, Mutex, MutexGuard};

/// A simple wrapper around `Arc<Mutex<T>>` to allow for safe sharing of data between threads.
/// Note that the type protected by this wrapper must implement Clone.
#[derive(Debug, Clone)]
pub struct Protected<T> {
    /// The `Arc<Mutex<T>>` that holds the data.
    item: Arc<Mutex<T>>,
}

impl<T> Protected<T> {
    /// Create a new `Protected<T>` with the given item.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to protect.
    ///
    /// # Returns
    ///
    /// A new `Protected<T>` containing the given item.
    pub fn new(item: T) -> Protected<T> {
        Protected {
            item: Arc::new(Mutex::new(item)),
        }
    }

    /// Lock the protected item for access.
    ///
    /// This function assumes that unwrap() of the `Mutex` after a
    /// lock operation will always succeed.
    ///
    /// # Returns
    ///
    /// A `MutexGuard<T>` that allows access to the protected item.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.item.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;
    use std::thread;

    #[test]
    fn test_create() {
        let protected_int = Protected::new(32);
        assert_eq!(protected_int.lock().deref(), &32);
    }

    #[test]
    fn test_mutate_through_lock() {
        let protected_int = Protected::new(0);
        *protected_int.lock() += 5;
        *protected_int.lock() += 10;
        assert_eq!(*protected_int.lock(), 15);
    }

    #[test]
    fn test_clone_shares_underlying_data() {
        // Cloning a `Protected<T>` clones the `Arc`, so both handles must observe
        // mutations made through either one.
        let original = Protected::new(String::from("a"));
        let clone = original.clone();

        original.lock().push('b');
        assert_eq!(*clone.lock(), "ab");

        clone.lock().push('c');
        assert_eq!(*original.lock(), "abc");
    }

    #[test]
    fn test_protects_non_clone_payload() {
        // The wrapper itself is `Clone` (it clones the `Arc`) even when the
        // protected value would be awkward to clone, such as a `Vec`.
        let protected_vec = Protected::new(Vec::<i32>::new());
        protected_vec.lock().push(1);
        protected_vec.lock().push(2);
        assert_eq!(*protected_vec.lock(), vec![1, 2]);
    }

    #[test]
    fn test_shared_across_threads() {
        let counter = Protected::new(0u64);
        let thread_count = 8;
        let increments = 1000;

        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let counter = counter.clone();
                thread::spawn(move || {
                    for _ in 0..increments {
                        *counter.lock() += 1;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(*counter.lock(), thread_count * increments);
    }

    #[test]
    fn test_debug_formatting() {
        let protected_int = Protected::new(42);
        let rendered = format!("{:?}", protected_int);
        assert!(rendered.contains("Protected"));
    }
}
